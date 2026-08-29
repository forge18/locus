export { Avatar } from "./Avatar";
export type { AvatarProps } from "./Avatar";
export {
  AVATAR_STYLES,
  AVATAR_STYLE_IDS,
  avatarStyleOption,
  deriveAvatar,
  derivedAvatarCacheSize,
  normalizeAvatarStyle,
} from "./derive";
export type { AvatarStyleId, AvatarStyleOption } from "./derive";
export {
  BOT_AVATAR_STYLE_SETTING,
  CORE_SETTINGS_DEFAULTS,
  DEFAULT_BOT_AVATAR_STYLE,
} from "./setting";
export {
  BOT_AVATAR_STYLE_STORAGE_KEY,
  persistAvatarStyle,
  savedAvatarStyle,
  setAvatarStylePreference,
  useAvatarStyle,
} from "./preferences";
