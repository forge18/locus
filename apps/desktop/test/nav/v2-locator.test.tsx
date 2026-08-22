import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { formatV2Locator, resolveV2Locator } from "../../src/nav/v2-locator";
import { Shell } from "../../src/shell/Shell";

describe("nav/v2-locator", () => {
  it("formats and resolves global and project scoped v2 locators", () => {
    expect(formatV2Locator("inbox")).toBe("locus://global/inbox");
    expect(resolveV2Locator("locus://project/locus/develop")).toEqual({
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
