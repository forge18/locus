import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { HarnessesView } from "../../src/screens/workshop/HarnessesView";
import {
  useExtensionTypes,
  useHarnessSummary,
  useHarnesses,
} from "../../src/data/harnesses";
import { read } from "../css";

const mount = () => render(() => <HarnessesView />);

describe("harnesses/computed-summary", () => {
  it("reports 29 of 88", () => {
    const { getByTestId } = mount();
    expect(getByTestId("harnesses-downgrade-line").textContent).toContain(
      "29 of 88",
    );
  });

  it("computes entries as harnesses times extension types", () => {
    expect(useHarnessSummary().entries).toBe(
      useHarnesses().length * useExtensionTypes().length,
    );
    expect(useHarnessSummary().entries).toBe(88);
  });

  it("computes downgrades by counting entries that name what was lost", () => {
    const counted = useHarnesses()
      .flatMap((h) => h.extensions)
      .filter((e) => e.weakerThanNative).length;
    expect(counted).toBe(29);
    expect(useHarnessSummary().downgrades).toBe(29);
  });

  it("holds both numbers only in the generated file", () => {
    expect(read("fixtures/generated/harnesses.ts")).toContain(
      "DOWNGRADE_COUNT = 29",
    );
    expect(read("screens/workshop/HarnessesView.tsx")).not.toMatch(
      /\b(29|88)\b/,
    );
  });
});
