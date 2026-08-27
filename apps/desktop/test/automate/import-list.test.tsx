import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/import-list", () => {
  it("opens the same provider preview from List", async () => {
    const { getByText, getByTestId } = render(() => <ManageView />);
    await fireEvent.click(getByText("List"));
    await fireEvent.click(getByText("Import task"));
    expect(
      getByTestId("automate-import-list").getAttribute("data-import-contract"),
    ).toBe("external-work-item");
    expect(getByTestId("automate-import-preview")).toBeTruthy();
  });
});
