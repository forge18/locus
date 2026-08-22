import { describe, expect, it } from "vitest";
import { registerThemes, type ThemeRegistration } from "../../src/styles/theme";

const testTheme: ThemeRegistration = {
  id: "high-contrast",
  label: "High contrast",
  fixture: "v2-dashboard",
  values: {
    "--surface-ground": "#000000",
    "--surface-chrome": "#101010",
    "--surface-raised": "#181818",
    "--surface-selected": "#242424",
    "--text-primary": "#ffffff",
    "--action-attention": "#ffdd00",
    "--status-working": "#b8a8ff",
    "--status-success": "#8de0ad",
    "--status-danger": "#ff9b91",
  },
};

describe("theme/registers-without-component-changes", () => {
  it("accepts a value set and fixture declaration without component registration", () => {
    expect(registerThemes([testTheme])).toEqual([testTheme]);
  });

  it("rejects incomplete value sets before a theme can ship", () => {
    const incomplete = { ...testTheme, values: { ...testTheme.values } };
    delete (incomplete.values as Partial<typeof incomplete.values>)["--status-danger"];
    expect(() => registerThemes([incomplete as ThemeRegistration])).toThrow("--status-danger");
  });
});
