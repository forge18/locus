export const THEME_STORAGE_KEY = "locus.theme";
export const INSTALLED_THEMES = ["dark", "light"] as const;

export type ThemeId = (typeof INSTALLED_THEMES)[number];

export function normalizeTheme(value: string | null | undefined): ThemeId {
  return value === "light" ? "light" : "dark";
}

export function savedTheme(storage: Pick<Storage, "getItem">): ThemeId {
  return normalizeTheme(storage.getItem(THEME_STORAGE_KEY));
}

export function applyTheme(
  documentElement: Pick<HTMLElement, "dataset">,
  theme: string | null | undefined,
): ThemeId {
  const resolved = normalizeTheme(theme);
  documentElement.dataset.theme = resolved;
  return resolved;
}

export function persistTheme(
  storage: Pick<Storage, "setItem">,
  documentElement: Pick<HTMLElement, "dataset">,
  theme: string,
): ThemeId {
  const resolved = applyTheme(documentElement, theme);
  storage.setItem(THEME_STORAGE_KEY, resolved);
  return resolved;
}
