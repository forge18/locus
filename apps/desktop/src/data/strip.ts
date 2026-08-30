import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

export type StripKind = "agent" | "shell";

/** Wire row from the `strip_cards` command: one running run, live from the store. */
export interface StripCardRow {
  id: string;
  project: string;
  agent: string;
  status: string;
  /** Seconds since the run started, so the UI derives elapsed time locally. */
  startedEpoch: number;
}

/**
 * A strip card is one running agent. The wire row carries what the store knows:
 * the run's project, agent, and status — role, tool, and token attribution land
 * with the run-supervisor slice, and are honestly absent until then.
 */
export interface StripCard {
  id: string;
  kind: StripKind;
  project: string;
  /** Null for a shell card — a terminal you drive yourself is not an agent. */
  agent: string | null;
  status: string | null;
  idleMinutes: number;
  /** Attribution fields arrive with the run-supervisor slice; absent until then.
   * Null means the harness reports nothing — unknown, never zero. */
  taskId?: string;
  role?: string | null;
  tool?: string | null;
  tokens?: number | null;
}

/** How badly a card wants a person. Higher goes first. */
function attention(card: StripCard): number {
  if (card.status === "stuck") return 3;
  if (card.status === "waiting") return 2;
  if (card.status === "idle") return 1;
  return 0;
}

/**
 * Live read: every running run across projects, sorted needs-attention first,
 * then activity. Never by project and never alphabetically — either would put
 * the same session in the same place whether or not anything was happening.
 */
export async function fetchStripCards(): Promise<Envelope<StripCard[]>> {
  const envelope = await dataProvider().query<StripCardRow>("strip_cards");
  if (envelope.status !== "ready") return envelope;
  const nowSeconds = Date.now() / 1000;
  const cards = envelope.data.map((row) => ({
    id: row.id,
    kind: "agent" as const,
    project: row.project,
    agent: row.agent,
    status: row.status,
    idleMinutes: Math.max(0, Math.floor((nowSeconds - row.startedEpoch) / 60)),
  }));
  return {
    status: "ready",
    data: cards.sort(
      (a, b) => attention(b) - attention(a) || a.idleMinutes - b.idleMinutes,
    ),
  };
}

/** Live count of running agents — the dispatch pill's headline number. */
export async function fetchRunningCount(): Promise<Envelope<number>> {
  return dataProvider().queryOne<number>("running_count");
}
