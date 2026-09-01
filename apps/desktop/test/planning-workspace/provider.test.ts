import { afterEach, describe, expect, it } from "vitest";
import { configureDataProvider, resetDataProvider } from "../../src/data/provider";
import {
  approvePlanningWorkspace,
  createPlanningWorkspace,
  deletePlanningWorkspace,
  listPlanningWorkspaceRevisions,
  listPlanningWorkspaceSessions,
  listPlanningWorkspaceSpecs,
  listPlanningWorkspaceTaskProvenance,
  listPlanningWorkspaces,
  linkPlanningWorkspaceSession,
  recordPlanningWorkspaceDecision,
  savePlanningWorkspaceCheckpoint,
  savePlanningWorkspaceSpec,
} from "../../src/data/planning-workspace";
import type { Envelope } from "../../src/data/envelope";

afterEach(resetDataProvider);

describe("planning-workspace/provider", () => {
  it("routes every workspace operation through its named live command", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    configureDataProvider({
      kind: "live",
      async query<T>(
        command: string,
        args?: Record<string, unknown>,
      ) {
        calls.push({ command, args });
        return { status: "empty" } as Envelope<T[]>;
      },
      async queryOne<T>(
        command: string,
        args?: Record<string, unknown>,
      ) {
        calls.push({ command, args });
        return { status: "empty" } as Envelope<T>;
      },
    });

    await listPlanningWorkspaces();
    await createPlanningWorkspace("p-1", "feature", "brief");
    await listPlanningWorkspaceRevisions("p-1", "w-1");
    await listPlanningWorkspaceSpecs("p-1", "w-1");
    await savePlanningWorkspaceSpec("p-1", "w-1", "r-1", "spec", {}, false);
    await listPlanningWorkspaceSessions("p-1", "w-1");
    await linkPlanningWorkspaceSession("p-1", "w-1", "s-1");
    await recordPlanningWorkspaceDecision("p-1", "w-1", ["r-1"], { key: "value" });
    await savePlanningWorkspaceCheckpoint("p-1", "w-1", 1, "in_progress", {});
    await approvePlanningWorkspace("p-1", "w-1", 2);
    await listPlanningWorkspaceTaskProvenance("p-1", "w-1");
    await deletePlanningWorkspace("p-1", "w-1");

    expect(calls.map(({ command }) => command)).toEqual([
      "planning_workspaces_list",
      "planning_workspace_create",
      "planning_workspace_revisions_list",
      "planning_workspace_specs_list",
      "planning_workspace_spec_save",
      "planning_workspace_sessions_list",
      "planning_workspace_session_link",
      "planning_workspace_decision_record",
      "planning_workspace_checkpoint_save",
      "planning_workspace_approve",
      "planning_workspace_task_provenance_list",
      "planning_workspace_delete",
    ]);
  });
});
