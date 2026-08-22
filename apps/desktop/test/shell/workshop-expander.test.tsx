import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/workshop-expander", () => {
  it("reveals Workshop links only after the Workshop control expands", () => {
    const { getByRole, getByTestId } = render(() => (
      <ProjectRail selectedProject="locus" />
    ));
    const workshop = getByRole("button", { name: "Workshop" });

    expect(getByTestId("workshop-rail-links").hidden).toBe(true);
    fireEvent.click(workshop);
    expect(getByTestId("workshop-rail-links").hidden).toBe(false);
    expect(getByTestId("workshop-rail-links").textContent).toContain("Agents");
  });
});
