import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/create-task-list", () => {
  it("opens the same manual task draft from List", async () => {
    const { getByText, getByTestId } = render(() => <ManageView />);
    await fireEvent.click(getByText("List"));
    await fireEvent.click(getByText("Add task"));
    expect(
      getByTestId("automate-create-task-list").getAttribute(
        "data-draft-contract",
      ),
    ).toBe("manual-task");
  });
});
