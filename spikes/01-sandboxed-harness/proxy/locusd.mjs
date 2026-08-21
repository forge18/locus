// locusd — the host daemon, Spike 1 subset. Node 24, zero dependencies.
//
// PLAN.md §Credentials puts three things at one chokepoint: credential
// injection, egress policy tiers, and an audit row per outbound call. This
// implements all three, because whether they can share a chokepoint is the
// spike's Open question and answering it here is nearly free.
//
// Three listeners:
//   1. HTTP  127.0.0.1:$LOCUS_PROXY_PORT   the credential proxy (container -> upstream)
//   2. UNIX  $LOCUS_SOCK_PATH              JSON-RPC 2.0, what `locus` calls
//   3. HTTP  127.0.0.1:$LOCUS_MOCK_PORT    a mock upstream that ASSERTS what it received
//
// The mock upstream is what makes the mechanism provable without a live API
// key: it accepts exactly one credential value and rejects everything else, so
// "the sentinel is worthless off-host" is an observation, not an assertion.

import http from 'node:http';
import https from 'node:https';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const PROXY_PORT = Number(process.env.LOCUS_PROXY_PORT || 43800);
const MOCK_PORT  = Number(process.env.LOCUS_MOCK_PORT  || 43801);
const SOCK_PATH  = process.env.LOCUS_SOCK_PATH || '/tmp/locus.sock';
const AUDIT      = process.env.LOCUS_AUDIT || './out/audit.ndjson';

// --- the host credential store ----------------------------------------------
//
// Locus has to offer BOTH ways in: paste an API key, or sign in. They are not
// alternatives to choose between — most people running Claude Code are on a
// subscription and have no API key at all, and anyone driving it from CI has a
// key and no browser. A design that serves only one of them serves half the
// users.
//
// Both land in the same place: a host-side store, outside any repo, mode 0600.
// The container never sees either kind. What differs is only the header the
// proxy writes on the way out, and whether a refresh is needed first:
//
//   kind = "api_key"   ->  x-api-key: <key>
//   kind = "oauth"     ->  authorization: Bearer <access_token>, refreshed on
//                          expiry using the refresh token, which never leaves
//                          this process either
//
// The real credential NEVER leaves this process: not to a file it did not come
// from, not to a log, not into a response body.
const STORE = process.env.LOCUS_CRED_STORE ||
  path.join(process.env.HOME || '/tmp', '.local/state/locus-spike/credentials.json');

function loadCredential() {
  // The environment wins when set, because that is how the spike's own tests
  // inject a synthetic credential without touching the operator's store.
  if (process.env.LOCUS_SPIKE_REAL_KEY) {
    return { kind: process.env.LOCUS_SPIKE_CRED_KIND || 'api_key',
             secret: process.env.LOCUS_SPIKE_REAL_KEY, origin: 'env' };
  }
  if (fs.existsSync(STORE)) {
    const c = JSON.parse(fs.readFileSync(STORE, 'utf8'));
    return { ...c, origin: 'store' };
  }
  return null;
}

let CRED = loadCredential();
if (!CRED) {
  console.error(JSON.stringify({ error: 'no credential', store: STORE,
    hint: 'run ./set-credential.sh api-key   or   ./set-credential.sh sign-in' }));
  process.exit(2);
}

// Kept for the tests that ask "is this value the real one" without knowing kind.
const REAL = CRED.secret ?? CRED.access_token;

// An oauth credential carries an expiry. Refreshing is the host's job and
// happens before injection, so a container never sees a 401 it cannot fix.
let refreshes = 0;
async function currentSecret() {
  if (CRED.kind !== 'oauth') return CRED.secret;
  const skew = 60_000;
  if (CRED.expires_at && Date.now() + skew < CRED.expires_at) return CRED.access_token;
  if (!CRED.refresh_url || !CRED.refresh_token) return CRED.access_token;
  const res = await fetch(CRED.refresh_url, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ grant_type: 'refresh_token', refresh_token: CRED.refresh_token }),
  });
  const body = await res.json();
  CRED.access_token = body.access_token ?? CRED.access_token;
  CRED.expires_at = Date.now() + (body.expires_in ?? 3600) * 1000;
  refreshes++;
  audit({ verb: 'credential', action: 'refresh', kind: 'oauth', ok: res.ok });
  return CRED.access_token;
}

