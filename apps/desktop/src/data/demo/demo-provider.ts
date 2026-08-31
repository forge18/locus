/**
 * The explicit demo/test provider. It answers only the commands whose fixtures a
 * slice has explicitly registered here — never imported by production code, never
 * selected by a Tauri runtime (the production bootstrap configures `liveProvider`).
 * See .specs/desktop-data-integration/contract.md.
 *
 * The registry grows per slice: task 3 seeds the `core` family for the tracer
 * bullet, later slices add their own families, and task 10 moves the remaining
 * fixture reads behind this seam.
 */
import { LOCAL_REMOTES, PROJECTS, REPOS } from "../../fixtures/core";
import { ARTIFACTS, ARTIFACT_COMMENTS } from "../../fixtures/artifacts";
import {
 AT_A_GLANCE_METRICS,
} from "../../fixtures/analytics";
import { METRICS } from "../../fixtures/telemetry";
import { QA_FINDINGS } from "../../fixtures/qa";
import { PLANS } from "../../fixtures/plan";
import { LONG_TERM_FACTS } from "../../fixtures/knowledge";
import type { Envelope } from "../envelope";
import { failed, ready } from "../envelope";
import type { DataProvider } from "../provider";

type FixtureQuery = (args?: Record<string, unknown>) => readonly unknown[];

const FIXTURES: Record<string, FixtureQuery> = {
 projects_list: () => PROJECTS,
 repos_list: (args) => {
  const projectId = args?.projectId;
  return typeof projectId === "string"
   ? REPOS.filter((repo) => repo.projectId === projectId)
   : REPOS;
 },
 local_remotes_list: () => LOCAL_REMOTES,
 plans_list: () => PLANS,
 memory_facts: () => LONG_TERM_FACTS,
 analytics_at_a_glance: () => AT_A_GLANCE_METRICS,
 telemetry_metrics: () => METRICS,
 qa_snapshot: (args) =>
  QA_FINDINGS.filter((finding) => finding.project === args?.projectId),
 artifacts_list: () => ARTIFACTS,
 artifact_comments: (args) =>
  ARTIFACT_COMMENTS.filter((comment) => comment.artifactId === args?.artifactId),
};

function demoEnvelope<T>(
 command: string,
 args: Record<string, unknown> | undefined,
): Envelope<T[]> {
 const fixture = FIXTURES[command];
 if (!fixture) {
  return failed(command, `demo provider has no fixture for ${command}`);
 }
 return ready(fixture(args) as T[]);
}

export const demoProvider: DataProvider = {
 kind: "demo",
 async query<T>(command: string, args?: Record<string, unknown>) {
  return demoEnvelope<T>(command, args);
 },
 async queryOne<T>(
  command: string,
  _args?: Record<string, unknown>,
 ): Promise<Envelope<T>> {
  // Single-value demo fixtures arrive with the slice that needs them; the honest
  // answer until then is a typed failure, never a fabricated row.
  return failed(
   command,
   `demo provider has no single-value fixture for ${command}`,
  );
 },
};
