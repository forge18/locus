import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { Shell } from "../../src/shell/Shell";
import { AgentDefsView } from "../../src/screens/workshop/AgentDefsView";
import { ExtensionsView } from "../../src/screens/workshop/ExtensionsView";
import { HarnessesView } from "../../src/screens/workshop/HarnessesView";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";
import { BackLink, createNavStore } from "../../src/nav";
import type { View } from "../../src/nav";
import type { JSX } from "solid-js";
import { SRC, read, rules } from "../css";

/**
 * Structural conformance against screenshots 20, 21, 23 and 30 — not a pixel
 * diff. jsdom has no layout engine; what is asserted is what the screenshots
 * encode that survives without one.
 */
const SHOTS = resolve(SRC, "../../../docs/design_handoff_locus_v2/screenshots");
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;

const mount = (view: View, screen: () => JSX.Element) => {
  const nav = createNavStore({ view });
  const r = render(() => <Shell nav={nav}>{screen()}</Shell>);
  return { nav, ...r };
};

describe("visual: workshop", () => {
  it("has all four reference screenshots to conform to", () => {
    for (const shot of [
      "20-workshop-agents.png",
      "21-workshop-cli.png",
      "23-workshop-harnesses.png",
      "30-workflows-visual.png",
    ]) {
      expect(existsSync(resolve(SHOTS, shot)), shot).toBe(true);
    }
  });

  it("lights Workshop with three tabs, and no Agents tab among them", () => {
    const { getByTestId } = mount("extensions", () => (
      <ExtensionsView onNavigate={() => {}} />
    ));
    expect(getByTestId("rail-workshop").getAttribute("aria-current")).toBe(
      "true",
    );
    const tabs = [...getByTestId("tabbar-tabs").querySelectorAll(".tab")].map(
      (t) => t.textContent,
    );
    expect(tabs).toEqual(["Extensions", "Workflow", "Harnesses"]);
    expect(tabs).not.toContain("Agents");
  });

  it("extensions: eight cards, recently edited, materialization", () => {
    const { getByTestId } = mount("extensions", () => (
      <ExtensionsView onNavigate={() => {}} />
    ));
    expect(getByTestId("type-grid").querySelectorAll(".type-card").length).toBe(
      8,
    );
    expect(getByTestId("recently-edited")).toBeTruthy();
    expect(getByTestId("materialization")).toBeTruthy();
  });

  it("agent definitions: Extensions stays lit, and the back link renders", () => {
    const nav = createNavStore({ view: "agents" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <AgentDefsView onNavigate={nav.go} />
      </Shell>
    ));
    expect(getByTestId("rail-workshop").getAttribute("aria-current")).toBe(
      "true",
    );
    const selected = getByTestId("tabbar-tabs").querySelectorAll(
      ".tab[data-selected]",
    );
    expect(selected.length).toBe(1);
    expect(selected[0].textContent).toBe("Extensions");

    const back = render(() => <BackLink nav={nav} />);
    expect(back.getByTestId("drilldown-back").textContent).toBe("Extensions");
  });

  it("workflow: a palette near 180px, a canvas that takes the room, seven chips", () => {
    const { getByTestId } = mount("canvas", () => <WorkflowView />);
    expect(rule(".wf-palette").body).toContain("clamp(150px, 14%, 220px)");
    expect(rule(".wf-canvas").body).toContain("flex: 1 1 auto");
    expect(getByTestId("wf-palette").querySelectorAll(".wf-chip").length).toBe(
      7,
    );
    expect(getByTestId("wf-inspector")).toBeTruthy();
  });

  it("harnesses: registered cards in a grid that reflows", () => {
    const { getByTestId } = mount("harnesses", () => <HarnessesView />);
    expect(rule(".hn-grid").body).toContain(
      "repeat(auto-fit, minmax(230px, 1fr))",
    );
    expect(
      getByTestId("harnesses-grid").querySelectorAll(".hn-card").length,
    ).toBe(11);
  });

  it("carries the copy the screenshots show, verbatim", () => {
    const extensions = mount("extensions", () => (
      <ExtensionsView onNavigate={() => {}} />
    ));
    expect(extensions.getByTestId("extensions-note").textContent).toContain(
      "authored once here, materialized fresh into every runtime at run start",
    );
    extensions.unmount();

    const workflow = mount("canvas", () => <WorkflowView />);
    expect(workflow.getByTestId("wf-no-model-note").textContent).toBe(
      "No model in the orchestration path — the graph decides",
    );
    workflow.unmount();

    const harnesses = mount("harnesses", () => <HarnessesView />);
    expect(harnesses.getByTestId("harnesses-tui-note").textContent).toContain(
      "a harness claiming true is refused at registration",
    );
  });

  it("shows the computed 29 of 88, not the screenshot’s stale 27 of 88", () => {
    const { getByTestId } = mount("harnesses", () => <HarnessesView />);
    const foot = getByTestId("harnesses-foot").textContent!;
    expect(foot).toContain("29 of 88");
    expect(foot).not.toContain("27 of 88");
  });

  it("paints every surface from a token", () => {
    expect(read("screens/screens.css")).not.toMatch(/#[0-9a-fA-F]{6}\b/);
  });
});
