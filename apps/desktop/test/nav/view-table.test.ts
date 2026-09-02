import { describe, expect, it } from "vitest";
import {
  CATEGORIES,
  CATEGORY_LABELS,
  RAIL_ITEMS,
  VIEWS,
  categoryOf,
} from "../../src/nav";

const TABLE: Array<[string, string, string]> = [
  ["inbox", "pill", "Inbox"],
  ["status", "telemetry", "Telemetry"],
  ["telemetry", "telemetry", "Telemetry"],
  ["mail", "telemetry", "Telemetry"],
  ["projects", "projects", "Projects"],
  ["workers", "workers", "Workers"],
  ["plan", "plan", "Plan"],
  ["sessions", "manage", "Manage"],
  ["qa", "review", "Review"],
  ["autorun", "pill", "Inbox"],
  ["schedule", "pill", "Inbox"],
  ["runs", "pill", "Inbox"],
  ["short", "knowledge", "Knowledge"],
  ["memory", "knowledge", "Knowledge"],
  ["artifact", "knowledge", "Knowledge"],
  ["wiki", "knowledge", "Knowledge"],
  ["settings", "settings", "Settings"],
  ["agents", "extensions", "Extensions"],
  ["cli", "plugins", "Plugins"],
  ["commands", "extensions", "Extensions"],
  ["harnesses", "plugins", "Plugins"],
  ["hooks", "extensions", "Extensions"],
  ["linters", "extensions", "Extensions"],
  ["styles", "extensions", "Extensions"],
  ["providers", "plugins", "Plugins"],
  ["rules", "extensions", "Extensions"],
  ["skills", "extensions", "Extensions"],
  ["canvas", "extensions", "Extensions"],
  ["workflows", "extensions", "Extensions"],
];

describe("nav/view-table", () => {
  it("holds every production view", () => {
    expect([...VIEWS].sort()).toEqual(TABLE.map(([v]) => v).sort());
  });

  it("maps each view to its category and rail label", () => {
    for (const [view, category, label] of TABLE) {
      expect(categoryOf(view as never), view).toBe(category);
      expect(CATEGORY_LABELS[category as never], view).toBe(label);
    }
  });

  it("has one rail item per category, in rail order", () => {
    expect(RAIL_ITEMS.map((r) => r.category)).toEqual([...CATEGORIES]);
  });

  it("gives each rail item a Phosphor glyph and a first view", () => {
    for (const item of RAIL_ITEMS) {
      expect(item.icon, item.category).toMatch(/^[a-z-]+$/);
      expect(categoryOf(item.firstView), item.category).toBe(item.category);
    }
  });

  it("is one exported constant, not a per-component copy", () => {
    expect(new Set(RAIL_ITEMS.map((r) => r.category)).size).toBe(
      CATEGORIES.length,
    );
  });
});
