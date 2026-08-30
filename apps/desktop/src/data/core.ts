import type {
 ProjectLocalRemote,
 ProjectRepo,
 ProjectSetup,
 ProjectSummary,
} from "../types/core";
import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

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
