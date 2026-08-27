import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/import-confirmation", () => {
  it("requires workflow selection and states the one-way completion boundary", async () => {
    const { getByText, getByTestId } = render(() => <ManageView />);
    await fireEvent.click(getByText("Import task"));
    expect(getByTestId("automate-import-workflow")).toBeTruthy();
    expect(getByTestId("automate-import-one-way").textContent).toContain(
      "No source write before local Done",
    );
    expect(getByTestId("automate-import-kanban").textContent).toContain(
      "preview before local task creation",
    );
  });
});
