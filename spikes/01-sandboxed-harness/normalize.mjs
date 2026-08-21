// normalize — source records in, canonical events out.
//
// PLAN.md §Materializers fixes the vocabulary and three rules:
//   * ordering is Locus's — `seq` is assigned on arrival at the core
//   * a missing verb is recorded as missing, never synthesized
//   * `raw` is kept on every event, so a normalization bug is repairable by
//     replay rather than by re-running the agent
//
// Usage: node normalize.mjs <source> <run_id> <out.json> <input...>
//   hooks-claude   <events.ndjson> <transcript.jsonl...>
//   session-log    <aider .aider.chat.history.md / llm history jsonl>
//   stream-json    <stdout.ndjson>
//   acp            <session-update.ndjson>

import fs from 'node:fs';

const VOCAB = new Set([
  'session_start', 'user', 'assistant', 'thinking', 'tool_call', 'tool_result',
  'tool_error', 'permission_request', 'subagent_start', 'subagent_stop',
  'aborted', 'session_end',
]);

const [, , source, runId, outPath, ...inputs] = process.argv;
if (!source || !runId || !outPath) {
  console.error('usage: normalize.mjs <source> <run_id> <out.json> <input...>');
  process.exit(2);
}

const readNdjson = (p) => {
  if (!fs.existsSync(p)) return [];
  return fs.readFileSync(p, 'utf8').split('\n').filter(Boolean).flatMap((l) => {
    try { return [JSON.parse(l)]; } catch { return []; }
  });
};

// usage exactly as the harness reports it. Absent stays absent — never zero.
const usageFrom = (u) => {
  if (!u) return null;
  const pick = (...keys) => { for (const k of keys) if (typeof u[k] === 'number') return u[k]; return null; };
  const out = {
    input:       pick('input_tokens', 'input', 'prompt_tokens'),
    output:      pick('output_tokens', 'output', 'completion_tokens'),
    cache_read:  pick('cache_read_input_tokens', 'cache_read'),
    cache_write: pick('cache_creation_input_tokens', 'cache_write'),
  };
  return Object.values(out).every((v) => v === null) ? null : out;
};

const events = [];
const emit = (kind, ts, raw, extra = {}) => {
  if (!VOCAB.has(kind)) throw new Error(`normalize: '${kind}' is not in the canonical vocabulary`);
  events.push({ run_id: runId, kind, ts: ts || null, ...extra, raw });
};

// --- hooks (claude) ----------------------------------------------------------
// The hook stream is the richest path — tool name and arguments arrive already
// separated — but it cannot see the model's own output. `assistant`, `thinking`
// and `usage` come from the transcript the harness writes, which is exactly why
// harnesses/claude.toml declares log_dir alongside source = "hooks".
function hooksClaude(eventsFile, transcripts) {
  const HOOK_TO_VERB = {
    SessionStart:     'session_start',
    UserPromptSubmit: 'user',
    PreToolUse:       'tool_call',
    SubagentStop:     'subagent_stop',
    SessionEnd:       'session_end',
    // Stop and PreCompact have no canonical verb. They are dropped rather than
    // mapped onto something close: "a missing verb is recorded as missing".
  };

  for (const rec of readNdjson(eventsFile)) {
    const h = rec.hook;
    const raw = rec.raw ?? {};

    if (h === 'PostToolUse') {
      const r = raw.tool_response ?? raw.toolResponse ?? {};
      const failed = r?.is_error === true || r?.isError === true ||
                     (typeof r?.error === 'string' && r.error.length > 0) ||
                     (typeof r?.stderr === 'string' && r.stderr.length > 0 && r?.exit_code);
      emit(failed ? 'tool_error' : 'tool_result', raw.timestamp, raw,
           { tool: raw.tool_name ?? raw.toolName ?? null });
      continue;
    }
    if (h === 'Notification') {
      // A Notification is only a permission_request when it is one. PLAN.md
      // calls permission_request a misconfiguration alarm, so widening it to
      // every notification would make the alarm meaningless.
      const msg = String(raw.message ?? '');
      if (/permission|approve|allow/i.test(msg)) emit('permission_request', raw.timestamp, raw);
      continue;
    }
    const verb = HOOK_TO_VERB[h];
    if (!verb) continue;
    emit(verb, raw.timestamp, raw, verb === 'tool_call'
      ? { tool: raw.tool_name ?? raw.toolName ?? null, args: raw.tool_input ?? raw.toolInput ?? null }
      : {});
  }

  for (const t of transcripts) {
    for (const rec of readNdjson(t)) {
      const msg = rec.message ?? rec;
      if (rec.type === 'assistant' || msg?.role === 'assistant') {
        const content = Array.isArray(msg?.content) ? msg.content : [];
        for (const block of content) {
          if (block.type === 'thinking' || block.type === 'redacted_thinking') {
            emit('thinking', rec.timestamp, rec);
          }
        }
        const hasText = content.some((b) => b.type === 'text' && b.text?.trim());
        const usage = usageFrom(msg?.usage);
        if (hasText || usage) {
          emit('assistant', rec.timestamp, rec,
               { usage, subagent: rec.isSidechain === true });
        }
      }
    }
  }
}

