import { describe, expect, it } from "vitest";
import { fireEvent, render } from "@solidjs/testing-library";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("nav/project-rail", () => {
  it("emits canonical locators for global routes", () => {
    const locators: string[] = [];
    const { getByText } = render(() => (
      <ProjectRail
        selectedProject="tapestry"
        onNavigate={(locator) => locators.push(locator)}
      />
    ));
    fireEvent.click(getByText("Inbox"));
    fireEvent.click(getByText("Dashboard"));
    expect(locators).toEqual([
      "locus://global/inbox",
      "locus://global/dashboard",
    ]);
  });

  it("keeps exactly one global route in the keyboard tab sequence", () => {
    const { getByTestId } = render(() => (
      <ProjectRail selectedProject="tapestry" />
    ));
    expect(
      getByTestId("global-rail-routes").querySelectorAll('button[tabindex="0"]'),
    ).toHaveLength(1);
  });
});
