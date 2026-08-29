import { categoryOf } from "./views";
import type { Category, View } from "./views";

export interface CategoryTab {
  view: View;
  label: string;
}

const CATEGORY_TABS: Record<Category, CategoryTab[]> = {
  setup: [],
  plan: [],
  manage: [{ view: "sessions", label: "List" }],
  interact: [],
  bots: [],
  review: [],
  analytics: [
    { view: "status", label: "Overview" },
    { view: "telemetry", label: "Telemetry" },
  ],
  memory: [
    { view: "short", label: "Short-term" },
    { view: "memory", label: "Long-term" },
    { view: "artifact", label: "Artifacts" },
    { view: "wiki", label: "Wiki" },
  ],
  settings: [],
  workshop: [],
  pill: [],
};

export function tabsFor(category: Category): CategoryTab[] {
  return CATEGORY_TABS[category];
}
export function activeTabFor(view: View): View | null {
  const tabs = tabsFor(categoryOf(view));
  return tabs.some((tab) => tab.view === view) ? view : null;
}

/**
 * The view a drill-down was entered from, or null. The canvas is the one current
 * drill-down: it renders a single workflow graph, appears in no rail list, and
 * the way out is the Workflows list it was opened from. Views the rail or a pill
 * lands on directly are not drill-downs, however object-like their params read —
 * `agents` has been the Workshop landing view since the M0.7 shell revision, so
 * it has no Extensions view to go back to.
 */
const DRILLDOWN_PARENTS: Partial<Record<View, View>> = {
  canvas: "workflows",
};

export function drilldownParent(view: View): View | null {
  return DRILLDOWN_PARENTS[view] ?? null;
}
