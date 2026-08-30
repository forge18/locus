import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

/** How many rows a page carries. One page is what a first paint has to wait for. */
export const PAGE_SIZE = 100;

/** Wire type: one row of `agents.sessions` via the `sessions_list` command. */
export interface SessionRow {
  id: string;
  projectId: string;
  project: string;
  agent: string;
  name: string;
  branch: string;
  status: string;
  createdAt: string | null;
}

/** Wire type: one run of a session via the `runs_for_session` command. */
export interface SessionRun {
  id: string;
  sessionId: string;
  status: string;
  resolvedModel: string;
  startedAt: string | null;
  endedAt: string | null;
  exitCode: number | null;
}

/** Live read: every session across projects, newest first. The host scopes by
 * project when a projectId is given; an unknown project is a typed not-found. */
export function fetchSessions(
  projectId?: string,
  offset = 0,
  limit = PAGE_SIZE,
): Promise<Envelope<SessionRow[]>> {
  return dataProvider().query<SessionRow>("sessions_list", {
    projectId,
    offset,
    limit,
  });
}

/** Live read: one session's runs, oldest first. */
export function fetchRunsForSession(
  sessionId: string,
): Promise<Envelope<SessionRun[]>> {
  return dataProvider().query<SessionRun>("runs_for_session", { sessionId });
}
