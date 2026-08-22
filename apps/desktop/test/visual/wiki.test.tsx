import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { Shell } from "../../src/shell/Shell";
import { WikiView } from "../../src/screens/wiki/WikiView";
import { createNavStore } from "../../src/nav";
import { SRC, read, rules } from "../css";

/**
 * Structural conformance against screenshots/17-memory-wiki.png — not a pixel diff.
 * jsdom has no layout engine; what is asserted is what the screenshot encodes
 * that survives without one.
 */
const SHOTS = resolve(SRC, "../../../docs/design_handoff_locus_v2/screenshots");
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;

const mount = () => {
  const nav = createNavStore({ view: "wiki" });
  return render(() => (
    <Shell nav={nav}>
      <WikiView nav={nav} />
    </Shell>
  ));
};

describe("visual: wiki", () => {
  it("has the reference screenshot to conform to", () => {
    expect(existsSync(resolve(SHOTS, "17-memory-wiki.png"))).toBe(true);
  });

  it("is three panes around 246 / flex / 284 inside the four bands", () => {
    const { getByTestId } = mount();
    for (const part of [
      "titlebar",
      "rail",
      "tabbar",
      "strip",
      "wiki-tree",
      "wiki-article",
      "wiki-side",
    ]) {
      expect(getByTestId(part), part).toBeTruthy();
    }
    expect(rule(".wiki-tree").body).toContain("clamp(200px, 19%, 300px)");
    expect(rule(".wiki-side").body).toContain("clamp(230px, 22%, 340px)");
  });

  it("lights Wiki on the rail and draws no tabs", () => {
    const { getByTestId } = mount();
    expect(getByTestId("rail-wiki").getAttribute("aria-current")).toBe("true");
    expect(getByTestId("tabbar-category").textContent).toBe("Wiki");
    expect(getByTestId("tabbar-tabs").querySelectorAll(".tab").length).toBe(0);
  });

  it("stacks the tree: ingest, note, then the six typed groups", () => {
    const { getByTestId } = mount();
    const tree = getByTestId("wiki-tree");
    expect(tree.children[0]).toBe(getByTestId("wiki-ingest"));
    expect(tree.children[1]).toBe(getByTestId("wiki-ingest-note"));
    expect(tree.querySelectorAll(".wiki-group").length).toBe(6);
  });

  it("stacks the article: tag and title, metadata, prose, links out, provenance", () => {
    const { getByTestId } = mount();
    const article = getByTestId("wiki-article");
    const order = [...article.children].map(
      (c) => c.className || c.getAttribute("data-testid"),
    );
    expect(order[0]).toContain("wiki-article-head");
    expect(order[1]).toContain("wiki-article-meta");
    expect(order[2]).toContain("wiki-prose");
    expect(article.textContent).toContain("Links out");
    expect(article.textContent).toContain("Provenance");
  });

  it("stacks the sidebar: graph, contradictions, lint, then the footer", () => {
    const { getByTestId } = mount();
    const side = getByTestId("wiki-side");
    const titles = [...side.querySelectorAll(".wiki-side-title")].map((t) =>
      t.textContent?.split("flagged")[0].trim(),
    );
    expect(titles).toEqual(["Graph", "Contradictions", "Locus wiki lint"]);
    expect(side.children[side.children.length - 1]).toBe(
      getByTestId("wiki-footer"),
    );
  });

  it("carries the copy the screenshot shows, verbatim", () => {
    const { getByTestId } = mount();
    expect(getByTestId("wiki-ingest-note").textContent).toBe(
      "Derived, then curated — a path or a URL, not a blank page.",
    );
    expect(getByTestId("wiki-graph-caption").textContent).toBe(
      "Pages are nodes, wikilinks are edges — the canvas renderer, repointed.",
    );
    expect(getByTestId("lint-clean").textContent).toContain(
      "153 pages otherwise clean",
    );
    expect(getByTestId("wiki-footer").textContent).toContain(
      "they share pgvector and nothing else",
    );
  });

  it("draws the graph at the size the screenshot shows", () => {
    const { getByTestId } = mount();
    expect(getByTestId("wiki-graph-svg").getAttribute("width")).toBe("258");
    expect(getByTestId("wiki-graph-svg").getAttribute("height")).toBe("132");
  });

  it("paints every surface from a token", () => {
    expect(read("screens/screens.css")).not.toMatch(/#[0-9a-fA-F]{6}\b/);
  });
});
