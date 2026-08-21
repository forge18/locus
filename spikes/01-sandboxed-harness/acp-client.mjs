// A minimal ACP client, on the HOST, driving a harness in a container.
//
// PLAN.md §ACP puts the client in the core, not in the container: the harness
// speaks ACP over its own stdio and the core holds the conversation. So this
// spawns `docker exec -i <container> hermes acp` and talks JSON-RPC 2.0 over
// that pipe, exactly as the core would over a PTY-less child.
//
// Every inbound message is written verbatim to the capture file BEFORE anything
// interprets it. PLAN.md: capture is separated from normalization so a
// normalization bug is repairable by replay.
//
// Usage: node acp-client.mjs <container> <capture.ndjson> <prompt> [harness-argv...]

import { spawn } from 'node:child_process';
import fs from 'node:fs';

const [, , container, capturePath, prompt, ...harnessArgv] = process.argv;
if (!container || !capturePath || !prompt) {
  console.error('usage: acp-client.mjs <container> <capture.ndjson> <prompt> [harness-argv...]');
  process.exit(2);
}

const argv = harnessArgv.length ? harnessArgv : ['hermes', 'acp', '--accept-hooks'];
const capture = fs.createWriteStream(capturePath, { flags: 'w' });
const child = spawn('docker', ['exec', '-i', container, ...argv], { stdio: ['pipe', 'pipe', 'pipe'] });

let nextId = 1;
const pending = new Map();
let stderrTail = '';

child.stderr.on('data', (d) => {
  stderrTail = (stderrTail + d.toString()).slice(-4000);
});

const send = (msg) => child.stdin.write(JSON.stringify(msg) + '\n');
const request = (method, params) => new Promise((resolve, reject) => {
  const id = nextId++;
  pending.set(id, { resolve, reject });
  send({ jsonrpc: '2.0', id, method, params });
});
const respond = (id, result) => send({ jsonrpc: '2.0', id, result });

let buf = '';
child.stdout.on('data', (chunk) => {
  buf += chunk.toString();
  let i;
  while ((i = buf.indexOf('\n')) >= 0) {
    const line = buf.slice(0, i).trim();
    buf = buf.slice(i + 1);
    if (!line) continue;

    capture.write(line + '\n');          // verbatim, before interpretation

    let msg;
    try { msg = JSON.parse(line); } catch { continue; }

    if (msg.id !== undefined && (msg.result !== undefined || msg.error !== undefined)) {
      const p = pending.get(msg.id);
      if (p) { pending.delete(msg.id); msg.error ? p.reject(new Error(JSON.stringify(msg.error))) : p.resolve(msg.result); }
      continue;
    }

    // Agent -> client requests. Answering them is the client's job; a run that
    // does not answer hangs, which is the failure `permission_request` exists
    // to make visible.
    if (msg.id !== undefined && msg.method) {
      switch (msg.method) {
        case 'session/request_permission':
          // Every harness launches with its own gate OFF, so this firing at all
          // is the misconfiguration alarm. Answer it so the run completes, and
          // let the normalizer record that it happened.
          respond(msg.id, { outcome: { outcome: 'selected',
            optionId: msg.params?.options?.[0]?.optionId ?? 'allow' } });
          break;
        case 'fs/read_text_file':
        case 'fs/write_text_file':
          // Declared unsupported in clientCapabilities; refuse rather than reach
          // into the container's filesystem from the host.
          send({ jsonrpc: '2.0', id: msg.id,
                 error: { code: -32601, message: 'client does not provide fs' } });
          break;
        default:
          respond(msg.id, {});
      }
    }
  }
});

const fail = (why, code = 1) => {
  console.error(JSON.stringify({ error: why, stderr_tail: stderrTail.split('\n').slice(-12) }));
  try { child.kill(); } catch {}
  process.exit(code);
};

child.on('exit', (code) => {
  if (code !== 0) fail(`harness exited ${code}`, code ?? 1);
});

const timeout = setTimeout(() => fail('timed out waiting for the harness', 4), 180_000);

try {
  const init = await request('initialize', {
    protocolVersion: 1,
    clientCapabilities: { fs: { readTextFile: false, writeTextFile: false }, terminal: false },
  });
  const session = await request('session/new', { cwd: '/workspace', mcpServers: [] });
  const sessionId = session.sessionId ?? session.session_id;
  const result = await request('session/prompt', {
    sessionId,
    prompt: [{ type: 'text', text: prompt }],
  });
  clearTimeout(timeout);
  capture.end();
  console.log(JSON.stringify({
    ok: true, protocol: init.protocolVersion ?? null,
    agent: init.agentCapabilities ? Object.keys(init.agentCapabilities) : null,
    session: sessionId, stop_reason: result.stopReason ?? result.stop_reason ?? null,
    capture: capturePath,
  }));
  child.stdin.end();
  child.kill();
  process.exit(0);
} catch (e) {
  clearTimeout(timeout);
  capture.end();
  fail(e.message, 5);
}
