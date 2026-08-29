// Mirrors the `core` Postgres schema (PLAN.md §Data model): projects, repos
// (multi-repo per project), local remotes, settings.

/** @schema core — a project is the unit everything else hangs off. */
export interface Project {
 id: string;
 name: string;
 /** Repos belonging to this project, in the order they were added. */
 repoIds: string[];
 createdAt: string;
}

/** @schema core — one repository inside a project. A project may hold several. */
export interface Repo {
 id: string;
 projectId: string;
 name: string;
 /** Where your own checkout lives. Agents never see it. */
 localPath: string;
 defaultBranch: string;
 /** The bare local remote agents clone from and push to. */
 localRemoteId: string;
}

/**
 * @schema core — the bare repository agents clone from. The workspace is a clone,
 * never a mount, so this is the only thing an agent container can reach.
 */
export interface LocalRemote {
 id: string;
 repoId: string;
 path: string;
 /** Branches pushed by agents, newest first. `main` is never among them. */
 branches: string[];
}

/** @schema core — a tier's model on one harness. Which model `high` means is policy. */
export type ModelTier = "low" | "medium" | "high" | "xhigh";

/**
 * @schema core — settings, keyed by harness and tier. An unset tier means the
 * harness's own default: Locus passes no model flag at all.
 */
export interface ModelTierSetting {
 harness: string;
 tier: ModelTier;
 /** The resolved model id, or null for "leave it to the harness". */
 model: string | null;
}

/** @schema core — everything in Settings that is not a model mapping. */
export interface Settings {
 modelTiers: ModelTierSetting[];
 /** The project filter in the title bar. Null means all projects. */
 activeProjectId: string | null;
 /** Install-wide derived bot style; a missing value resolves to Bottts. */
 "bots.avatar_style"?: string;
}
