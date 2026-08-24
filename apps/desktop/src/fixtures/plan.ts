// schema: agents.sessions (the ACP planning session) + agents.artifacts (kind = 'plan')
// replaced by: invoke("plans_list") + Channel<AgentEvent>("acp_session_update")

/** The seven planning stages; the auditor is a role, not a stage. */
export const PLAN_STEPS = [
  "Inputs",
  "Orient",
  "Converse",
  "Synthesis",
  "Recommend",
  "Decompose",
  "Approved",
] as const;

export type PlanStep = (typeof PLAN_STEPS)[number];

export type PlanState = "in_progress" | "draft_rejected" | "approved";

export interface PlanSummary {
  id: string;
  title: string;
  project: string;
  state: PlanState;
  /** Which of the seven it is on. The breadcrumb derives its states from this. */
  step: PlanStep;
  /** The step line the card shows. */
  stepLine: string;
  /** Set on a rejected draft: what it scored and how much it left open. */
  confidence: number | null;
  open: number | null;
  /** Set on an approved plan: how much of it actually landed. */
  landed: string | null;
  age: string;
}

export const PLANS: PlanSummary[] = [
  {
    id: "pl-1",
    title: "Provenance beats recency in memory conflicts",
    project: "loom-db",
    state: "in_progress",
    step: "Converse",
    stepLine: "step 3 · 8 of 12 answered",
    confidence: null,
    open: null,
    landed: null,
    age: "12m",
  },
  {
    id: "pl-2",
    title: "Egress proxy policy tiers",
    project: "tapestry",
    state: "in_progress",
    step: "Synthesis",
    stepLine: "step 3 · 8 of 12 answered",
    confidence: null,
    open: null,
    landed: null,
    age: "41m",
  },
  {
    id: "pl-3",
    title: "Compile successful runs into recipes",
    project: "weaver",
    state: "draft_rejected",
    step: "Recommend",
    stepLine: "confidence 0.31 · open[6]",
    confidence: 0.31,
    open: 6,
    landed: null,
    age: "2d",
  },
  {
    id: "pl-4",
    title: "Marketplace trust model",
    project: "tapestry",
    state: "draft_rejected",
    step: "Recommend",
    stepLine: "deferred to M8",
    confidence: 0.44,
    open: 9,
    landed: null,
    age: "5d",
  },
  {
    id: "pl-5",
    title: "Byte-deterministic materialization",
    project: "tapestry",
    state: "approved",
    step: "Approved",
    stepLine: "8 tasks landed",
    confidence: null,
    open: null,
    landed: "8 tasks landed",
    age: "1h",
  },
  {
    id: "pl-6",
    title: "Per-run port allocation",
    project: "weaver",
    state: "approved",
    step: "Approved",
    stepLine: "3 tasks landed",
    confidence: null,
    open: null,
    landed: "3 tasks landed",
    age: "1d",
  },
];

export const SELECTED_PLAN_ID = "pl-1";

/** Who is speaking. The auditor is its own speaker because it is its own agent. */
export type Speaker = "interviewer" | "researcher" | "auditor" | "you";

export interface PlanMessage {
  id: string;
  speaker: Speaker;
  /** Mono initials on the avatar. */
  initials: string;
  caption: string;
  body: string;
  /** What the speaker went and found, as a compact fact row. */
  facts: string[];
  /** A finding line, called out under the body. */
  finding: string | null;
}

export const CONVERSATION: PlanMessage[] = [
  {
    id: "m-1",
    speaker: "interviewer",
    initials: "IN",
    caption: "interviewer · plan",
    body: "When two agents write the same memory key with contradicting values, which wins — the newer write, or the one with stronger provenance?",
    facts: [],
    finding: null,
  },
  {
    id: "m-2",
    speaker: "you",
    initials: "YOU",
    caption: "you",
    body: "Provenance. A fact confirmed by a passing verify outranks a fact an agent asserted, whatever the timestamps say.",
    facts: [],
    finding: null,
  },
  {
    id: "m-3",
    speaker: "researcher",
    initials: "RE",
    caption: "researcher · dispatched by the interviewer",
    body: "Prior art: amq resolves by last-write-wins and documents it as a known failure. memsearch has no conflict rule. Neither carries provenance, so neither can implement what you described.",
    facts: ["3 repos indexed", "4 wiki pages", "2 decisions"],
    finding: "a-7719 · not in the inbox by design",
  },
  {
    id: "m-4",
    speaker: "auditor",
    initials: "AU",
    caption: "auditor · fresh context · ISO/IEC/IEEE 29148 · two-reader test",
    body: '"Stronger provenance" is not defined for two facts that both came from passing verifies. Back to the loop, once.',
    facts: [],
    finding: "Finding — missed question",
  },
];

export interface ScopeDecision {
  question: string;
  detail: string;
  widen: string;
  keepOut: string;
}

/**
 * A scope change resolves inline. PLAN.md counts it separately from a question
 * precisely because it is not one — it is a decision the conversation makes on the
 * spot, and turning it into a gate would put it in the inbox where it does not go.
 */
