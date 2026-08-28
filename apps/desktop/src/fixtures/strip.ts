// schema: agents.sessions + agents.runs (a shell pane is neither)
// replaced by: invoke("strip_cards") + emit("session_status_changed")

/**
 * A strip card is one running agent, or one terminal you drive yourself. The
 * second is not a session — no agent, no events, no cost attribution — which is
 * why it is a kind here rather than a status.
 */
export type StripKind = "agent" | "shell";

export interface StripCard {
 id: string;
 /** Owning task; shell cards intentionally have none. */
 taskId?: string;
 kind: StripKind;
 project: string;
 /** `agent@version`, or null for your own shell. */
 agent: string | null;
 role: string | null;
 /** running · waiting · idle · stuck, or null for a shell. */
 status: "running" | "waiting" | "idle" | "stuck" | null;
 /** The tool the agent is in right now. */
 tool: string | null;
 /** Null where the harness reports no usage — unknown, not zero. */
 tokens: number | null;
 /** Minutes since the last event; drives the activity half of the ordering. */
 idleMinutes: number;
}

/**
 * Deliberately authored so needs-attention order and activity order disagree:
 * the stuck card is the *least* recently active, so a sort that only looked at
 * activity would bury the one thing that needs a person.
 */
export const STRIP_CARDS: StripCard[] = [
 {
  id: "st-1",
  taskId: "t-004",
  kind: "agent",
  project: "tapestry",
  agent: "builder@4",
  role: "builder",
  status: "running",
  tool: "edit_file",
  tokens: 41_200,
  idleMinutes: 0,
 },
 {
  id: "st-2",
  kind: "agent",
  project: "loom-db",
  agent: "builder@4",
  role: "builder",
  status: "idle",
  tool: null,
  tokens: 18_940,
  idleMinutes: 3,
 },
 {
  id: "st-3",
  taskId: "t-005",
  kind: "agent",
  project: "weaver",
  agent: "builder@4",
  role: "builder",
  status: "stuck",
  tool: "run_command",
  tokens: 102_300,
  idleMinutes: 14,
 },
 {
  id: "st-4",
  kind: "agent",
  project: "texere",
  agent: "builder@3",
  role: "builder",
  status: "waiting",
  tool: null,
  tokens: null,
  idleMinutes: 6,
 },
 {
  id: "st-5",
  kind: "shell",
  project: "tapestry",
  agent: null,
  role: null,
  status: null,
  tool: null,
  tokens: null,
  idleMinutes: 1,
 },
 {
  id: "st-6",
  taskId: "t-007",
  kind: "agent",
  project: "tapestry",
  agent: "reviewer@2",
  role: "reviewer",
  status: "running",
  tool: "read_file",
  tokens: 7_410,
  idleMinutes: 2,
 },
];
