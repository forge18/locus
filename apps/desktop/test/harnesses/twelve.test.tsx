import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { HarnessesView } from "../../src/screens/workshop/HarnessesView";
import { useHarnessSummary, useHarnesses } from "../../src/data/harnesses";
import { read } from "../css";

const mount = () => render(() => <HarnessesView />);

describe("harnesses/registered", () => {
  it("renders every registered card", () => {
    const { getByTestId } = mount();
    expect(
      getByTestId("harnesses-grid").querySelectorAll(".hn-card").length,
    ).toBe(11);
  });

  it("takes the count from the registry, not from a literal", () => {
    const { getByTestId } = mount();
    expect(getByTestId("harnesses-count").textContent).toBe("11");
    expect(useHarnessSummary().harnesses).toBe(11);
    expect(useHarnesses().length).toBe(11);
  });

  it("names the registered harnesses", () => {
    expect(useHarnesses().map((h) => h.name)).toEqual([
      "aider",
      "antigravity",
      "claude",
      "codex",
      "copilot",
      "cursor",
      "dsh",
      "gemini",
      "omp",
      "opencode",
      "pi",
    ]);
  });

  it("holds no numeric literal in the screen source", () => {
    const source = read("screens/workshop/HarnessesView.tsx");
    expect(source).not.toMatch(/\b(12|33|96|88|29|27)\b/);
  });

  it("reports every one of them with tui = false, which is why they are here", () => {
    const { getByTestId } = mount();
    expect(getByTestId("harnesses-tui-note").textContent).toContain(
      "tui = false is required on all 11",
    );
  });
});
