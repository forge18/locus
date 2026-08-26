import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("workshop/extensions-scope", () => {
  it("keeps extension editors and Workflows under Extensions", () => {
    const view = render(() => <ProjectRail selectedProject="locus" />);
    fireEvent.click(view.getByRole("button", { name: "Workshop" }));
    const labels = [
      ...view
        .getByTestId("workshop-extension-links")
        .querySelectorAll("button"),
    ].map((button) => button.textContent);
    expect(labels).toEqual([
      "Agents",
      "Commands",
      "Base context",
      "Hooks",
      "Linters",
      "Output styles",
      "Rules",
      "Skills",
      "Workflows",
    ]);
    expect(labels).not.toContain("Harness");
    expect(labels).not.toContain("Provider");
    expect(labels).not.toContain("CLI Tool");
    expect(labels).not.toContain("Canvas");
  });
});
