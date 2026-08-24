import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { DispatchView } from "../../src/screens/dispatch/DispatchView";

describe("dispatch permission mode", () => {
  it("defaults each new job to bypass and explains the consequence", async () => {
    const { getByTestId, getByLabelText } = render(() => (
      <DispatchView tab="schedules" />
    ));
    const mode = getByTestId("dispatch-permission-mode");

    expect((mode.querySelector('input[value="bypass"]') as HTMLInputElement).checked).toBe(true);
    expect(getByTestId("dispatch-permission-consequence").textContent).toContain(
      "bypass alarm",
    );

    await fireEvent.click(getByLabelText("Gated approval"));
    expect((mode.querySelector('input[value="gated"]') as HTMLInputElement).checked).toBe(true);
    expect(getByTestId("dispatch-permission-consequence").textContent).toContain(
      "wait for a human action",
    );
  });
});