// Where the proxy forwards. Default is the mock; set to https://api.anthropic.com
// for the live run.
const UPSTREAM = new URL(process.env.LOCUS_UPSTREAM || `http://127.0.0.1:${MOCK_PORT}`);

// Sentinel: what candidate A puts in the container. Deliberately shaped like a
// key so that a harness which validates the format still starts.
const SENTINEL = process.env.LOCUS_SENTINEL || 'sk-locus-sentinel-' + '0'.repeat(32);

// Egress policy tiers (PLAN.md §Credentials: "per-agent network policy tiers").
const TIERS = {
  none:     [],
  model:    ['api.anthropic.com', '127.0.0.1', 'localhost'],
  packages: ['api.anthropic.com', 'registry.npmjs.org', 'crates.io', '127.0.0.1', 'localhost'],
  open:     ['*'],
};
const TIER = process.env.LOCUS_EGRESS_TIER || 'model';

// --- per-run tokens: candidate B -------------------------------------------
const tokens = new Map();   // token -> { run_id, expires, revoked }
const TOKEN_TTL_MS = Number(process.env.LOCUS_TOKEN_TTL_MS || 60_000);

function mint(run_id) {
  const token = 'locus-run-' + crypto.randomBytes(24).toString('hex');
  tokens.set(token, { run_id, expires: Date.now() + TOKEN_TTL_MS, revoked: false });
  return { token, expires_in_ms: TOKEN_TTL_MS };
}
function tokenState(token) {
  const t = tokens.get(token);
  if (!t) return 'unknown';
  if (t.revoked) return 'revoked';
  if (Date.now() > t.expires) return 'expired';
  return 'valid';
}
function revokeRun(run_id) {
  let n = 0;
  for (const [, t] of tokens) if (t.run_id === run_id && !t.revoked) { t.revoked = true; n++; }
  return n;
}

// --- audit ------------------------------------------------------------------
fs.mkdirSync(path.dirname(AUDIT), { recursive: true });
function audit(row) {
  // seq and ts are the host's, never the container's.
  fs.appendFileSync(AUDIT, JSON.stringify({ ts: new Date().toISOString(), ...row }) + '\n');
}

function presentedCredential(headers) {
  if (headers['x-api-key']) return { header: 'x-api-key', value: headers['x-api-key'] };
  const a = headers['authorization'];
  if (a && /^bearer /i.test(a)) return { header: 'authorization', value: a.slice(7) };
  return null;
}

// classify without ever logging the value
function classify(value) {
  if (!value) return 'absent';
  if (value === SENTINEL) return 'sentinel';
  if (value === REAL || value === CRED.access_token) return 'real-credential';
  if (value === CRED.refresh_token) return 'refresh-token';
  if (tokens.has(value)) return `run-token:${tokenState(value)}`;
  return 'unrecognized';
}

