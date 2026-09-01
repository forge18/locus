import { render, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import BotsView from "../../src/screens/bots/BotsView";
import { ready, type Envelope } from "../../src/data/envelope";
import {
  configureDataProvider,
  type DataProvider,
} from "../../src/data/provider";
import { configureDemoProvider } from "../../src/data/demo/bootstrap";
import type { Bot } from "../../src/data/bots";

const bot: Bot = {
  id: "live-bot",
  projectId: "project-1",
  name: "Live bot",
  agentDefId: "agent-1",
  homeSessionId: "home-1",
  activeRunId: null,
  branch: "bots/live-bot",
  containerId: null,
  containerState: "cold",
  warmUntil: null,
  lastActivityAt: null,
  totalCostMicros: null,
};

function liveProvider(query: (command: string) => void): DataProvider {
  return {
    kind: "live",
    query: async <T,>(command: string) => {
      query(command);
      return command === "bots_list" ? ready([bot] as T[]) : ready([] as T[]);
    },
    queryOne: async <T,>(): Promise<Envelope<T>> => ({ status: "empty" }),
  };
}

afterEach(() => {
  configureDemoProvider();
});

describe("bots/live-provider", () => {
  it("loads bot rows through the live provider without fixture rows", async () => {
    const query = vi.fn<(command: string) => void>();
    configureDataProvider(liveProvider(query));
    const view = render(() => <BotsView projectId="project-1" />);

    await waitFor(() =>
      expect(view.getByTestId("bot-row-live-bot")).toBeTruthy(),
    );
    expect(query).toHaveBeenCalledWith("bots_list");
    expect(view.queryByText("Keeper")).toBeNull();
    expect(view.queryByText(/Demo data/)).toBeNull();
  });
});
