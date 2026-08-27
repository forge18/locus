import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import ManageView from "../../src/screens/manage/ManageView";

describe("kanban/ci-status", () => {
  it("shows normalized CI state on the owning task card", () => {
    const { getByTestId } = render(() => <ManageView />);
    expect(getByTestId("kanban-ci-t-004").textContent).toContain("CI · passed");
    expect(getByTestId("kanban-ci-t-005").textContent).toContain("CI · failed");
  });
});