// --- session-log -------------------------------------------------------------
// The weakest path, and PLAN.md says so: ordering is file position, `thinking`
// is usually absent, and `usage` often appears only in the final record.
function sessionLog(files) {
  for (const f of files) {
    for (const rec of readNdjson(f)) {
      const kind = rec.kind ?? rec.event ?? rec.type;
      const map = {
        session_start: 'session_start', start: 'session_start',
        user: 'user', prompt: 'user',
        assistant: 'assistant', response: 'assistant', message: 'assistant',
        tool_call: 'tool_call', tool_use: 'tool_call', command: 'tool_call',
        tool_result: 'tool_result', tool_output: 'tool_result',
        tool_error: 'tool_error', error: 'tool_error',
        session_end: 'session_end', end: 'session_end',
      };
      const verb = map[kind];
      if (!verb) continue;
      emit(verb, rec.ts ?? rec.timestamp, rec, {
        tool: rec.tool ?? rec.name ?? null,
        usage: usageFrom(rec.usage),
      });
    }
  }
}

// --- stream-json -------------------------------------------------------------
// The harness's own newline-delimited JSON on stdout. The TOML declares which
// key holds the record type and its value -> verb table; this is the claude
// dialect of it, used when a second capture source is exercised through the
// same binary.
function streamJson(files) {
  for (const f of files) {
    for (const rec of readNdjson(f)) {
      const t = rec.type;
      if (t === 'system' && rec.subtype === 'init') { emit('session_start', null, rec); continue; }
      if (t === 'user') {
        const content = rec.message?.content;
        const results = Array.isArray(content) ? content.filter((b) => b.type === 'tool_result') : [];
        if (results.length) {
          for (const r of results) emit(r.is_error ? 'tool_error' : 'tool_result', null, rec, { tool: r.tool_use_id ?? null });
        } else {
          emit('user', null, rec);
        }
        continue;
      }
      if (t === 'assistant') {
        const content = rec.message?.content ?? [];
        for (const b of content) {
          if (b.type === 'thinking') emit('thinking', null, rec);
          if (b.type === 'tool_use') emit('tool_call', null, rec, { tool: b.name, args: b.input ?? null });
        }
        const usage = usageFrom(rec.message?.usage);
        if (content.some((b) => b.type === 'text' && b.text?.trim()) || usage) {
          emit('assistant', null, rec, { usage });
        }
        continue;
      }
      if (t === 'result') {
        emit(rec.subtype === 'error_during_execution' ? 'aborted' : 'session_end', null, rec,
             { usage: usageFrom(rec.usage) });
        continue;
      }
    }
  }
}

// --- acp ---------------------------------------------------------------------
// One mapping for every ACP harness, not one per harness.
function acp(files) {
  for (const f of files) {
    for (const rec of readNdjson(f)) {
      const u = rec.params?.update ?? rec.update ?? rec;
      switch (u.sessionUpdate ?? u.type) {
        case 'agent_message_chunk': emit('assistant', rec.ts, rec); break;
        case 'agent_thought_chunk': emit('thinking', rec.ts, rec); break;
        case 'tool_call':           emit('tool_call', rec.ts, rec, { tool: u.title ?? u.kind ?? null }); break;
        case 'tool_call_update':
          emit(u.status === 'failed' ? 'tool_error' : 'tool_result', rec.ts, rec, { tool: u.title ?? null });
          break;
        case 'user_message_chunk':  emit('user', rec.ts, rec); break;
        default: break;
      }
      if (rec.method === 'session/request_permission') emit('permission_request', rec.ts, rec);
    }
  }
}

switch (source) {
  case 'hooks-claude': hooksClaude(inputs[0], inputs.slice(1)); break;
  case 'session-log':  sessionLog(inputs); break;
  case 'stream-json':  streamJson(inputs); break;
  case 'acp':          acp(inputs); break;
  default: console.error(`normalize: unknown source '${source}'`); process.exit(2);
}

// Ordering is Locus's. A source with no ordering guarantee still yields a
// totally ordered stream, so `seq` is assigned last and by arrival.
events.sort((a, b) => (a.ts && b.ts) ? String(a.ts).localeCompare(String(b.ts)) : 0);
events.forEach((e, i) => { e.seq = i; });

fs.writeFileSync(outPath, JSON.stringify(events, null, 2));
const counts = {};
for (const e of events) counts[e.kind] = (counts[e.kind] ?? 0) + 1;
const withUsage = events.filter((e) => e.usage && e.usage.input > 0).length;
console.log(JSON.stringify({ source, run_id: runId, out: outPath, total: events.length,
                             kinds: counts, events_with_real_usage: withUsage }));
