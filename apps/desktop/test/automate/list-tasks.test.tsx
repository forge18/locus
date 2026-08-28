import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { useTasks } from "../../src/data/board";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/list-tasks", () => {
  it("renders the same task query as a List", async () => {
    const { getByText, getByTestId } = render(() => <ManageView />);
    await fireEvent.click(getByText("List"));
    expect(
      getByTestId("automate-list-tasks").querySelectorAll(
        "[data-task-locator]",
      ),
    ).toHaveLength(useTasks().length);
  });
});
