import { describe, expect, it } from "vitest";
import { read, rules } from "../css";

const tokens = read("styles/tokens.css");
const light = rules(tokens).find(
  (rule) => rule.selector === "[data-theme='light']",
);

describe("theme/light-token-contract", () => {
  it("defines the cool-neutral Light semantic palette under the light theme selector", () => {
    expect(light).toBeDefined();

    for (const token of [
      "--surface-ground: #f3f6f8",
      "--surface-chrome: #e8eef3",
      "--surface-raised: #ffffff",
      "--surface-selected: #e3edf5",
      "--text-primary: #16212b",
      "--text-secondary: #405262",
      "--action-attention: #9a5b00",
      "--status-working: #675bb0",
      "--status-success: #237250",
      "--status-danger: #a7372d",
      "--data-1: #5f85ad",
      "--data-2: #5980b4",
      "--data-3: #4671ad",
      "--data-hi: #315f9a",
    ]) {
      expect(light!.body).toContain(token);
    }
  });

  it("keeps the v2 aliases bound to the Light semantic roles", () => {
    for (const token of [
      "--bg: var(--surface-ground)",
      "--bg-deep: var(--surface-chrome)",
      "--sf: var(--surface-raised)",
      "--sf2: var(--surface-selected)",
      "--tx: var(--text-primary)",
      "--ac: var(--action-attention)",
      "--ac2: var(--status-working)",
      "--ok: var(--status-success)",
      "--bad: var(--status-danger)",
    ]) {
      expect(light!.body).toContain(token);
    }
  });

  it("keeps geometry and typography theme-independent on the document root", () => {
    const root = rules(tokens).find((rule) => rule.selector === ":root");
    expect(root?.body).toContain("--r-card: 7px");
    expect(root?.body).toContain("--t-body: 14px");
    expect(root?.body).toContain("--fs: 'Inter'");
  });
});
