import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/memory-expander", () => {
  it("reveals Knowledge links only after the Knowledge control expands", () => {
    const { getByRole, getByTestId } = render(() => (
      <ProjectRail selectedProject="locus" />
    ));
    const memory = getByRole("button", { name: "Knowledge" });

    expect(getByTestId("memory-rail-links").hidden).toBe(true);
    fireEvent.click(memory);
    expect(getByTestId("memory-rail-links").hidden).toBe(false);
    expect(getByTestId("memory-rail-links").textContent).toContain(
      "Short-term",
    );
  });
});
