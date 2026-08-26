// schema: agents.sessions + agents.runs + agents.events
// replaced by: invoke("sessions_list") + Channel<AgentEvent>("session_events")

import type { AgentEvent, EventVerb, Usage } from "../types/event";
import type { Run, Session, SessionStatus } from "../types/agents";
import { ago, pick, rng } from "./rng";

const AGENTS = [
  "builder@4",
  "builder@3",
  "reviewer@2",
  "planner@3",
  "auditor@1",
  "ingest@2",
];
const ROLES = [
  "builder",
  "builder",
  "reviewer",
  "planner",
  "auditor",
  "ingest",
];
const PROJECTS = ["p-tapestry", "p-loom-db", "p-weaver", "p-texere"];
const REPOS = ["r-tapestry-app", "r-loom-db", "r-weaver", "r-texere"];
const STATUSES: SessionStatus[] = [
  "running",
  "running",
  "running",
  "waiting",
  "idle",
  "stuck",
  "done",
];
const MODELS = [
  "claude-opus-5",
  "claude-sonnet-5",
  "gpt-5.2-codex",
  "gemini-3-pro",
];

const usageFor = (next: () => number): Usage => ({
  input: Math.floor(next() * 180_000) + 4_000,
  output: Math.floor(next() * 24_000) + 500,
  cacheRead: Math.floor(next() * 900_000),
  cacheWrite: Math.floor(next() * 60_000),
});

/** 300 sessions, which is what the Manage list is drawn at. */
export const SESSIONS: Session[] = (() => {
  const next = rng(20260820);
  return Array.from({ length: 300 }, (_, i) => {
    const p = Math.floor(next() * PROJECTS.length);
    const a = Math.floor(next() * AGENTS.length);
    const status = pick(next, STATUSES);
    // Two sessions in ten come from a harness that reports no usage at all. Those
    // read *unknown*, never zero — see `usage: null` below.
    const reportsUsage = next() > 0.2;
    return {
      id: `s-${String(i).padStart(4, "0")}`,
      projectId: PROJECTS[p],
      repoId: REPOS[p],
      agent: AGENTS[a],
      role: ROLES[a],
      status,
      branch: `agent/${(0x8f21 + i).toString(16)}-${pick(next, ["notify", "index", "parser", "ingest", "retry"])}`,
      taskId:
        next() > 0.3
          ? `t-${String(Math.floor(next() * 40)).padStart(3, "0")}`
          : null,
      handedOffFrom:
        next() > 0.92
          ? `s-${String(Math.max(0, i - 3)).padStart(4, "0")}`
          : null,
      runIds: [`r-${String(i).padStart(4, "0")}-0`],
      startedAt: ago(Math.floor(next() * 2_400) + 5),
      lastEventAt: ago(Math.floor(next() * 60)),
      usage: reportsUsage ? usageFor(next) : null,
    };
  });
})();

/** The one session the Agents screen opens on, matching the design copy. */
export const SELECTED_SESSION_ID = SESSIONS[0].id;

export const RUNS: Run[] = SESSIONS.flatMap((s, i) => {
  const next = rng(770_000 + i);
  return s.runIds.map((id) => ({
    id,
    sessionId: s.id,
    status:
      s.status === "running"
        ? ("running" as const)
        : pick(next, ["passed", "passed", "failed", "aborted"] as const),
    startedAt: s.startedAt,
    endedAt: s.status === "running" ? null : s.lastEventAt,
    resolvedModel: pick(next, MODELS),
    permissionPosture: "bypass" as const,
    exitCode: s.status === "running" ? null : Math.floor(next() * 2),
    usage: s.usage,
    artifactIds: [],
  }));
});

const VERB_SEQUENCE: EventVerb[] = [
  "session_start",
  "user",
  "thinking",
  "assistant",
  "tool_call",
  "tool_result",
  "assistant",
  "tool_call",
  "tool_error",
  "subagent_start",
  "subagent_stop",
  "assistant",
];

/**
 * The transcript for one session. Only the twelve canonical verbs appear — a
 * harness that cannot report `thinking` produces no `thinking` events, so a
 * fixture inventing a thirteenth verb would teach the UI a shape that cannot exist.
 */
export function eventsFor(sessionId: string): AgentEvent[] {
  const index = SESSIONS.findIndex((s) => s.id === sessionId);
  const next = rng(31_000 + Math.max(0, index));
  const session = SESSIONS[Math.max(0, index)];

  return VERB_SEQUENCE.map((verb, seq) => ({
    id: `${sessionId}-e${seq}`,
    runId: session.runIds[0],
    seq,
    ts: ago(40 - seq * 3),
    verb,
    text:
      verb === "user"
        ? "Add the notification channel behind the Sink trait, and cover the closed-receiver case."
        : verb === "thinking"
          ? "The Sink trait already has a blanket impl, so the channel can ride it without a new bound."
          : verb === "assistant"
            ? "Threading the channel through Supervisor::spawn now; the bus lock stays untouched."
            : undefined,
    tool: verb.startsWith("tool_")
      ? pick(next, ["read_file", "edit_file", "run_command", "grep"])
      : undefined,
    args:
      verb === "tool_call"
        ? { path: "crates/tapestry-core/src/notify.rs" }
        : undefined,
    usage:
      verb === "assistant" || verb === "session_end"
        ? session.usage
        : undefined,
    raw: { source: "hooks", verb },
  }));
}

/**
 * The handful of sessions the Agents screen actually opens on, with the detail a
 * transcript needs. The 300 above are the list; these are the ones drawn.
 */
