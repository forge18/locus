import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import BotsView from "../../src/screens/bots/BotsView";

beforeEach(() => {
  document.body.innerHTML = "";
  const root = document.createElement("div");
  root.id = "root";
  document.body.appendChild(root);
});

describe("bots/routines-sheet", () => {
  it("keeps routine controls in a sheet over the bot view", async () => {
    const view = render(() => <BotsView />, {
      container: document.getElementById("root")!,
    });
    await fireEvent.click(view.getByTestId("open-routines"));
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="bots-routines-sheet"]'),
      ).toBeTruthy(),
    );
    expect(
      document.querySelector('[data-testid="bot-routine-routine-health"]'),
    ).toBeTruthy();
    expect(view.getByTestId("bot-home-pane")).toBeTruthy();
  });

  it("pauses, edits, deletes, and test-runs a routine without replacing the panel", async () => {
    const view = render(() => <BotsView />, {
      container: document.getElementById("root")!,
    });
    await fireEvent.click(view.getByTestId("open-routines"));
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="bots-routines-sheet"]'),
      ).toBeTruthy(),
    );
    await fireEvent.click(view.getByRole("button", { name: "Pause" }));
    await waitFor(() =>
      expect(
        document
          .querySelector('[data-testid="bot-routine-routine-health"]')
          ?.getAttribute("data-enabled"),
      ).toBe("false"),
    );
    expect(view.getByRole("button", { name: "Enable" })).toBeTruthy();

    await fireEvent.click(view.getByRole("button", { name: "Edit" }));
    const prompt = view.getByRole("textbox", { name: "Routine prompt" });
    await fireEvent.input(prompt, { target: { value: "updated prompt" } });
    await fireEvent.click(view.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="bot-routine-routine-health"]')
          ?.textContent,
      ).toContain("updated prompt"),
    );

    await fireEvent.click(view.getByRole("button", { name: "Test run" }));
    expect(
      view.getByTestId("bot-routine-test-result").getAttribute("data-test-run"),
    ).toBe("true");

    await fireEvent.click(view.getByRole("button", { name: "Delete" }));
    expect(view.queryByTestId("bot-routine-routine-health")).toBeNull();
    expect(view.getByTestId("bot-home-pane")).toBeTruthy();
  });
});
