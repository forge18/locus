import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/sync-status", () => {
  it("renders linked-task sync state and controls", async () => {
    const view = render(() => <ManageView />);
    await fireEvent.click(view.getByTestId("manage-task-t-010"));
    await waitFor(() =>
      expect(view.getByTestId("automate-sync-status")).toBeTruthy(),
    );
    expect(
      view.getByTestId("automate-sync-status").getAttribute("data-sync-status"),
    ).toBe("synced");
    expect(view.getByRole("button", { name: "Sync now" })).toBeTruthy();
    expect(
      view.getByRole("button", { name: "Push current status" }),
    ).toBeTruthy();
    expect(view.getByRole("textbox", { name: "External note" })).toBeTruthy();
    expect(view.getByTestId("automate-sync-conflict").textContent).toContain(
      "Last conflict: external won",
    );
  });
});
