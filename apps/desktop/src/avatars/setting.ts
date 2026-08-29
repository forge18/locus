/** Install-wide core.settings key for the derived bot avatar style. */
export const BOT_AVATAR_STYLE_SETTING = "bots.avatar_style" as const;
export const DEFAULT_BOT_AVATAR_STYLE = "bottts" as const;

/** Missing settings resolve to these shipped values without creating persistence. */
export const CORE_SETTINGS_DEFAULTS = Object.freeze({
  [BOT_AVATAR_STYLE_SETTING]: DEFAULT_BOT_AVATAR_STYLE,
});
