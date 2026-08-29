import { beforeEach, describe, expect, it } from "vitest";
import {
  BOT_AVATAR_STYLE_SETTING,
  CORE_SETTINGS_DEFAULTS,
  DEFAULT_BOT_AVATAR_STYLE,
} from "../../src/data/settings";
import {
  BOT_AVATAR_STYLE_STORAGE_KEY,
  persistAvatarStyle,
  savedAvatarStyle,
} from "../../src/avatars/preferences";

beforeEach(() => {
  window.localStorage.clear();
});

describe("avatars/style-setting", () => {
  it("declares the app-wide core setting and Bottts default", () => {
    expect(BOT_AVATAR_STYLE_SETTING).toBe("bots.avatar_style");
    expect(CORE_SETTINGS_DEFAULTS[BOT_AVATAR_STYLE_SETTING]).toBe(
      DEFAULT_BOT_AVATAR_STYLE,
    );
    expect(savedAvatarStyle(window.localStorage)).toBe("bottts");
  });

  it("normalizes unknown stored styles to the default", () => {
    window.localStorage.setItem(BOT_AVATAR_STYLE_STORAGE_KEY, "not-a-style");
    expect(savedAvatarStyle(window.localStorage)).toBe("bottts");
  });

  it("persists a valid style without making it project state", () => {
    const storage = new Map<string, string>();
    const fakeStorage = {
      setItem(key: string, value: string) {
        storage.set(key, value);
      },
    };

    expect(persistAvatarStyle(fakeStorage, "lorelei")).toBe("lorelei");
    expect(storage.get(BOT_AVATAR_STYLE_STORAGE_KEY)).toBe("lorelei");
  });
});
