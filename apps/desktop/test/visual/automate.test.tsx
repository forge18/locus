import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { Shell } from "../../src/shell/Shell";
import { AgentsView } from "../../src/screens/automate/AgentsView";
import { KanbanView } from "../../src/screens/automate/KanbanView";
import { createNavStore } from "../../src/nav";
import { SRC, read, rules } from "../css";

/**
 * Structural conformance against screenshots/09-automate-kanban.png and
 * 10-automate-agents.png — not a pixel diff. jsdom has no layout engine; what is
 * asserted is what the screenshots encode that survives without one.
 */
const SHOTS = resolve(SRC, "../../../docs/design_handoff_locus_v2/screenshots");
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;

const mountKanban = () => {
  const nav = createNavStore({ view: "board" });
  return render(() => (
    <Shell nav={nav}>
      <KanbanView />
    </Shell>
  ));
};

const mountAgents = () => {
  const nav = createNavStore({ view: "sessions" });
  return render(() => (
    <Shell nav={nav}>
      <AgentsView />
    </Shell>
  ));
};

describe("visual: automate", () => {
  it("has both reference screenshots to conform to", () => {
    expect(existsSync(resolve(SHOTS, "09-automate-kanban.png"))).toBe(true);
    expect(existsSync(resolve(SHOTS, "10-automate-agents.png"))).toBe(true);
  });

  it("kanban: Automate lit, Kanban tab first and selected", () => {
    const { getByTestId } = mountKanban();
    expect(getByTestId("rail-automate").getAttribute("aria-current")).toBe(
      "true",
    );
    expect(getByTestId("tabbar-category").textContent).toBe("Automate");
    const tabs = [...getByTestId("tabbar-tabs").querySelectorAll(".tab")].map(
      (t) => t.textContent,
    );
    expect(tabs).toEqual(["Kanban", "Agents"]);
    expect(getByTestId("tab-board").getAttribute("data-selected")).toBe("");
  });

  it("kanban: header notes over six columns at 9px gaps", () => {
    const { getByTestId } = mountKanban();
    expect(getByTestId("kanban-title").textContent).toBe(
      "Fixed columns across every project",
    );
    expect(
      getByTestId("kanban-columns").querySelectorAll(".kanban-column").length,
    ).toBe(6);
    expect(rule(".kanban-columns").body).toContain("gap: var(--g-4)");
  });

  it("kanban: the four card variants the screenshot draws", () => {
    const { getByTestId, container } = mountKanban();
    expect(container.querySelector(".task-card-stuck")).not.toBe(null);
    expect(container.querySelector(".task-card-approval")).not.toBe(null);
    expect(container.querySelector(".task-card-done")).not.toBe(null);
    expect(
      getByTestId("task-card-t-002").querySelector("use")!.getAttribute("href"),
    ).toBe("#ph-prohibit-inset");
  });

  it("agents: Automate lit, Agents tab selected", () => {
    const { getByTestId } = mountAgents();
    expect(getByTestId("rail-automate").getAttribute("aria-current")).toBe(
      "true",
    );
    expect(getByTestId("tab-sessions").getAttribute("data-selected")).toBe("");
  });

  it("agents: a 356px list beside the transcript", () => {
    const { getByTestId } = mountAgents();
    expect(
      (getByTestId("session-list") as HTMLElement).style.getPropertyValue(
        "--pane-w",
      ),
    ).toBe("356px");
    expect(getByTestId("transcript-pane")).toBeTruthy();
  });

  it("agents: header, transcript, conditional footer, status bar, in that order", () => {
    const { getByTestId } = mountAgents();
    const pane = getByTestId("transcript-pane");
    const order = [...pane.children].map(
      (c) => c.getAttribute("data-testid") ?? c.className,
    );
    expect(order[0]).toBe("transcript-head");
    expect(order[1]).toBe("transcript");
    expect(order[2]).toBe("session-footer-stuck");
    expect(order[3]).toBe("session-status-bar");
  });

  it("carries the copy both screenshots show, verbatim", () => {
    const kanban = mountKanban();
    expect(kanban.getByTestId("kanban-blocked-note").textContent).toContain(
      "blocked is a status, not a column",
    );
    kanban.unmount();

    const agents = mountAgents();
    expect(agents.getByTestId("session-list-foot").textContent).toContain(
      "a session you stopped watching is not a session you ended",
    );
    expect(agents.getByTestId("pty-note").textContent).toBe(
      "PTY attached from the host · one session per terminal",
    );
  });

  it("paints every surface from a token", () => {
    expect(read("screens/screens.css")).not.toMatch(/#[0-9a-fA-F]{6}\b/);
  });
});
