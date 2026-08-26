// Mirrors the `agents` Postgres schema (PLAN.md §Data model): agent_defs
// (versioned), sessions, runs, parent/child run edges, normalized events, and
// artifacts with their comment threads. Events live in `./event`.

import type { Usage } from "./event";

/**
 * @schema agents — a versioned agent definition: frontmatter as JSONB, Markdown
 * body. Authored once here and materialized into whatever layout each harness reads.
 */
export interface AgentDef {
 id: string;
 name: string;
 version: number;
 /** Frontmatter, verbatim. */
 frontmatter: Record<string, unknown>;
 body: string;
 /** Tools this agent may reach, by name. */
 tools: string[];
 /** Which tier it asks for; the harness setting decides what that resolves to. */
 modelTier: "low" | "medium" | "high" | "xhigh";
 readOnly: boolean;
}

/** @schema agents — how a session ended up where it is. */
export type SessionStatus = "running" | "waiting" | "idle" | "stuck" | "done";

/**
 * @schema agents — a durable, named thread of work with ONE agent. The session is
 * what survives a loop reset; the run is the thing that resets.
 */
export interface Session {
 id: string;
 projectId: string;
 repoId: string;
 /** `agent@version`, as it is written everywhere in the UI. */
 agent: string;
 role: string;
 status: SessionStatus;
 /** The session's own branch on the local remote. Never `main`. */
 branch: string;
 /** The board task this session serves, if any. */
 taskId: string | null;
 /** Set when this session inherited the work from another. */
 handedOffFrom: string | null;
 runIds: string[];
 startedAt: string;
 lastEventAt: string;
 /**
  * The sum of its runs, or null when no run reported usage. Null is *unknown*,
  * not zero.
  */
 usage: Usage | null;
}

/** @schema agents — how a run finished, or that it has not. */
export type RunStatus = "running" | "passed" | "failed" | "aborted";
export type PermissionPosture = "bypass" | "gated";

/**
 * @schema agents — one container lifetime, which is one terminal. A run is over
 * when it is over; resuming means starting another run in the same session.
 */
export interface Run {
 id: string;
 sessionId: string;
 status: RunStatus;
 startedAt: string;
 endedAt: string | null;
 /** The actual model id that answered, not the tier that was asked for. */
 resolvedModel: string;
 /** Immutable per-job dispatch choice; bypass is the default. */
 permissionPosture: PermissionPosture;
 exitCode: number | null;
 /** Reported by the harness, or null where it reported nothing. */
 usage: Usage | null;
 artifactIds: string[];
}

/** @schema agents — a parent/child run edge, for subagent work. */
export interface RunEdge {
 parentRunId: string;
 childRunId: string;
}

/** @schema agents — what an artifact is, which decides how it is reviewed. */
export type ArtifactKind =
 | "diff"
 | "plan"
 | "diagram"
 | "image"
 | "recording"
 | "walkthrough"
 | "finding"
 | "payload"
 | "report"
 | "log"
 | "video"
 | "handoff";

/**
 * @schema agents — what you review instead of tool calls. Carries a text body or a
 * blob path, never both, plus the derived representation the agent reads.
 */
export interface Artifact {
 id: string;
 runId: string;
 kind: ArtifactKind;
 title: string;
 /** Set for text artifacts. */
 body: string | null;
 /** Set for blob artifacts (images, video). */
 blobPath: string | null;
 mediaType: string;
 sha256: string;
 /**
  * The representation an agent reads, derived from the body. Text is cheaper than
  * pixels, so this is what gets handed over — never the blob.
  */
 derivedText: string | null;
 /** Null while the desktop uses its in-memory fixture seam. */
 createdAt: string | null;
}

/** @schema agents — a comment on an artifact. Threads hang off `parentId`. */
export interface ArtifactComment {
 id: string;
 artifactId: string;
 parentId: string | null;
 author: string;
 body: string;
 /** Null while the desktop uses its in-memory fixture seam. */
 createdAt: string | null;
}
