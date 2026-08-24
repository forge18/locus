import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { PlanView } from "../../src/screens/plan/PlanView";
import { ONE_APPROVAL_RULE } from "../../src/data/plan";
import { read, rules } from "../css";

const mount = () => render(() => <PlanView />);

describe("plan/one-approval-rule", () => {
  it("carries the rule verbatim in the list footer", () => {
    const { getByTestId } = mount();
    expect(getByTestId("plan-list-footer").textContent).toBe(
      "Nothing reaches the board until one approval at the end.",
    );
  });

  it("states it from one constant, so the screen and the spec cannot drift", () => {
    expect(ONE_APPROVAL_RULE).toBe(
      "Nothing reaches the board until one approval at the end.",
    );
  });

  it("sits under a top hairline, below the list rather than in it", () => {
    const rule = rules(read("screens/screens.css")).find(
      (r) => r.selector === ".plan-list-footer",
    )!;
    expect(rule.body).toContain("border-top: 1px solid var(--border-subtle)");
    expect(rule.body).toContain("flex: none");
  });

  it("offers exactly one approve action on the whole screen", () => {
    const { container } = mount();
    expect(
      container.querySelectorAll('[data-testid="recommendation-approve"]'),
    ).toHaveLength(1);
    expect(container.querySelectorAll("button").length).toBeGreaterThan(0);
  });
});
