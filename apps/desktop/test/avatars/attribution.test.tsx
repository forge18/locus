import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AVATAR_STYLES } from "../../src/avatars/derive";
import { setAvatarStylePreference } from "../../src/avatars/preferences";
import { GuardrailsView } from "../../src/screens/settings/GuardrailsView";

describe("avatars/attribution", () => {
  it("keeps the active creator and license visible", () => {
    setAvatarStylePreference("bottts");
    const view = render(() => <GuardrailsView />);
    const bottts = AVATAR_STYLES.find((style) => style.id === "bottts")!;
    const attribution = view.getByTestId("avatar-attribution");

    expect(attribution.textContent).toContain(bottts.creator);
    expect(attribution.textContent).toContain(bottts.license);
    expect(attribution.querySelector("a")?.getAttribute("href")).toBe(
      bottts.licenseUrl,
    );
  });

  it("shows creator and license metadata in every picker entry", async () => {
    setAvatarStylePreference("bottts");
    const view = render(() => <GuardrailsView />);
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

    for (const style of AVATAR_STYLES) {
      const option = document.querySelector(
        `[data-testid="avatar-style-option-${style.id}"]`,
      );
      expect(option?.textContent).toContain(style.creator);
      expect(option?.textContent).toContain(style.license);
    }
  });
});
