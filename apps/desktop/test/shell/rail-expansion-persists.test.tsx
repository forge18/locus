import { fireEvent, render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import {
  ProjectRail,
  RAIL_EXPANSION_STORAGE_KEY,
} from "../../src/shell/ProjectRail";

afterEach(() => localStorage.removeItem(RAIL_EXPANSION_STORAGE_KEY));

describe("shell/rail-expansion-persists", () => {
  it("restores Knowledge and Extensions expansion after remount", () => {
    const first = render(() => <ProjectRail selectedProject="locus" />);
    fireEvent.click(first.getByRole("button", { name: "Knowledge" }));
    fireEvent.click(
      first.getByRole("button", { name: "Extensions / Plugins" }),
    );
    first.unmount();

    const restored = render(() => <ProjectRail selectedProject="locus" />);
    expect(restored.getByTestId("memory-rail-links").hidden).toBe(false);
    expect(restored.getByTestId("workshop-rail-links").hidden).toBe(false);
  });

  it("ignores malformed persisted expansion state", () => {
    localStorage.setItem(RAIL_EXPANSION_STORAGE_KEY, "not json");

    const rail = render(() => <ProjectRail selectedProject="locus" />);

    expect(rail.getByTestId("memory-rail-links").hidden).toBe(true);
    expect(rail.getByTestId("workshop-rail-links").hidden).toBe(true);
  });
});
