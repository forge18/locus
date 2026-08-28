import { expect, it } from "vitest";
import { mount } from "./support";

it("agent-panel/layout keeps the stream flexible and the composer present", () => {
  const view = mount();
  expect(view.getByTestId("agent-pane").className).toBe("agent-pane");
  expect(view.getByTestId("agent-stream").parentElement?.className).toBe("agent-stream-shell");
  expect(view.getByTestId("agent-composer")).toBeTruthy();
});