export interface SessionDetail {
  id: string;
  project: string;
  agent: string;
  role: string;
  status: SessionStatus;
  task: string;
  /** The tool it is in right now, or null between tools. */
  tool: string | null;
  /** Null where the harness reports nothing — unknown, not zero. */
  tokens: string | null;
  runs: number;
  runId: string;
  /** The shell prompt the transcript ends on. */
  prompt: string;
  /** Minutes since the last event; the activity half of the ordering. */
  idleMinutes: number;
  transcript: TranscriptLine[];
}

export interface TranscriptLine {
  verb: EventVerb;
  text: string;
}

export const SESSION_DETAILS: SessionDetail[] = [
  {
    id: "sd-weaver",
    project: "weaver",
    agent: "builder@4",
    role: "builder",
    status: "stuck",
    task: "Parser: recover from an unterminated block",
    tool: "run_command",
    tokens: "102.3k",
    runs: 3,
    runId: "run-5a71",
    prompt: "weaver ❯ ",
    idleMinutes: 14,
    transcript: [
      {
        verb: "session_start",
        text: "session_start · weaver · builder@4 · run 3 of 3",
      },
      { verb: "user", text: "Make cargo test -p weaver parser:: pass." },
      {
        verb: "thinking",
        text: "The unterminated block case reaches the recovery arm with an empty span.",
      },
      { verb: "tool_call", text: "run_command  cargo test -p weaver parser::" },
      { verb: "tool_error", text: "test result: FAILED. 41 passed; 1 failed" },
      {
        verb: "assistant",
        text: "Same failure as the previous two iterations. Trying a wider span.",
      },
      { verb: "tool_call", text: "edit_file  crates/weaver/src/parser.rs" },
      { verb: "tool_result", text: "1 hunk applied" },
      { verb: "tool_call", text: "run_command  cargo test -p weaver parser::" },
      { verb: "tool_error", text: "test result: FAILED. 41 passed; 1 failed" },
      { verb: "aborted", text: "aborted · guardrail: 3 stuck iterations" },
    ],
  },
  {
    id: "sd-tapestry",
    project: "tapestry",
    agent: "builder@4",
    role: "builder",
    status: "running",
    task: "Thread the channel through Supervisor::spawn",
    tool: "edit_file",
    tokens: "41.2k",
    runs: 1,
    runId: "run-8f21",
    prompt: "tapestry ❯ ",
    idleMinutes: 0,
    transcript: [
      {
        verb: "session_start",
        text: "session_start · tapestry · builder@4 · run 1",
      },
      {
        verb: "user",
        text: "Bound the notify channel and count what it drops.",
      },
      {
        verb: "thinking",
        text: "Sink has a blanket impl, so the channel rides it without a new bound.",
      },
      {
        verb: "tool_call",
        text: "read_file  crates/tapestry-core/src/notify.rs",
      },
      { verb: "tool_result", text: "118 lines" },
      {
        verb: "assistant",
        text: "Adding a Dropped error and a counter, then threading it through spawn.",
      },
      {
        verb: "tool_call",
        text: "edit_file  crates/tapestry-core/src/notify.rs",
      },
      { verb: "tool_result", text: "2 hunks applied" },
    ],
  },
  {
    id: "sd-texere",
    project: "texere",
    agent: "builder@3",
    role: "builder",
    status: "waiting",
    task: "Ingest: dedupe on sha256 rather than path",
    tool: null,
    // This harness reports no usage at all: unknown, not zero.
    tokens: null,
    runs: 2,
    runId: "run-3c04",
    prompt: "texere ❯ ",
    idleMinutes: 6,
    transcript: [
      {
        verb: "session_start",
        text: "session_start · texere · builder@3 · run 2",
      },
      {
        verb: "assistant",
        text: "Plan is written. Waiting on the gate before implementation.",
      },
      { verb: "permission_request", text: "permission_request · gate: human" },
    ],
  },
  {
    id: "sd-loom-db",
    project: "loom-db",
    agent: "builder@4",
    role: "builder",
    status: "idle",
    task: "Online index rebuild",
    tool: null,
    tokens: "18.9k",
    runs: 1,
    runId: "run-9b12",
    prompt: "loom-db ❯ ",
    idleMinutes: 3,
    transcript: [
      {
        verb: "session_start",
        text: "session_start · loom-db · builder@4 · run 1",
      },
      {
        verb: "assistant",
        text: "Asked which migration path to take; nothing to do until that lands.",
      },
    ],
  },
  {
    id: "sd-review",
    project: "tapestry",
    agent: "reviewer@2",
    role: "reviewer",
    status: "running",
    task: "Retry policy on the payments client",
    tool: "read_file",
    tokens: "7.4k",
    runs: 1,
    runId: "run-1d55",
    prompt: "tapestry ❯ ",
    idleMinutes: 2,
    transcript: [
      {
        verb: "session_start",
        text: "session_start · tapestry · reviewer@2 · run 1",
      },
      { verb: "tool_call", text: "read_file  apps/web/src/payments/client.ts" },
      { verb: "tool_result", text: "212 lines" },
      { verb: "subagent_start", text: "subagent_start · security lens" },
      { verb: "subagent_stop", text: "subagent_stop · 0 findings" },
      {
        verb: "assistant",
        text: "Idempotency key is threaded. No double-charge on retry.",
      },
    ],
  },
];

export const SELECTED_DETAIL_ID = "sd-weaver";

export const PTY_NOTE = "PTY attached from the host · one session per terminal";

export const SESSION_LIST_FOOTER =
  "Sorted by needs-attention, then activity. Selecting one does not close the others — a session you stopped watching is not a session you ended.";

export const GUARDRAIL_NOTE = "kill & reassign after 3 stuck iterations";

export const HANDOFF_SUMMARY =
  "Handoff carries: the goal, what passed, the two spans already tried, and the open question about empty-span recovery.";

export const WAITING_NOTE = "Waiting ≠ idle.";
