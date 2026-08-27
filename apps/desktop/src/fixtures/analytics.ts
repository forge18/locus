export type AnalyticsRange = "24h" | "7d" | "30d" | "90d" | "all";
export type AnalyticsScope = "all" | string;
export type AnalyticsMeasure = "spend" | "tokens" | "cache" | "runs";
export type BreakdownDimension =
  | "Model"
  | "Harness"
  | "Agent"
  | "Role"
  | "Workflow";
export type ExtensionKind =
  | "all"
  | "skill"
  | "rule"
  | "hook"
  | "linter"
  | "style"
  | "agent";

export const ANALYTICS_RANGES = Object.freeze([
  { value: "24h", label: "24h", buckets: 24, unit: "hour" },
  { value: "7d", label: "7d", buckets: 7, unit: "day" },
  { value: "30d", label: "30d", buckets: 30, unit: "day" },
  { value: "90d", label: "90d", buckets: 13, unit: "week" },
  { value: "all", label: "All", buckets: 12, unit: "month" },
] as const);

export const ANALYTICS_MEASURES = Object.freeze([
  { value: "spend", label: "Spend" },
  { value: "tokens", label: "Tokens" },
  { value: "cache", label: "Cache read" },
  { value: "runs", label: "Runs" },
] as const);

export const BREAKDOWN_DIMENSIONS: readonly BreakdownDimension[] =
  Object.freeze(["Model", "Harness", "Agent", "Role", "Workflow"]);
export const EXTENSION_KINDS: readonly ExtensionKind[] = Object.freeze([
  "all",
  "skill",
  "rule",
  "hook",
  "linter",
  "style",
  "agent",
]);

export interface AnalyticsStat {
  id: AnalyticsMeasure;
  label: string;
  value: string;
  note: string;
}
export const ANALYTICS_STATS: readonly AnalyticsStat[] = Object.freeze([
  { id: "spend", label: "Spend", value: "$184.20", note: "all providers" },
  { id: "tokens", label: "Tokens", value: "12.4M", note: "input + output" },
  { id: "cache", label: "Cache read", value: "88%", note: "prefix tokens" },
  { id: "runs", label: "Runs", value: "612", note: "scheduled and manual" },
]);

export interface AtAGlanceMetric {
  id: string;
  label: string;
  value: string;
  note: string;
}
export const AT_A_GLANCE_METRICS: readonly AtAGlanceMetric[] = Object.freeze([
  {
    id: "verify-pass-rate",
    label: "Verify pass rate",
    value: "79%",
    note: "484 / 612 runs",
  },
  {
    id: "guardrail-trips",
    label: "Guardrail trips",
    value: "38",
    note: "last 30 days",
  },
  {
    id: "spec-gap-rate",
    label: "Spec-gap rate",
    value: "7.4%",
    note: "arbiter classifications",
  },
  {
    id: "agent-trust",
    label: "Agent trust",
    value: "0.82",
    note: "last 20 runs · token-discounted",
  },
]);

export interface BreakdownRow {
  dimension: string;
  tokens: string;
  cache: string;
  spend: string;
  runs: number;
  perRun: string;
}
export const ANALYTICS_BREAKDOWN: readonly BreakdownRow[] = Object.freeze([
  {
    dimension: "claude-opus-5",
    tokens: "4.8M",
    cache: "91%",
    spend: "$82.40",
    runs: 188,
    perRun: "$0.44",
  },
  {
    dimension: "gpt-5.2-codex",
    tokens: "3.2M",
    cache: "84%",
    spend: "$54.10",
    runs: 141,
    perRun: "$0.38",
  },
  {
    dimension: "builder@4",
    tokens: "2.1M",
    cache: "89%",
    spend: "$31.20",
    runs: 96,
    perRun: "$0.33",
  },
]);

export interface TaskOutcome {
  label: string;
  count: number;
}
export const TASK_OUTCOMES: readonly TaskOutcome[] = Object.freeze([
  { label: "Landed", count: 74 },
  { label: "Abandoned", count: 9 },
  { label: "Still open", count: 18 },
  { label: "Landed after rework", count: 12 },
]);

