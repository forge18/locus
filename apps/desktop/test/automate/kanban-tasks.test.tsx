import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { COLUMN_ORDER, useTasks } from "../../src/data/board";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/kanban-tasks", () => {
  it("renders the project task query across fixed Kanban columns", () => {
    const { getByTestId } = render(() => <ManageView />);
    expect(getByTestId("automate-kanban-tasks")).toBeTruthy();
    expect(document.querySelectorAll("[data-column]").length).toBe(
      COLUMN_ORDER.length,
    );
    for (const task of useTasks())
      expect(getByTestId(`manage-task-${task.id}`)).toBeTruthy();
  });
});
