import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { LocatorPalette } from "../../src/nav/LocatorPalette";
import { resolve } from "../../src/nav/locator";

describe("palette/resolves-all-kinds", () => {
  it("opens the highlighted result with the arrow and enter keys", () => {
    const opened: string[] = [];
    const view = render(() => (
      <LocatorPalette
        open
        current="locus://tapestry/view/plan"
        onOpenChange={() => undefined}
        onResolve={(target) => opened.push(target.route)}
      />
    ));
    const input = view.getByTestId("locator-palette-input");
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(opened).toEqual(["inbox"]);
  });

  it("delegates every object locator to the desktop resolver", () => {
    const resolved: string[] = [];
    render(() => (
      <LocatorPalette
        open
        current="locus://tapestry/view/inbox"
        onOpenChange={() => undefined}
        onResolve={(target) => resolved.push(target.route)}
      />
    ));
    for (const locator of [
      "locus://all/view/inbox",
      "locus://tapestry/task/t-1",
      "locus://tapestry/artifact/a-1",
      "locus://tapestry/page/p-1",
      "locus://tapestry/workflow/wf-1/execution/ex-1",
      "locus://tapestry/agent/builder@4",
    ]) {
      expect(() => resolve(locator)).not.toThrow();
    }
    expect(resolved).toEqual([]);
  });
});
