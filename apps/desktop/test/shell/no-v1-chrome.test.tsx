import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { Shell } from "../../src/shell/Shell";

describe("shell/no-v1-chrome", () => {
  it("does not render the v1 filter, tab bar, or activity strip", () => {
    const { container } = render(() => (
      <Shell nav={createNavStore()}>
        <div />
      </Shell>
    ));
    expect(container.querySelector('[data-testid="tabbar"]')).toBeNull();
    expect(
      container.querySelector('[data-testid="activity-strip"]'),
    ).toBeNull();
    expect(
      container.querySelector('[data-testid="project-filter"]'),
    ).toBeNull();
  });
});
