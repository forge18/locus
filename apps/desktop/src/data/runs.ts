import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

/** How many rows a page carries. One page is what a first paint has to wait for. */
export const PAGE_SIZE = 100;

/** Wire row from the `dispatch_runs_page` command: every run, newest first. */
export interface DispatchRunRow {
  id: string;
  project: string;
  agent: string;
  branch: string;
  status: string;
  /** From the agent definition's frontmatter; absent where none is declared. */
  harness: string | null;
  role: string | null;
  /** The model that actually answered — an id, never a tier. */
  model: string;
  events: number;
  errors: number;
  startedAt: string | null;
}

/** Live read: one page of runs, newest first. The host scopes by project when
 * a projectId is given; an unknown project is a typed not-found. */
export function fetchRunsPage(
  offset: number,
  limit = PAGE_SIZE,
): Promise<Envelope<DispatchRunRow[]>> {
  return dataProvider().query<DispatchRunRow>("dispatch_runs_page", {
    offset,
    limit,
  });
}

/** Live count of every run — the header's headline number. */
export function fetchRunsCount(): Promise<Envelope<number>> {
  return dataProvider().queryOne<number>("dispatch_runs_count");
}