// --- 1. the credential proxy -------------------------------------------------
const proxy = http.createServer((req, res) => {
  const presented = presentedCredential(req.headers);
  const kind = classify(presented?.value);

  const allowed = TIERS[TIER]?.includes('*') || TIERS[TIER]?.includes(UPSTREAM.hostname);
  if (!allowed) {
    audit({ verb: 'egress', decision: 'denied', tier: TIER, host: UPSTREAM.hostname, path: req.url, credential: kind });
    res.writeHead(403, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({ type: 'error', error: { type: 'egress_denied', tier: TIER, host: UPSTREAM.hostname } }));
  }

  // Injection: only a credential the host itself issued is upgraded. An
  // unrecognized value is forwarded UNCHANGED, so a container cannot smuggle
  // its own key out through the audited path and have it laundered.
  let inject = false;
  if (kind === 'sentinel') inject = true;
  else if (kind === 'run-token:valid') inject = true;

  if (kind.startsWith('run-token:') && !inject) {
    audit({ verb: 'egress', decision: 'denied', reason: kind, tier: TIER, path: req.url });
    res.writeHead(401, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({ type: 'error', error: { type: 'authentication_error', message: kind } }));
  }

  const headers = { ...req.headers, host: UPSTREAM.host };
  const send = async () => {
  if (inject) {
    // The header is chosen by the CREDENTIAL's kind, not by what the container
    // presented. A container asking with x-api-key and a host holding an OAuth
    // token is the ordinary case for a subscription user, and it has to work.
    const secret = await currentSecret();
    delete headers['x-api-key'];
    delete headers['authorization'];
    if (CRED.kind === 'oauth') {
      headers['authorization'] = 'Bearer ' + secret;
      headers['anthropic-beta'] = [headers['anthropic-beta'], 'oauth-2025-04-20']
        .filter(Boolean).join(',');
    } else {
      headers['x-api-key'] = secret;
    }
  }
  delete headers['content-length'];
  delete headers['accept-encoding'];

  const mod = UPSTREAM.protocol === 'https:' ? https : http;
  const up = mod.request({
    protocol: UPSTREAM.protocol,
    hostname: UPSTREAM.hostname,
    port: UPSTREAM.port || (UPSTREAM.protocol === 'https:' ? 443 : 80),
    method: req.method,
    path: req.url,
    headers,
  }, (upRes) => {
    audit({ verb: 'egress', decision: 'allowed', tier: TIER, host: UPSTREAM.hostname,
            path: req.url, credential_presented: kind, injected: inject, status: upRes.statusCode });
    res.writeHead(upRes.statusCode, upRes.headers);
    upRes.pipe(res);
  });
  up.on('error', (e) => {
    audit({ verb: 'egress', decision: 'error', host: UPSTREAM.hostname, path: req.url, error: e.code || e.message });
    res.writeHead(502, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ type: 'error', error: { type: 'upstream', message: e.code || e.message } }));
  });
  req.pipe(up);
  };
  send().catch((e) => {
    audit({ verb: 'egress', decision: 'error', reason: 'credential', error: e.message });
    res.writeHead(502, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ type: 'error', error: { type: 'credential', message: e.message } }));
  });
});

// --- 2. the JSON-RPC socket --------------------------------------------------
try { fs.unlinkSync(SOCK_PATH); } catch {}
const RUN_NONCE = process.env.LOCUS_RUN_NONCE || '';
const rpc = http.createServer((req, res) => {
  // When the socket is relayed over TCP (see locus-sockd), the port is
  // reachable by every container on the host, so the run nonce is what puts an
  // authenticator back in a path that a bind-mounted socket authenticated by
  // being mounted at all.
  if (RUN_NONCE && req.headers['x-locus-run-nonce'] !== RUN_NONCE) {
    audit({ verb: 'rpc', decision: 'denied', reason: 'bad-or-missing-run-nonce' });
    res.writeHead(403, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({ jsonrpc: '2.0', id: null,
      error: { code: -32000, message: 'run nonce rejected' } }));
  }
  let body = '';
  req.on('data', (c) => body += c);
  req.on('end', () => {
    let msg; try { msg = JSON.parse(body || '{}'); } catch { msg = {}; }
    const reply = (result) => {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({ jsonrpc: '2.0', id: msg.id ?? null, result }));
    };
    switch (msg.method) {
      case 'ping':
        audit({ verb: 'rpc', method: 'ping' });
        return reply({ ok: true, tier: TIER });
      case 'creds.get': {
        const run_id = msg.params?.run_id || 'unknown';
        const { token, expires_in_ms } = mint(run_id);
        // The audit row records that a token was issued, never the token.
        audit({ verb: 'rpc', method: 'creds.get', run_id, expires_in_ms });
        return reply({ base_url: `http://host.docker.internal:${PROXY_PORT}`,
                       header: 'x-api-key', token, expires_in_ms });
      }
      case 'creds.revoke': {
        const run_id = msg.params?.run_id || 'unknown';
        const n = revokeRun(run_id);
        audit({ verb: 'rpc', method: 'creds.revoke', run_id, revoked: n });
        return reply({ revoked: n });
      }
      default:
        res.writeHead(400, { 'content-type': 'application/json' });
        res.end(JSON.stringify({ jsonrpc: '2.0', id: msg.id ?? null,
                                 error: { code: -32601, message: 'method not found' } }));
    }
  });
});

