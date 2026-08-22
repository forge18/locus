import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { Shell } from "../../src/shell/Shell";
import { DevelopView } from "../../src/screens/develop/DevelopView";
import { createNavStore } from "../../src/nav";
import { SRC, read, rules } from "../css";

/**
 * Structural conformance against screenshots/08-develop.png — not a pixel diff.
 * jsdom has no layout engine; what is asserted is what the screenshot encodes
 * that survives without one.
 */
const SHOTS = resolve(SRC, "../../../docs/design_handoff_locus_v2/screenshots");
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;

const mount = () => {
  const nav = createNavStore({ view: "develop" });
  return render(() => (
    <Shell nav={nav}>
      <DevelopView />
    </Shell>
  ));
};

describe("visual: develop", () => {
  it("has the reference screenshot to conform to", () => {
    expect(existsSync(resolve(SHOTS, "08-develop.png"))).toBe(true);
  });

  it("is three columns at 206 / flex / 252 inside the four bands", () => {
    const { getByTestId } = mount();
    for (const part of [
      "titlebar",
      "rail",
      "tabbar",
      "strip",
      "dev-tree",
      "dev-editor",
      "git-panel",
    ]) {
      expect(getByTestId(part), part).toBeTruthy();
    }
    expect(
      (getByTestId("dev-tree") as HTMLElement).style.getPropertyValue(
        "--pane-w",
      ),
    ).toBe("206px");
    expect(
      (getByTestId("git-panel") as HTMLElement).style.getPropertyValue(
        "--pane-w",
      ),
    ).toBe("252px");
  });

  it("lights Develop on the rail and draws no tabs", () => {
    const { getByTestId } = mount();
    expect(getByTestId("rail-develop").getAttribute("aria-current")).toBe(
      "true",
    );
    expect(getByTestId("tabbar-category").textContent).toBe("Develop");
    expect(getByTestId("tabbar-tabs").querySelectorAll(".tab").length).toBe(0);
  });

  it("stacks the editor: tab strip, diff, footer", () => {
    const { getByTestId } = mount();
    const editor = getByTestId("dev-editor");
    expect(
      [...editor.children].map((c) => c.getAttribute("data-testid")),
    ).toEqual(["dev-tabs", "diff", "dev-footer"]);
  });

  it("shows the diff side by side under two headers", () => {
    const { getByTestId } = mount();
    expect(getByTestId("diff-header-left").textContent).toBe("HEAD · main");
    expect(getByTestId("diff-header-right").textContent).toBe(
      "agent/8f21-notify · builder@4",
    );
    expect(rule(".diff-body").body).toContain("grid-template-columns: 1fr 1fr");
  });

  it("stacks the git panel: header, branch, body, footer", () => {
    const { getByTestId } = mount();
    const panel = getByTestId("git-panel");
    const order = [...panel.children].map((c) => c.className.split(" ")[0]);
    expect(order.slice(0, 4)).toEqual([
      "git-head",
      "git-branch-block",
      "git-body",
      "git-foot",
    ]);
  });

  it("carries the copy the screenshot shows, verbatim", () => {
    const { getByTestId } = mount();
    expect(getByTestId("dev-tree-foot").textContent).toBe(
      "Linked repo · your own checkout at ~/Repos/tapestry",
    );
    expect(getByTestId("dev-lsp").textContent).toBe(
      "rust-analyzer · 0 errors · 2 hints",
    );
    expect(getByTestId("dev-footer-note").textContent).toBe(
      "Reviewing what an agent changed is the primary editor surface",
    );
    expect(getByTestId("git-foot-note").textContent).toBe(
      "Working tree is your own checkout — the agent pushed to the branch, you decide what lands.",
    );
    expect(getByTestId("dev-chunks").textContent).toBe("2 chunks");
  });

  it("paints every surface from a token", () => {
    expect(read("screens/screens.css")).not.toMatch(/#[0-9a-fA-F]{6}\b/);
  });
});
