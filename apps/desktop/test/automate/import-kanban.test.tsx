import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { GITHUB_WORK_ITEM_PROVIDER_FIXTURE } from "../../src/data/work-items";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/import-kanban", () => {
  it("opens provider selection and an external item preview from Kanban", async () => {
    const { getByText, getByTestId } = render(() => (
      <ManageView workItemProviders={GITHUB_WORK_ITEM_PROVIDER_FIXTURE} />
    ));
    await fireEvent.click(getByText("Import task"));
    expect(getByTestId("automate-import-kanban")).toBeTruthy();
    expect(getByTestId("automate-import-providers").textContent).toContain(
      "GitHub",
    );
    expect(getByTestId("automate-import-preview").textContent).toContain(
      "GitHub",
    );
  });
});
