import { describe, expect, it } from "vitest";
import {
  DOWNGRADE_COUNT,
  EXTENSION_COUNTS,
  EXTENSION_TYPES,
  HARNESSES,
} from "../../src/fixtures/generated/harnesses";

describe("fixtures/extension-counts", () => {
  it("reports native and downgraded for each of the eight types", () => {
    expect(EXTENSION_COUNTS.map((c) => c.type)).toEqual([...EXTENSION_TYPES]);
    for (const c of EXTENSION_COUNTS) {
      expect(c.native + c.downgraded, c.type).toBe(HARNESSES.length);
    }
  });

  it("adds up to the same registry downgrades the summary reports", () => {
    expect(EXTENSION_COUNTS.reduce((n, c) => n + c.downgraded, 0)).toBe(
      DOWNGRADE_COUNT,
    );
  });

  it("comes from the same parse as the harness list, not a second count", () => {
    for (const c of EXTENSION_COUNTS) {
      const downgraded = HARNESSES.filter(
        (h) => h.extensions.find((e) => e.type === c.type)!.weakerThanNative,
      ).length;
      expect(downgraded, c.type).toBe(c.downgraded);
    }
  });

  it("shows context as the type nothing downgrades — every harness reads a file", () => {
    expect(EXTENSION_COUNTS.find((c) => c.type === "context")!.downgraded).toBe(
      0,
    );
  });
});
