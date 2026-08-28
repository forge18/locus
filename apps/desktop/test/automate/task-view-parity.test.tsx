import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/task-view-parity", () => {
  it("resolves Kanban and List cards to the same task locators", async () => {
    const { getByText, getByTestId } = render(() => <ManageView />);
    const kanban = [
      ...getByTestId("automate-kanban-tasks").querySelectorAll(
        "[data-task-locator]",
      ),
    ].map((node) => node.getAttribute("data-task-locator"));
    await fireEvent.click(getByText("List"));
    const list = [
      ...getByTestId("automate-list-tasks").querySelectorAll(
        "[data-task-locator]",
      ),
    ].map((node) => node.getAttribute("data-task-locator"));
    expect(list).toEqual(kanban);
  });
});
