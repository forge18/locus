import { createSignal, type Accessor } from "solid-js";
import { BOT_AVATAR_STYLE_SETTING, DEFAULT_BOT_AVATAR_STYLE } from "./setting";
import { normalizeAvatarStyle, type AvatarStyleId } from "./derive.ts";

export const BOT_AVATAR_STYLE_STORAGE_KEY = `locus.${BOT_AVATAR_STYLE_SETTING}`;

type StorageReader = Pick<Storage, "getItem">;
type StorageWriter = Pick<Storage, "setItem">;

export function savedAvatarStyle(storage: StorageReader): AvatarStyleId {
  return normalizeAvatarStyle(storage.getItem(BOT_AVATAR_STYLE_STORAGE_KEY));
}

const initialAvatarStyle =
  typeof window === "undefined"
    ? DEFAULT_BOT_AVATAR_STYLE
    : savedAvatarStyle(window.localStorage);
const [currentAvatarStyle, setCurrentAvatarStyle] =
  createSignal<AvatarStyleId>(initialAvatarStyle);

/** Reactive app-wide avatar style. It is install-scoped, never project-scoped. */
export function useAvatarStyle(): Accessor<AvatarStyleId> {
  return currentAvatarStyle;
}

export function persistAvatarStyle(
  storage: StorageWriter,
  value: string,
): AvatarStyleId {
  const resolved = normalizeAvatarStyle(value);
  storage.setItem(BOT_AVATAR_STYLE_STORAGE_KEY, resolved);
  setCurrentAvatarStyle(resolved);
  return resolved;
}

/** Update the setting from a webview control, while remaining safe in non-browser tests. */
export function setAvatarStylePreference(value: string): AvatarStyleId {
  const resolved = normalizeAvatarStyle(value);
  if (typeof window === "undefined") {
    setCurrentAvatarStyle(resolved);
  } else {
    persistAvatarStyle(window.localStorage, resolved);
  }
  return resolved;
}
