export const THEME_STORAGE_KEY = "locus.theme";

export const REQUIRED_THEME_VALUES = [
  "--surface-ground",
  "--surface-chrome",
  "--surface-raised",
  "--surface-selected",
  "--text-primary",
  "--action-attention",
  "--status-working",
  "--status-success",
  "--status-danger",
] as const;

export interface ThemeRegistration {
  id: string;
  label: string;
  fixture: string;
  values: Readonly<Record<(typeof REQUIRED_THEME_VALUES)[number], string>>;
}

export function registerThemes(
  themes: readonly ThemeRegistration[],
): readonly ThemeRegistration[] {
  const ids = new Set<string>();
  for (const theme of themes) {
    if (!theme.id || ids.has(theme.id) || !theme.fixture)
      throw new Error("theme needs a unique id and fixture");
    for (const token of REQUIRED_THEME_VALUES) {
      if (!theme.values[token])
        throw new Error(`${theme.id} is missing ${token}`);
    }
    ids.add(theme.id);
  }
  return themes;
}

export const INSTALLED_THEMES = ["dark", "light"] as const;

export type ThemeId = (typeof INSTALLED_THEMES)[number];

/** The slice of `window` theme resolution needs; jsdom may leave matchMedia undefined. */
export type ThemePreferenceSource = {
  matchMedia?: (query: string) => { matches: boolean };
};

const PREFER_DARK_QUERY = "(prefers-color-scheme: dark)";

/** The OS color-scheme preference. Dark stays the default while it cannot be read. */
export function systemTheme(source: ThemePreferenceSource = window): ThemeId {
  const preference = source.matchMedia?.(PREFER_DARK_QUERY);
  return preference?.matches === false ? "light" : "dark";
}

export function normalizeTheme(
  value: string | null | undefined,
  source: ThemePreferenceSource = window,
): ThemeId {
  if (value === "light" || value === "dark") return value;
  return systemTheme(source);
}

export function savedTheme(
  storage: Pick<Storage, "getItem">,
  source: ThemePreferenceSource = window,
): ThemeId {
  return normalizeTheme(storage.getItem(THEME_STORAGE_KEY), source);
}

export function applyTheme(
  documentElement: Pick<HTMLElement, "dataset">,
  theme: string | null | undefined,
  source: ThemePreferenceSource = window,
): ThemeId {
  const resolved = normalizeTheme(theme, source);
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
