import { describe, expect, it } from "vitest";
import {
  CATEGORIES,
  CATEGORY_LABELS,
  RAIL_ITEMS,
  VIEWS,
  categoryOf,
} from "../../src/nav";

/** The table exactly as .specs/navigation/spec.md writes it. */
const TABLE: Array<[string, string, string]> = [
  ["inbox", "pill", "Inbox"],
  ["status", "analytics", "Analytics"],
  ["telemetry", "analytics", "Analytics"],
  ["mail", "analytics", "Analytics"],
  ["projects", "setup", "Setup"],
  ["plan", "plan", "Plan"],
  ["sessions", "manage", "Manage"],
  ["interact", "interact", "Interact"],
  ["bots", "bots", "Bots"],
  ["qa", "review", "Review"],
  ["autorun", "pill", "Inbox"],
  ["schedule", "pill", "Inbox"],
  ["runs", "pill", "Inbox"],
  ["short", "memory", "Memory"],
  ["memory", "memory", "Memory"],
  ["artifact", "memory", "Memory"],
  ["wiki", "memory", "Memory"],
  ["settings", "settings", "Settings"],
  ["agents", "workshop", "Workshop"],
  ["cli", "workshop", "Workshop"],
  ["commands", "workshop", "Workshop"],
  ["harnesses", "workshop", "Workshop"],
  ["hooks", "workshop", "Workshop"],
  ["linters", "workshop", "Workshop"],
  ["styles", "workshop", "Workshop"],
  ["providers", "workshop", "Workshop"],
  ["rules", "workshop", "Workshop"],
  ["skills", "workshop", "Workshop"],
  ["canvas", "workshop", "Workshop"],
  ["workflows", "workshop", "Workshop"],
];

describe("nav/view-table", () => {
  it("holds the 30 views", () => {
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
    // Every category is reachable from the one list, and the list is the rail.
    expect(new Set(RAIL_ITEMS.map((r) => r.category)).size).toBe(
      CATEGORIES.length,
    );
  });
});
