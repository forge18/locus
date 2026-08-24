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
export function drilldownParent(_view: View): View | null {
  return null;
}
