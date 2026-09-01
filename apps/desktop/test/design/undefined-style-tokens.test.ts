import { describe, expect, it } from "vitest";
import { read, rules } from "../css";

const tokens = read("styles/tokens.css");
const screens = read("screens/screens.css");
const manage = read("screens/manage/manage.css");
const agentPane = read("panes/agent-pane.css");
const extensionEditor = read("screens/workshop/ExtensionEditor.css");
const tokenRules = rules(tokens);

const themeRule = (theme: string) =>
  tokenRules.find((rule) =>
    new RegExp(`\\[data-theme=["']${theme}["']\\]`).test(rule.selector),
  );

const tokenValue = (theme: string, token: string): string => {
  const body = themeRule(theme)?.body;
  const value = body?.match(new RegExp(`${token}:\\s*([^;]+);`))?.[1];
  expect(value, `${theme} ${token}`).toBeDefined();
  return value!;
};

const color = (
  value: string,
  theme?: string,
): [number, number, number] => {
  const hex = value.match(/^#([0-9a-f]{6})$/i);
  if (hex) {
    return [0, 2, 4].map((offset) => parseInt(hex[1].slice(offset, offset + 2), 16)) as [
      number,
      number,
      number,
    ];
  }

  const mix = value.match(
    /^color-mix\(in srgb, (.+) ([\d.]+)%, (.+)\)$/,
  );
  expect(mix, `supported color value: ${value}`).toBeDefined();
  const first = resolve(mix![1], theme);
  const second = resolve(mix![3], theme);
  const proportion = Number(mix![2]) / 100;
  return first.map(
    (channel, index) => channel * proportion + second[index] * (1 - proportion),
  ) as [number, number, number];
};

const resolve = (value: string, theme?: string): [number, number, number] => {
  const variable = value.match(/^var\((--[a-z0-9-]+)\)$/);
  if (variable) return resolve(tokenValue(theme!, variable[1]), theme);
  return color(value, theme);
};

const luminance = (rgb: [number, number, number]): number =>
  rgb.reduce((total, channel, index) => {
    const normalized = channel / 255;
    const linear =
      normalized <= 0.03928
        ? normalized / 12.92
        : ((normalized + 0.055) / 1.055) ** 2.4;
    return total + linear * [0.2126, 0.7152, 0.0722][index];
  }, 0);

const contrast = (
  foreground: [number, number, number],
  background: [number, number, number],
): number => {
  const foregroundLuminance = luminance(foreground);
  const backgroundLuminance = luminance(background);
  return (
    (Math.max(foregroundLuminance, backgroundLuminance) + 0.05) /
    (Math.min(foregroundLuminance, backgroundLuminance) + 0.05)
  );
};

describe("design/undefined-style-tokens", () => {
  it("defines the four previously missing roles for both installed themes", () => {
    for (const theme of ["dark", "light"]) {
      for (const token of [
        "--action-primary",
        "--status-warning",
        "--status-danger-deep",
        "--status-danger-pale",
      ]) {
        expect(tokenValue(theme, token)).toBeTruthy();
      }

      expect(tokenValue(theme, "--action-primary")).toBe(
        "var(--action-attention)",
      );
      expect(tokenValue(theme, "--status-warning")).toBe(
        "var(--action-attention)",
      );
    }
  });

  it("defines the shared pill radius for cross-surface consumers", () => {
    const root = tokenRules.find((rule) => rule.selector === ":root");
    expect(root?.body).toContain("--r-pill: 999px");
    expect(agentPane).not.toMatch(/--r-pill\s*:/);
    expect(screens).toContain(".inbox-throughput-meter {");
    expect(screens).toContain("border-radius: var(--r-pill)");
  });

  it("keeps warning text and danger-chip text at AA contrast in both themes", () => {
    for (const theme of ["dark", "light"]) {
      const dangerForeground = resolve(
        tokenValue(theme, "--status-danger-pale"),
        theme,
      );
      const dangerBackground = resolve(
        tokenValue(theme, "--status-danger-deep"),
        theme,
      );
      expect(
        contrast(dangerForeground, dangerBackground),
        `${theme} danger chip`,
      ).toBeGreaterThanOrEqual(4.5);

      const warningForeground = resolve(
        tokenValue(theme, "--status-warning"),
        theme,
      );
      const ground = resolve(tokenValue(theme, "--surface-ground"), theme);
      expect(
        contrast(warningForeground, ground),
        `${theme} warning text`,
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("uses the resolved semantic roles at every former undefined call site", () => {
    expect(screens).toContain("background: var(--status-danger-deep)");
    expect(screens).toContain("color: var(--status-danger-pale)");
    expect(manage).toContain(".edge-amber{color:var(--status-warning)}");
    expect(manage).toContain(
      "border-left:2px solid var(--status-warning)",
    );
    expect(extensionEditor).toContain("color:var(--action-primary)");
    expect(extensionEditor).toContain(
      "border-left-color:var(--action-primary)",
    );
  });
});
