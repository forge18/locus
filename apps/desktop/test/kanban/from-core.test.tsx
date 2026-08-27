import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { COLUMN_ORDER, COLUMN_LABELS, useTasks } from "../../src/data/board";
import ManageView from "../../src/screens/manage/ManageView";

describe("kanban/from-core", () => {
  it("renders fixed columns and task cards from the board data projection", () => {
    const { getByTestId, getByText } = render(() => <ManageView />);
    expect(getByTestId("manage").getAttribute("data-view")).toBe("kanban");
    for (const column of COLUMN_ORDER) {
      const section = document.querySelector(`[data-column="${column}"]`);
      expect(section?.getAttribute("data-column-label")).toBe(
        COLUMN_LABELS[column],
      );
    }
    for (const task of useTasks())
      expect(getByTestId(`manage-task-${task.id}`)).toBeTruthy();
    expect(getByText("In Progress")).toBeTruthy();
  });
});