// --- 3. the mock upstream ----------------------------------------------------
// Accepts exactly one value. This is the instrument, not the system.
const mock = http.createServer((req, res) => {
  // The provider's token endpoint, so the refresh path is exercised rather than
  // asserted. It mints a new access token and reports the old one as dead.
  if (req.url === '/oauth/token') {
    let b = ''; req.on('data', (c) => b += c);
    return req.on('end', () => {
      CRED.access_token = 'oauth-access-' + crypto.randomBytes(16).toString('hex');
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({ access_token: CRED.access_token, expires_in: 3600 }));
    });
  }
  const presented = presentedCredential(req.headers);
  const wantHeader = CRED.kind === 'oauth' ? 'authorization' : 'x-api-key';
  const ok = presented?.value === (CRED.kind === 'oauth' ? CRED.access_token : CRED.secret)
             && presented?.header === wantHeader;
  fs.appendFileSync(AUDIT, JSON.stringify({
    ts: new Date().toISOString(), verb: 'upstream',
    saw: classify(presented?.value), header: presented?.header ?? null,
    expected_header: wantHeader, accepted: ok, path: req.url,
  }) + '\n');
  if (!ok) {
    res.writeHead(401, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({ type: 'error', error: { type: 'authentication_error',
      message: 'upstream received a credential it does not accept' } }));
  }
  let body = ''; req.on('data', c => body += c);
  req.on('end', () => {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify({
      id: 'msg_mock', type: 'message', role: 'assistant', model: 'mock',
      content: [{ type: 'text', text: 'ok' }],
      usage: { input_tokens: 11, output_tokens: 3, cache_read_input_tokens: 0, cache_creation_input_tokens: 0 },
    }));
  });
});

proxy.listen(PROXY_PORT, '0.0.0.0', () => {});
mock.listen(MOCK_PORT, '127.0.0.1', () => {});
// The same JSON-RPC handler on two transports. A Linux host bind-mounts the
// unix socket; a macOS host relays to the TCP port (locus-sockd). The agent
// sees a unix socket at /run/locus.sock either way.
const RPC_PORT = Number(process.env.LOCUS_RPC_PORT || 43802);
const rpcTcp = http.createServer((req, res) => rpc.emit('request', req, res));

rpc.listen(SOCK_PATH, () => {
  fs.chmodSync(SOCK_PATH, 0o666);
  rpcTcp.listen(RPC_PORT, '0.0.0.0', () => {
    console.log(JSON.stringify({ ready: true, proxy: PROXY_PORT, mock: MOCK_PORT,
                                 sock: SOCK_PATH, rpc_port: RPC_PORT,
                                 upstream: UPSTREAM.origin, tier: TIER,
                                 nonce_required: Boolean(RUN_NONCE),
                                 credential_kind: CRED.kind,
                                 credential_origin: CRED.origin }));
  });
});

for (const sig of ['SIGINT', 'SIGTERM']) {
  process.on(sig, () => { try { fs.unlinkSync(SOCK_PATH); } catch {} process.exit(0); });
}
