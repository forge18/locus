import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/create-task-parity", () => {
  it("uses one draft contract for both entry points", async () => {
    const kanban = render(() => <ManageView />);
    await fireEvent.click(kanban.getByText("Add task"));
    const kanbanDraft = kanban.getByTestId("automate-create-task-kanban");
    const list = render(() => <ManageView />);
    await fireEvent.click(list.getByText("List"));
    await fireEvent.click(list.getByText("Add task"));
    const listDraft = list.getByTestId("automate-create-task-list");
    expect(kanbanDraft.getAttribute("data-draft-contract")).toBe(
      listDraft.getAttribute("data-draft-contract"),
    );
    expect(
      kanbanDraft.querySelector("input")?.getAttribute("placeholder"),
    ).toBe(listDraft.querySelector("input")?.getAttribute("placeholder"));
  });
});
