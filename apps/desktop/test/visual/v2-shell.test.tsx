import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { Shell } from "../../src/shell/Shell";

const fixtures = ["default", "running", "needs-you"] as const;

describe("visual/v2-shell", () => {
  for (const fixture of fixtures) {
    it(`captures the ${fixture} shell fixture`, () => {
      const { getByTestId } = render(() => (
        <Shell nav={createNavStore()}>
          <div data-testid="fixture-body" data-fixture={fixture} />
        </Shell>
      ));
      expect(getByTestId("window")).toBeTruthy();
      expect(getByTestId("project-rail")).toBeTruthy();
      expect(getByTestId("fixture-body").getAttribute("data-fixture")).toBe(
        fixture,
      );
    });
  }
});
