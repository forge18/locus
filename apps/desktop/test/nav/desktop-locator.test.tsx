import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import {
  formatDesktopLocator,
  resolveDesktopLocator,
} from "../../src/nav/desktop-locator";
import { Shell } from "../../src/shell/Shell";

describe("nav/desktop-locator", () => {
  it("formats and resolves global and project scoped desktop locators", () => {
    expect(formatDesktopLocator("inbox")).toBe("locus://all/view/inbox");
    expect(resolveDesktopLocator("locus://locus/view/plan")).toEqual({
      route: "plan",
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
      document.querySelector(
        '[data-testid="locator-palette-input"][data-mode="locator"]',
      ),
    ).toBeTruthy();
  });

  it("opens Cmd-P on the unified search path", () => {
    render(() => (
      <Shell
        nav={createNavStore()}
        searchAll={(query) => [
          {
            kind: "wiki",
            project: "locus",
            label: query,
            locator: "locus://locus/page/overview",
            score: 3,
          },
        ]}
      >
        <div />
      </Shell>
    ));
    fireEvent.keyDown(document, { key: "p", metaKey: true });
    const input = document.querySelector(
      '[data-testid="locator-palette-input"][data-mode="search"]',
    ) as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.placeholder).toContain("code, wiki, tasks, and runs");
    fireEvent.input(input, { target: { value: "overview" } });
    expect(
      document.querySelector("[data-testid=palette-results]")?.textContent ??
        "",
    ).toContain("locus");
  });
});
