import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/project-switcher-keyboard", () => {
  it("moves the active project match with Arrow keys", () => {
    const { getByTestId } = render(() => (
      <ProjectRail
        selectedProject="locus"
        projects={["locus", "locus-cli", "tapestry"]}
      />
    ));
    const filter = getByTestId("project-switcher-filter");

    fireEvent.input(filter, { target: { value: "loc" } });
    fireEvent.keyDown(filter, { key: "ArrowDown" });
    expect(
      getByTestId("project-switcher-option-locus-cli").getAttribute(
        "aria-selected",
      ),
    ).toBe("true");

    fireEvent.keyDown(filter, { key: "ArrowDown" });
    expect(
      getByTestId("project-switcher-option-locus").getAttribute(
        "aria-selected",
      ),
    ).toBe("true");
  });
});
