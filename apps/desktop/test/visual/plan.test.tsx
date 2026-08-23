import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { Shell } from "../../src/shell/Shell";
import { PlanView } from "../../src/screens/plan/PlanView";
import { createNavStore } from "../../src/nav";
import { SRC, read, rules } from "../css";

/**
 * Structural conformance against screenshots/05-plan-conversation.png — not a pixel diff.
 * jsdom has no layout engine, so this asserts what the screenshot encodes that
 * survives without one: which elements, in what order, at which declared sizes,
 * carrying which copy.
 */
const SHOTS = resolve(SRC, "../../../docs/design_handoff_locus_desktop/screenshots");
const rule = (file: string, sel: string) =>
  rules(read(file)).find((r) => r.selector === sel)!;

const mount = () => {
  const nav = createNavStore({ view: "plan" });
  return render(() => (
    <Shell nav={nav}>
      <PlanView />
    </Shell>
  ));
};

describe("visual: plan", () => {
  it("has the reference screenshot to conform to", () => {
    expect(existsSync(resolve(SHOTS, "05-plan-conversation.png"))).toBe(true);
  });

  it("is a desktop workspace around the plan list, active tab, and outputs", () => {
    const { getByTestId } = mount();
    for (const part of [
      "titlebar",
      "rail",
      "tabbar",
      "strip",
      "plan-list",
      "plan-conversation",
      "plan-outputs",
    ]) {
      expect(getByTestId(part), part).toBeTruthy();
    }
    expect(rule("screens/screens.css", ".plan-list").body).toContain(
      "clamp(180px, 17%, 260px)",
    );
    expect(rule("screens/screens.css", ".plan-outputs").body).toContain(
      "clamp(240px, 23%, 340px)",
    );
  });

  it("lights Plan on the rail and shows no tabs, because Plan has none", () => {
    const { getByTestId } = mount();
    expect(getByTestId("rail-plan").getAttribute("aria-current")).toBe("true");
    expect(getByTestId("tabbar-category").textContent).toBe("Plan");
    expect(getByTestId("tabbar-tabs").querySelectorAll(".tab").length).toBe(0);
  });

  it("runs the title and nine-stage breadcrumb along the workspace summary", () => {
    const { getByTestId } = mount();
    const head = getByTestId("plan").querySelector(".plan-summary")!;
    expect(head.contains(getByTestId("plan-title"))).toBe(true);
    expect(head.contains(getByTestId("breadcrumb"))).toBe(true);
    expect(getByTestId("breadcrumb").children.length).toBe(9);
  });

  it("stacks the conversation: messages, the scope card, then the live line", () => {
    const { getByTestId } = mount();
    const ids = [...getByTestId("plan-messages").children].map((c) =>
      c.getAttribute("data-testid"),
    );
    expect(ids.filter((id) => id?.startsWith("msg-")).length).toBe(4);
    expect(ids).toContain("scope-decision");
    expect(ids[ids.length - 1]).toBe("plan-live");
  });

  it("shows the outputs rail in the drawn order", () => {
    const { getByTestId } = mount();
    expect(
      [...getByTestId("plan-outputs").querySelectorAll(".output-card")].map(
        (c) => c.getAttribute("data-testid"),
      ),
    ).toEqual([
      "output-spec",
      "output-tasks",
      "output-tools",
      "recommendation",
    ]);
  });

  it("carries the copy the screenshot shows, verbatim", () => {
    const { getByTestId, container } = mount();
    expect(getByTestId("plan-list-footer").textContent).toBe(
      "Nothing reaches the board until one approval at the end.",
    );
    expect(getByTestId("scope-decision-title").textContent).toBe(
      "Scope decision — resolves inline, not as a separate gate",
    );
    expect(getByTestId("recommendation-approve").textContent).toBe(
      "Approve — 4 tasks to the board",
    );
    expect(getByTestId("plan-acp").textContent).toBe("ACP · session/prompt");
    expect(container.textContent).toContain(
      "interviewer is re-opening question 14 of 14",
    );
  });

  it("paints every surface from a token", () => {
    expect(read("screens/screens.css")).not.toMatch(/#[0-9a-fA-F]{6}\b/);
  });
});