export const SCOPE_DECISION: ScopeDecision = {
  question: "Scope decision — resolves inline, not as a separate gate",
  detail:
    "Provenance ranking needs a confidence column on `memory.fact`. That is a migration outside the stated goal.",
  widen: "Widen scope",
  keepOut: "Keep out, note as open",
};

export interface Recommendation {
  confidence: number;
  open: number;
  /**
   * What would raise the figure. PLAN.md is explicit that "medium; high once the
   * migration path is confirmed" is an action and a bare percentage is not.
   */
  condition: string;
  ratchet: string;
  taskCount: number;
}

export const RECOMMENDATION: Recommendation = {
  confidence: 0.62,
  open: 2,
  condition:
    "high once the provenance tie-break and the decay interaction are settled",
  ratchet: "Ratchet says one more elicitation pass before approval.",
  taskCount: 4,
};

export interface SpecOutput {
  name: string;
  lines: string[];
}

export interface DraftOutputs {
  spec: SpecOutput;
  tasks: string[];
  /** Tools already present in the image. */
  tools: string[];
  /** Tools this plan would add — drawn as outline chips, because they are new. */
  newTools: string[];
}

export const DRAFT_OUTPUTS: DraftOutputs = {
  spec: {
    name: "spec.md",
    lines: [
      "14 requirements · 3 trust boundaries · 6 error conditions",
      "Pass 2 cut 4 unsupported clauses",
    ],
  },
  tasks: [
    "confidence column on memory.fact",
    "provenance ranker + tie-break",
    "keeper: contradiction sweep",
    "locus memory explain",
  ],
  tools: ["rg+grep", "sqlx-cli", "cargo-nextest"],
  newTools: ["+ pgvector"],
};

/** The live line under the conversation: a pulsing dot and what is happening now. */
export const LIVE_LINE = "interviewer is re-opening question 14 of 14";

/** The one-approval rule, verbatim, as the list footer carries it. */
export const ONE_APPROVAL_RULE =
  "Nothing reaches the board until one approval at the end.";

/** What a new plan needs before it can start. */
export const NEW_PLAN_NOTE =
  "Starts with a goal, a target repo, and the repos involved — the goal is an input, not an output.";

/** The mono label on the conversation footer: this conversation is an ACP session. */
export const ACP_LABEL = "ACP · session/prompt";

export interface SpecRequirement {
  id: string;
  body: string;
  finding: string | null;
}

export const SPEC_REQUIREMENTS: SpecRequirement[] = [
  {
    id: "R-05",
    body: "When two facts share a key, the system MUST rank by provenance before recency. A fact confirmed by a passing verify outranks a fact an agent asserted, whatever the timestamps say.",
    finding: null,
  },
  {
    id: "R-06",
    body: "Provenance MUST be stored as a confidence value on memory.fact, written at insert time. It MUST NOT be recomputed at read time.",
    finding: null,
  },
  {
    id: "R-07",
    body: "Where two facts carry equal confidence, the tie-break MUST be the older verified_at — the fact that has survived longer wins.",
    finding: "answers auditor finding — missed question, stage 5",
  },
  {
    id: "R-08",
    body: "A losing fact MUST be retained and marked superseded, not deleted. locus memory explain MUST be able to show why the winner won.",
    finding: null,
  },
];

export type PlanGranularity = "spec" | "every-task" | "spec-carve-outs";

export interface PlanGranularityOption {
  id: PlanGranularity;
  label: string;
  detail: string;
  yield: string;
}

export const PLAN_GRANULARITY_OPTIONS: PlanGranularityOption[] = [
  {
    id: "spec",
    label: "The spec",
    detail:
      "One card for the whole plan. The agent decomposes it at run time and you watch one thing move.",
    yield: "coarsest · nothing to manage",
  },
  {
    id: "every-task",
    label: "Every task",
    detail:
      "One card per task from the spec. Full visibility, and a board you now have to tend.",
    yield: "finest · dependencies carried",
  },
  {
    id: "spec-carve-outs",
    label: "Spec + carve-outs",
    detail:
      "The spec rides as one card; the tasks you expect to be long get their own.",
    yield: "recommended for this plan",
  },
];

export interface PlanTask {
  id: string;
  title: string;
  role: string;
  estimate: string;
  dependency: string;
}

export const PLAN_TASKS: PlanTask[] = [
  {
    id: "T-01",
    title: "Confidence column on memory.fact, with a down migration",
    role: "impl",
    estimate: "~1 run",
    dependency: "—",
  },
  {
    id: "T-02",
    title: "Provenance ranker + the documented tie-break",
    role: "impl",
    estimate: "~3 runs",
    dependency: "T-01",
  },
  {
    id: "T-03",
    title: "keeper: contradiction sweep over ranked facts",
    role: "maintain",
    estimate: "~2 runs",
    dependency: "T-02",
  },
  {
    id: "T-04",
    title: "locus memory explain <key>",
    role: "impl",
    estimate: "~1 run",
    dependency: "T-02",
  },
];
