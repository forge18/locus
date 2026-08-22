import { describe, expect, it } from "vitest";
import { INSTALLED_THEMES } from "../../src/styles/theme";
import { read, rules } from "../css";

const themeRules = rules(read("styles/tokens.css"));

describe("theme/all-installed", () => {
  it("declares one complete token value set for every installed theme", () => {
    for (const theme of INSTALLED_THEMES) {
      const rule = themeRules.find(
        (candidate) => candidate.selector === `[data-theme='${theme}']`,
      );
      expect(rule, `${theme} selector`).toBeDefined();
      for (const token of [
        "--surface-ground:",
        "--surface-raised:",
        "--text-primary:",
        "--action-attention:",
        "--status-working:",
      ]) {
        expect(rule!.body, `${theme} ${token}`).toContain(token);
      }
    }
  });
});
