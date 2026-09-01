import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

for (const state of ["ready", "working", "blocked"] as const) {
  describe("shell/dispatch-dot", () => {
    it(`renders the ${state} Dispatch state`, () => {
      const { getByTestId } = render(() => (
        <ProjectRail selectedProject="locus" dispatchState={state} />
      ));
      expect(getByTestId("project-rail").querySelector('[data-testid="dispatch-dot"]')).toBeNull();
    });
  });
}
