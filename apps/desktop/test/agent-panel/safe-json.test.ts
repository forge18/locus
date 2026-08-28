import { expect, it } from "vitest";
import { safeJson } from "../../src/panes/agent-pane-utils";

it("agent-panel/safe-json redacts secret-like tool arguments", () => {
  const rendered = safeJson({ apiKey: "secret", nested: { password: "secret" }, path: "src/lib.rs" });
  expect(rendered).toContain("[redacted]");
  expect(rendered).not.toContain('"secret"');
  expect(rendered).toContain("src/lib.rs");
});
