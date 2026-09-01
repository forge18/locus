import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AppTitleBar } from "../../src/shell/AppTitleBar";

describe("a11y/shell-live-regions", () => {
  it("announces action-required counts assertively and run noise politely", () => {
    const { getByTestId } = render(() => (
      <AppTitleBar
        categoryLabel="Telemetry"
        viewLabel="Runs"
        running={2}
        needsYou={1}
      />
    ));
    expect(getByTestId("running-count").getAttribute("aria-live")).toBe(
      "polite",
    );
    expect(getByTestId("needs-you-count").getAttribute("aria-live")).toBe(
      "assertive",
    );
  });
});