export interface WorkflowTiming {
  workflow: string;
  runs: number;
  median: string;
  p90: string;
  iterations: string;
  verified: number;
}
export const WORKFLOW_TIMINGS: readonly WorkflowTiming[] = Object.freeze([
  {
    workflow: "builder → verify",
    runs: 244,
    median: "8m",
    p90: "24m",
    iterations: "2.1",
    verified: 219,
  },
  {
    workflow: "reviewer → self-review",
    runs: 96,
    median: "4m",
    p90: "12m",
    iterations: "1.4",
    verified: 91,
  },
]);

export interface RetrievalTier {
  tier: string;
  hits: number;
  useful: string;
  averageTokens: string;
}
export const RETRIEVAL_TIERS: readonly RetrievalTier[] = Object.freeze([
  { tier: "Short-term", hits: 1204, useful: "unknown", averageTokens: "1.2k" },
  { tier: "Long-term", hits: 442, useful: "72%", averageTokens: "0.8k" },
  { tier: "Artifacts", hits: 318, useful: "64%", averageTokens: "2.4k" },
  { tier: "Wiki", hits: 156, useful: "58%", averageTokens: "1.8k" },
]);

export interface ExtensionUsage {
  kind: Exclude<ExtensionKind, "all">;
  name: string;
  hits: number;
  note: string;
}
export const EXTENSION_USAGE: readonly ExtensionUsage[] = Object.freeze([
  {
    kind: "skill",
    name: "dispatch-review",
    hits: 88,
    note: "loaded in 34% of runs",
  },
  {
    kind: "rule",
    name: "git-authority",
    hits: 612,
    note: "loaded in every run",
  },
  { kind: "hook", name: "session-end", hits: 594, note: "1 failing" },
  { kind: "linter", name: "eslint", hits: 44, note: "invoked by QA checks" },
  {
    kind: "style",
    name: "brief-bright-gone",
    hits: 612,
    note: "loaded in every run",
  },
  {
    kind: "agent",
    name: "builder@4",
    hits: 244,
    note: "materialized for 244 runs",
  },
]);

export const TELEMETRY_FACETS = Object.freeze([
  {
    key: "harness",
    label: "harness",
    values: ["claude 412", "codex 88", "cursor 41"],
  },
  {
    key: "project",
    label: "project",
    values: ["tapestry 288", "loom-db 163", "weaver 122"],
  },
  {
    key: "agent-role",
    label: "agent · role",
    values: ["builder@4 · impl 201", "reviewer@2 · review 140"],
  },
  {
    key: "model-tier",
    label: "model tier",
    values: ["high 334", "medium 180", "low 45"],
  },
  {
    key: "verify",
    label: "verify",
    values: ["failed 126", "passed 484", "aborted 15"],
  },
  {
    key: "arbiter-class",
    label: "arbiter class",
    values: ["bug 61", "spec gap 34", "noise 21"],
  },
  { key: "branch", label: "branch", values: ["agent/* 641", "main 0"] },
]);

export const TELEMETRY_VERBS = Object.freeze([
  "tool_call",
  "tool_result",
  "assistant",
  "thinking",
  "user",
  "tool_error",
  "subagent_start",
  "subagent_stop",
  "session_start",
  "session_end",
  "aborted",
  "permission_request",
]);

export const TELEMETRY_ACTIONS = Object.freeze([
  { verb: "tool_call", count: 50796 },
  { verb: "tool_result", count: 48708 },
  { verb: "assistant", count: 47837 },
  { verb: "thinking", count: 11264 },
  { verb: "user", count: 5611 },
  { verb: "tool_error", count: 2190 },
  { verb: "subagent_start", count: 1318 },
  { verb: "subagent_stop", count: 1311 },
  { verb: "session_start", count: 641 },
  { verb: "session_end", count: 626 },
  { verb: "aborted", count: 15 },
  { verb: "permission_request", count: 2 },
]);

export const TELEMETRY_SESSIONS = Object.freeze([
  {
    when: "2026-08-20 09:35",
    harness: "claude",
    project: "tapestry",
    repo: "core",
    agent: "builder@4",
    role: "impl",
    models: "claude-opus-5",
    runs: 3,
    events: 1402,
    errors: 18,
    tokens: "41.2k",
    status: "running",
    id: "9cd39051",
  },
  {
    when: "2026-08-20 08:22",
    harness: "codex",
    project: "loom-db",
    repo: "db",
    agent: "auditor@2",
    role: "audit",
    models: "gpt-5.2",
    runs: 1,
    events: 1694,
    errors: 16,
    tokens: "9.4k",
    status: "closed",
    id: "a5abc2c9",
  },
]);
