import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/import-completion-status", () => {
  it("shows idempotent completion delivery state in task detail", () => {
    const { getByTestId } = render(() => <ManageView />);
    expect(
      getByTestId("automate-import-completion-status").textContent,
    ).toContain("one idempotent comment after local Done");
  });
});
