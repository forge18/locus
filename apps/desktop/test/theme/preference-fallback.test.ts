import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  applyTheme,
  normalizeTheme,
  persistTheme,
  savedTheme,
  systemTheme,
  THEME_STORAGE_KEY,
  type ThemePreferenceSource,
} from "../../src/styles/theme";
import { read } from "../css";

/** A stand-in for `window` whose color-scheme query answers `prefersDark`. */
function osPreference(prefersDark: boolean): ThemePreferenceSource {
  return { matchMedia: () => ({ matches: prefersDark }) };
}

/** Runs the inline theme bootstrap from index.html against jsdom. */
function bootFromHtml(): string | undefined {
  const script = read("../index.html").match(
    /<script>([\s\S]*?)<\/script>/,
  )?.[1];
  if (!script) throw new Error("index.html lost its theme bootstrap script");
  delete document.documentElement.dataset.theme;
  new Function(script)();
  return document.documentElement.dataset.theme;
}

describe("theme/preference-fallback", () => {
  beforeEach(() => {
    window.localStorage.clear();
    delete document.documentElement.dataset.theme;
  });
  afterEach(() => vi.unstubAllGlobals());

  it("maps the OS color-scheme preference onto the installed themes", () => {
    expect(systemTheme(osPreference(true))).toBe("dark");
    expect(systemTheme(osPreference(false))).toBe("light");
  });

  it("falls back to Dark when the OS preference cannot be read", () => {
    expect(systemTheme({})).toBe("dark");
    expect(normalizeTheme(null, {})).toBe("dark");
    expect(normalizeTheme("midnight", {})).toBe("dark");
  });

  it("keeps an exactly-stored theme regardless of the OS preference", () => {
    expect(normalizeTheme("dark", osPreference(false))).toBe("dark");
    expect(normalizeTheme("light", osPreference(true))).toBe("light");
  });

  it("resolves missing and unknown values to the OS preference", () => {
    for (const value of [null, undefined, "", "midnight"]) {
      expect(normalizeTheme(value, osPreference(false))).toBe("light");
      expect(normalizeTheme(value, osPreference(true))).toBe("dark");
    }
  });

  it("falls back to the OS preference when storage holds nothing usable", () => {
    expect(savedTheme(window.localStorage, osPreference(false))).toBe("light");
    window.localStorage.setItem(THEME_STORAGE_KEY, "midnight");
    expect(savedTheme(window.localStorage, osPreference(false))).toBe("light");
    window.localStorage.setItem(THEME_STORAGE_KEY, "light");
    expect(savedTheme(window.localStorage, osPreference(true))).toBe("light");
  });

  it("reads the OS preference through window when no source is injected", () => {
    vi.stubGlobal(
      "matchMedia",
      (query: string) => ({ matches: false, media: query }),
    );
    expect(savedTheme(window.localStorage)).toBe("light");
    expect(applyTheme(document.documentElement, undefined)).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("persists only the stable identifier and applies it to the root", () => {
    const root = document.documentElement;
    expect(persistTheme(window.localStorage, root, "light")).toBe("light");
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("light");
    expect(root.dataset.theme).toBe("light");
  });

  it("falls back safely when an invalid value is applied", () => {
    expect(applyTheme(document.documentElement, "custom")).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("no longer hard-codes a theme on the root element", () => {
    expect(read("../index.html")).not.toMatch(/<html[^>]*data-theme/);
  });

  it("boots into the stored theme from index.html before first paint", () => {
    window.localStorage.setItem(THEME_STORAGE_KEY, "light");
    expect(bootFromHtml()).toBe("light");
    window.localStorage.setItem(THEME_STORAGE_KEY, "dark");
    expect(bootFromHtml()).toBe("dark");
  });

  it("boots into the OS preference from index.html when nothing is stored", () => {
    vi.stubGlobal(
      "matchMedia",
      (query: string) => ({ matches: false, media: query }),
    );
    expect(bootFromHtml()).toBe("light");
    vi.stubGlobal(
      "matchMedia",
      (query: string) => ({ matches: true, media: query }),
    );
    expect(bootFromHtml()).toBe("dark");
  });

  it("boots Dark from index.html when no preference resolves", () => {
    expect(bootFromHtml()).toBe("dark");
  });
});
