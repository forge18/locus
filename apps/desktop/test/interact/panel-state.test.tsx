import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import InteractView from "../../src/screens/interact/InteractView";

describe("interact/panel-state", () => {
  it("shows the selected session cost and permission posture", async () => {
    const view = render(() => <InteractView />);

    expect(view.getByTestId("agent-cost-toggle").textContent).toContain(
      "$0.42",
    );
    expect(
      view.getByTestId("agent-pane").getAttribute("data-permission-posture"),
    ).toBe("bypass");

    await fireEvent.click(view.getByText("Review parser behavior"));
    expect(
      view.getByTestId("agent-pane").getAttribute("data-permission-posture"),
    ).toBe("gated");
    expect(view.queryByTestId("interact-commit")).toBeNull();
  });

  it("swaps research for the changed-files rail", async () => {
    const view = render(() => <InteractView />);

    expect(view.getByText("Changed this session")).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Research" }));
    expect(view.getByText("Live research is not yet available")).toBeTruthy();
    expect(view.queryByText("Changed this session")).toBeNull();

    await fireEvent.click(view.getByRole("button", { name: "Close research" }));
    expect(view.getByText("Changed this session")).toBeTruthy();
  });
});
