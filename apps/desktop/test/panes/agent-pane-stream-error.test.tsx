import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  streamFromCore: vi.fn().mockRejectedValue(new Error("stream unavailable")),
  replayRunEvents: vi.fn().mockResolvedValue([]),
}));

vi.mock("../../src/transcript/from-core", () => ({
  streamFromCore: mocks.streamFromCore,
  replayRunEvents: mocks.replayRunEvents,
}));

import { AgentPane } from "../../src/panes/AgentPane";

describe("agent pane stream setup", () => {
  it("renders subscription failures inline", async () => {
    const pane = render(() => <AgentPane runId="run-1" />);

    await waitFor(() => {
      expect(pane.getByTestId("inline-error-cause").textContent).toBe(
        "stream unavailable",
      );
    });
  });
});
