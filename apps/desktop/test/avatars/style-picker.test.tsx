import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AVATAR_STYLES } from "../../src/avatars/derive";
import { setAvatarStylePreference } from "../../src/avatars/preferences";
import { GuardrailsView } from "../../src/screens/settings/GuardrailsView";
import BotsView from "../../src/screens/bots/BotsView";

describe("avatars/style-picker", () => {
  it("lists every bundled style and updates bot avatars immediately", async () => {
    setAvatarStylePreference("bottts");
    const view = render(() => (
      <>
        <GuardrailsView />
        <BotsView />
      </>
    ));
    const before = view.getByTestId("bot-avatar-keeper").getAttribute("src");

    const trigger = view.getByTestId("avatar-style-trigger");
    await fireEvent.pointerDown(trigger, {
      pointerId: 1,
      pointerType: "mouse",
    });
    await fireEvent.pointerUp(trigger, { pointerId: 1, pointerType: "mouse" });
    await waitFor(() =>
      expect(document.querySelectorAll('[role="option"]')).toHaveLength(
        AVATAR_STYLES.length,
      ),
    );
    expect(
      document.querySelector('[data-testid="avatar-style-option-lorelei"]'),
    ).toBeTruthy();

    await fireEvent.click(
      document.querySelector('[data-testid="avatar-style-option-lorelei"]')!,
    );
    expect(view.getByTestId("avatar-style-trigger").textContent).toContain(
      "Lorelei",
    );
    expect(view.getByTestId("bot-avatar-keeper").getAttribute("src")).not.toBe(
      before,
    );
    expect(
      view.getByTestId("bot-avatar-keeper").getAttribute("data-avatar-style"),
    ).toBe("lorelei");
  });
});
