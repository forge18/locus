import { render, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import InteractView from "../../src/screens/interact/InteractView";
import { ready, readyOne } from "../../src/data/envelope";
import {
  configureDataProvider,
  type DataProvider,
} from "../../src/data/provider";
import { configureDemoProvider } from "../../src/data/demo/bootstrap";
import type { InteractSessionRow } from "../../src/data/interact";

const row: InteractSessionRow = {
  id: "live-session",
  projectId: "project-1",
  project: "project-1",
  name: "Live session",
  agent: "builder@1",
  harness: "pi",
  branch: "interact/live-session",
  status: "active",
  state: "open",
  boardTaskId: null,
  runId: null,
  runStatus: null,
  model: "model-1",
  permissionPosture: "gated",
  createdAt: "2026-09-01T00:00:00Z",
  repo: "repo",
  baseCommit: "abc1234",
  changedFiles: [],
  cost: null,
};

function liveProvider(query: (command: string) => void): DataProvider {
  return {
    kind: "live",
    query: async <T,>(command: string) => {
      query(command);
      return command === "interact_sessions_list"
        ? ready([row] as T[])
        : ready([] as T[]);
    },
    queryOne: async <T,>() => readyOne<T>(null),
  };
}

afterEach(() => {
  configureDemoProvider();
});

describe("interact/live-provider", () => {
  it("loads sessions through the live provider instead of screen fixtures", async () => {
    const query = vi.fn();
    configureDataProvider(liveProvider(query));
    const view = render(() => <InteractView projectId="project-1" />);

    await waitFor(() =>
      expect(view.getByTestId("interact-sessions-rail").textContent).toContain(
        "Live session",
      ),
    );
    expect(query).toHaveBeenCalledWith("interact_sessions_list");
    expect(view.queryByText("Try the notification path")).toBeNull();
  });
});
