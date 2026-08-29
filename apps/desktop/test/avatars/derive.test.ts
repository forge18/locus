import * as collection from "@dicebear/collection";
import { describe, expect, it } from "vitest";
import {
  AVATAR_STYLES,
  deriveAvatar,
  derivedAvatarCacheSize,
} from "../../src/avatars/derive";
import { DEFAULT_BOT_AVATAR_STYLE } from "../../src/data/settings";

describe("avatars/derive", () => {
  it("registers every bundled DiceBear style with attribution", () => {
    expect(AVATAR_STYLES).toHaveLength(Object.keys(collection).length);
    expect(AVATAR_STYLES.every((style) => style.creator && style.license)).toBe(
      true,
    );
    expect(
      AVATAR_STYLES.some((style) => style.id === DEFAULT_BOT_AVATAR_STYLE),
    ).toBe(true);
  });

  it("returns a transparent SVG data URI and memoizes it by style and seed", () => {
    const before = derivedAvatarCacheSize();
    const first = deriveAvatar(DEFAULT_BOT_AVATAR_STYLE, "bot-keeper");
    const second = deriveAvatar(DEFAULT_BOT_AVATAR_STYLE, "bot-keeper");

    expect(first).toMatch(/^data:image\/svg\+xml;utf8,/);
    expect(decodeURIComponent(first)).toMatch(/^data:image\/svg\+xml;utf8,/);
    expect(first).toBe(second);
    expect(derivedAvatarCacheSize()).toBe(before + 1);
  });
});
