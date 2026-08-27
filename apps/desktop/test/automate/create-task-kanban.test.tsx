import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/create-task-kanban", () => {
  it("opens the shared manual task draft from Kanban", async () => {
    const { getByText, getByTestId } = render(() => <ManageView />);
    await fireEvent.click(getByText("Add task"));
    expect(
      getByTestId("automate-create-task-kanban").getAttribute(
        "data-draft-contract",
      ),
    ).toBe("manual-task");
  });
});
