import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";

it("agent-panel/responsive preserves the composer and collapses research at narrow widths", () => {
  const css = readFileSync(resolve(process.cwd(), "src/panes/agent-pane.css"), "utf8");
  expect(css).toContain("@media (max-width: 1100px)");
  expect(css).toContain("@media (max-width: 520px)");
  expect(css).toContain("min-width: 520px");
  expect(css).toContain("flex: none");
});
