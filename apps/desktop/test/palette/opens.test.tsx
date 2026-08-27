import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { Shell } from "../../src/shell/Shell";

describe("palette/opens", () => {
  it("opens the locator palette with Cmd-K", () => {
    render(() => (
      <Shell nav={createNavStore()}>
        <div />
      </Shell>
    ));
    fireEvent.keyDown(document, { key: "k", metaKey: true });
    expect(
      document.querySelector('[data-testid="locator-palette-input"]'),
    ).toBeTruthy();
  });
});
