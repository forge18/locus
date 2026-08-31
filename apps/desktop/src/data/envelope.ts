/**
 * The typed envelope every live accessor returns. A screen reads `status` and
 * renders the matching state — loading, empty, ready, failed — and never falls
 * back to a fixture on failure. See .specs/desktop-data-integration/contract.md.
 *
 * `failed` carries the Tauri command that failed so a screen (and a user report)
 * can name the boundary that broke instead of showing a generic error.
 */
export interface AccessorFailure {
 /** The registered command that rejected the call, e.g. "projects_list". */
 command: string;
 /** Human-readable failure description from the host or the IPC boundary. */
 message: string;
}

export type Envelope<T> =
 | { status: "loading" }
 | { status: "empty" }
 | { status: "ready"; data: T }
 | { status: "failed"; error: AccessorFailure };

/** Wrap a successful query result. An empty array is `empty`, not an empty `ready`. */
export function ready<T>(data: T[]): Envelope<T[]> {
 return data.length === 0 ? { status: "empty" } : { status: "ready", data };
}

/** Wrap a successful single-value query result (an absent row is `empty`). */
export function readyOne<T>(data: T | null | undefined): Envelope<T> {
 return data == null ? { status: "empty" } : { status: "ready", data };
}

/** Wrap a rejected invoke with the command that failed and any thrown value. */
export function failed(command: string, cause: unknown): Envelope<never> {
 return {
  status: "failed",
  error: {
   command,
   message: cause instanceof Error ? cause.message : String(cause),
  },
 };
}
