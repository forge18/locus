import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ExtensionEditor } from "../../src/screens/workshop/ExtensionEditor";

describe("workshop/frontmatter-table", () => {
  it("exposes the frontmatter grid as a table with named columns", () => {
    const { getByTestId } = render(() => <ExtensionEditor type="skills" />);
    const table = getByTestId("frontmatter").querySelector('[role="table"]');
    expect(table?.getAttribute("aria-label")).toBe("Frontmatter fields");
    expect(table?.querySelectorAll('[role="columnheader"]')).toHaveLength(3);
    expect(table?.querySelectorAll('[role="row"]').length).toBeGreaterThan(1);
  });

  it("splits every field row into three cells a reader can walk", () => {
    const { getByTestId } = render(() => <ExtensionEditor type="skills" />);
    const rows = getByTestId("frontmatter").querySelectorAll(
      '[role="row"][data-testid]',
    );
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) {
      expect(row.querySelectorAll('[role="cell"]')).toHaveLength(3);
    }
  });
});
