import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/task-detail", () => {
  it("shows workflow, root session, run tree, evidence, and controls", () => {
    const { getByTestId } = render(() => <ManageView />);
    expect(getByTestId("automate-task-detail")).toBeTruthy();
    expect(getByTestId("automate-task-detail").textContent).toContain(
      "Workflow",
    );
    expect(getByTestId("automate-task-detail").textContent).toContain(
      "Root session",
    );
    expect(getByTestId("automate-task-detail").textContent).toContain(
      "Evidence",
    );
    expect(getByTestId("automate-task-controls")).toBeTruthy();
  });
});
