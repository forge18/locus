import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { HarnessesView } from "../../src/screens/workshop/HarnessesView";
import { useHarnesses } from "../../src/data/harnesses";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;
const mount = () => render(() => <HarnessesView />);

describe("harnesses/mechanism-badges", () => {
  it("badges every card", () => {
    const { getByTestId } = mount();
    for (const harness of useHarnesses()) {
      expect(
        getByTestId(`hn-badge-${harness.name}`).textContent,
        harness.name,
      ).toBe(harness.badge.label);
    }
  });

  it("uses the ACP mechanism for every registered harness", () => {
    for (const harness of useHarnesses()) {
      expect(harness.badge).toEqual({ variant: "acp", label: "ACP" });
    }
  });

  it("gives ACP its own blue", () => {
    const { getByTestId } = mount();
    const acp = useHarnesses()[0]!;
    expect(getByTestId(`hn-badge-${acp.name}`).textContent).toBe("ACP");
    expect(rule(".hn-badge-acp").body).toContain(
      "background: color-mix(in srgb, var(--code-keyword) 18%, transparent)",
    );
    expect(rule(".hn-badge-acp").body).toContain("color: var(--code-keyword)");
  });

  it("derives the badge from the file, never from a table in the source", () => {
    expect(read("screens/workshop/HarnessesView.tsx")).toContain(
      "harness.badge.label",
    );
    expect(read("screens/workshop/HarnessesView.tsx")).not.toMatch(
      /'hooks · plugin'|'ACP'/,
    );
  });
});
