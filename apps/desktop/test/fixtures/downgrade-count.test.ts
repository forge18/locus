import { describe, expect, it } from "vitest";
import {
  DOWNGRADE_COUNT,
  ENTRY_COUNT,
  EXTENSION_TYPES,
  HARNESSES,
} from "../../src/fixtures/generated/harnesses";

describe("fixtures/downgrade-count", () => {
  it("reports 29 downgrades across 88 entries", () => {
    expect(DOWNGRADE_COUNT).toBe(29);
    expect(ENTRY_COUNT).toBe(88);
  });

  it("computes the entry count rather than stating it", () => {
    expect(ENTRY_COUNT).toBe(HARNESSES.length * EXTENSION_TYPES.length);
    expect(HARNESSES.flatMap((h) => h.extensions).length).toBe(ENTRY_COUNT);
  });

  it("counts a downgrade only where the file names what was lost", () => {
    const named = HARNESSES.flatMap((h) => h.extensions).filter(
      (e) => e.weakerThanNative,
    );
    expect(named.length).toBe(DOWNGRADE_COUNT);
    for (const e of named) {
      expect(
        e.weakerThanNative!.length,
        `${e.type} says nothing`,
      ).toBeGreaterThan(10);
    }
  });

  it("has every harness declare all eight extension types", () => {
    for (const h of HARNESSES) {
      expect(h.extensions.map((e) => e.type).sort(), h.name).toEqual(
        [...EXTENSION_TYPES].sort(),
      );
    }
  });

  it("leaves the reference harnesses with nothing lost", () => {
    for (const name of ["claude", "pi", "omp"]) {
      const h = HARNESSES.find((x) => x.name === name)!;
      expect(h.extensions.filter((e) => e.weakerThanNative).length, name).toBe(
        0,
      );
    }
  });
});
