import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { Shell } from "../../src/shell/Shell";

describe("shell/project-switcher", () => {
  it("does not render a global project switcher", () => {
    const nav = createNavStore();
    const { queryByTestId } = render(() => (
      <Shell nav={nav}>
        <div />
      </Shell>
    ));

    expect(queryByTestId("project-switcher-option-tapestry")).toBeNull();
    expect(queryByTestId("project-switcher-option-loom-db")).toBeNull();
  });
});
