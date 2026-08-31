import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { DispatchPill } from "../../src/shell/DispatchPill";
import { InboxPill } from "../../src/shell/InboxPill";

describe("title pill popover dismissal", () => {
  it("closes the Dispatch popover on Escape, refocuses the pill, and reports the close", async () => {
    const onOpenChange = vi.fn();
    const { getByTestId, queryByTestId } = render(() => (
      <DispatchPill running={1} needsYou={0} onOpenChange={onOpenChange} />
    ));

    await fireEvent.click(getByTestId("dispatch-pill"));
    expect(getByTestId("dispatch-popover")).toBeTruthy();

    await fireEvent.keyDown(document.body, { key: "Escape" });
    expect(queryByTestId("dispatch-popover")).toBeNull();
    expect(onOpenChange).toHaveBeenLastCalledWith(false);
    expect(document.activeElement).toBe(getByTestId("dispatch-pill"));
  });

  it("closes the Dispatch popover on a press outside it but not inside it", async () => {
    const { getByTestId, queryByTestId } = render(() => (
      <DispatchPill running={1} needsYou={0} />
    ));

    await fireEvent.click(getByTestId("dispatch-pill"));
    await fireEvent.pointerDown(getByTestId("dispatch-popover"));
    expect(queryByTestId("dispatch-popover")).not.toBeNull();

    await fireEvent.pointerDown(document.body);
    expect(queryByTestId("dispatch-popover")).toBeNull();
  });

  it("leaves Dispatch trigger presses to toggle instead of dismissing twice", async () => {
    const { getByTestId, queryByTestId } = render(() => (
      <DispatchPill running={1} needsYou={0} />
    ));

    await fireEvent.click(getByTestId("dispatch-pill"));
    await fireEvent.pointerDown(getByTestId("dispatch-pill"));
    expect(queryByTestId("dispatch-popover")).not.toBeNull();

    await fireEvent.click(getByTestId("dispatch-pill"));
    expect(queryByTestId("dispatch-popover")).toBeNull();
  });

  it("closes the Inbox popover on Escape and refocuses the pill", async () => {
    const { getByTestId, queryByTestId } = render(() => (
      <InboxPill count={1} />
    ));

    await fireEvent.click(getByTestId("inbox-pill"));
    expect(getByTestId("inbox-popover")).toBeTruthy();

    await fireEvent.keyDown(document.body, { key: "Escape" });
    expect(queryByTestId("inbox-popover")).toBeNull();
    expect(document.activeElement).toBe(getByTestId("inbox-pill"));
  });

  it("closes the Inbox popover on a press outside it but not on presses inside it", async () => {
    const { getByTestId, queryByTestId } = render(() => (
      <InboxPill count={1} />
    ));

    await fireEvent.click(getByTestId("inbox-pill"));
    await fireEvent.pointerDown(getByTestId("inbox-popover"));
    expect(queryByTestId("inbox-popover")).not.toBeNull();

    await fireEvent.pointerDown(document.body);
    expect(queryByTestId("inbox-popover")).toBeNull();
  });

  it("leaves Inbox trigger presses to the toggle so the pill can close itself", async () => {
    const { getByTestId, queryByTestId } = render(() => <InboxPill count={1} />);

    await fireEvent.click(getByTestId("inbox-pill"));
    await fireEvent.pointerDown(getByTestId("inbox-pill"));
    expect(queryByTestId("inbox-popover")).not.toBeNull();

    await fireEvent.click(getByTestId("inbox-pill"));
    expect(queryByTestId("inbox-popover")).toBeNull();
  });
});
