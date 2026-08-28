import { expect, it } from "vitest";
import { mount } from "./support";

it("agent-panel/header renders identity metadata and controls", () => {
  const view = mount();
  expect(view.getByTestId("agent-task-chip").textContent).toContain("task-42");
  expect(view.getByTestId("agent-workflow-chip").textContent).toContain("workflow-1");
  expect(view.getByLabelText("Session name")).toBeTruthy();
  expect(view.getByRole("button", { name: "Research" })).toBeTruthy();
  expect(view.getByRole("button", { name: "Manual" })).toBeTruthy();
});
