import type {
 ProjectLocalRemote,
 ProjectRepo,
 ProjectSetup,
 ProjectSummary,
} from "../types/core";
import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

export type CapabilityPolicy = "defer_to_project" | { allow_only: string[] };

export interface CapabilityPolicies {
 cliTools: CapabilityPolicy;
 commands: CapabilityPolicy;
 skills: CapabilityPolicy;
}

export interface ProjectCapabilityPolicy {
 revision: number;
 policies: CapabilityPolicies;
}

/**
 * Live accessors for the Setup tracer bullet (desktop-data-integration slice 3).
 * Every read crosses the provider seam and returns a typed `Envelope` — never a
 * fixture, never a silent empty. See .specs/desktop-data-integration/contract.md.
 */

/** Every project, alphabetically — the Setup screen's project list. */
export function fetchProjects(): Promise<Envelope<ProjectSummary[]>> {
 return dataProvider().query<ProjectSummary>("projects_list");
}

/** Every repo of one project. The host scopes the query; a repo of another
 * project can never appear here. */
export function fetchRepos(
 projectId: string,
): Promise<Envelope<ProjectRepo[]>> {
 return dataProvider().query<ProjectRepo>("repos_list", { projectId });
}

/** The bare remotes of one project's repos, joined through `core.repos`. */
export function fetchLocalRemotes(
 projectId: string,
): Promise<Envelope<ProjectLocalRemote[]>> {
 return dataProvider().query<ProjectLocalRemote>("local_remotes_list", {
  projectId,
 });
}

/** One project's harness policy and base context. */
export function fetchProjectSetup(
 projectId: string,
): Promise<Envelope<ProjectSetup>> {
 return dataProvider().queryOne<ProjectSetup>("project_setup", { projectId });
}

export function fetchProjectCapabilityPolicy(
 projectId: string,
): Promise<Envelope<ProjectCapabilityPolicy>> {
 return dataProvider().queryOne<ProjectCapabilityPolicy>(
  "project_capability_policy",
  { projectId },
 );
}

export function saveProjectCapabilityPolicy(
 projectId: string,
 policies: CapabilityPolicies,
): Promise<Envelope<ProjectCapabilityPolicy>> {
 return dataProvider().queryOne<ProjectCapabilityPolicy>(
  "project_capability_policy_set",
  { projectId, policies },
 );
}

// Mutations (slice 5). Each returns the refreshed read shape so the screen can
// update its envelope from the response instead of refetching.

/** Replace the project's base context. Empty content clears it and its budget —
 * the domain rule keeps the two together. */
export function saveBaseContext(
 projectId: string,
 content: string,
 tokenBudget: number | undefined,
): Promise<Envelope<ProjectSetup>> {
 return dataProvider().queryOne<ProjectSetup>("project_base_context_set", {
  projectId,
  content,
  tokenBudget,
 });
}

/** Archive or restore a project. */
export function setProjectArchived(
 projectId: string,
 archived: boolean,
): Promise<Envelope<{ archived: boolean }>> {
 return dataProvider().queryOne<{ archived: boolean }>("project_archive_set", {
  projectId,
  archived,
 });
}

/** Rename a project; the response carries the new name. */
export function renameProject(
 projectId: string,
 name: string,
): Promise<Envelope<{ id: string; name: string }>> {
 return dataProvider().queryOne<{ id: string; name: string }>(
  "project_rename",
  { projectId, name },
 );
}
