import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { formatDesktopLocator, resolveDesktopLocator } from "../../src/nav/desktop-locator";
import { Shell } from "../../src/shell/Shell";

describe("nav/desktop-locator", () => {
  it("formats and resolves global and project scoped desktop locators", () => {
    expect(formatDesktopLocator("inbox")).toBe("locus://global/inbox");
    expect(resolveDesktopLocator("locus://project/locus/develop")).toEqual({
      route: "develop",
      scope: { kind: "project", project: "locus" },
    });
  });

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
