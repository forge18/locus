import { categoryOf } from "./views";
import type { Category, View } from "./views";

export interface CategoryTab {
  view: View;
  label: string;
}

const CATEGORY_TABS: Record<Category, CategoryTab[]> = {
  projects: [],
  workers: [],
  telemetry: [
    { view: "status", label: "Overview" },
    { view: "telemetry", label: "Telemetry" },
    { view: "mail", label: "Mail" },
  ],
  plan: [],
  manage: [{ view: "sessions", label: "List" }],
  review: [],
  extensions: [
    { view: "agents", label: "Agents" },
    { view: "commands", label: "Commands" },
    { view: "hooks", label: "Hooks" },
    { view: "linters", label: "Linters" },
    { view: "styles", label: "Output styles" },
    { view: "rules", label: "Rules" },
    { view: "skills", label: "Skills" },
    { view: "canvas", label: "Canvas" },
    { view: "workflows", label: "Workflows" },
  ],
  plugins: [
    { view: "cli", label: "CLI tools" },
    { view: "harnesses", label: "Harnesses" },
    { view: "providers", label: "Providers" },
  ],
  knowledge: [
    { view: "short", label: "Short-term" },
    { view: "memory", label: "Long-term" },
    { view: "artifact", label: "Artifacts" },
    { view: "wiki", label: "Wiki" },
  ],
  settings: [],
  pill: [],
};

export function tabsFor(category: Category): CategoryTab[] {
  return CATEGORY_TABS[category];
}
export function activeTabFor(view: View): View | null {
  const tabs = tabsFor(categoryOf(view));
  return tabs.some((tab) => tab.view === view) ? view : null;
}

const DRILLDOWN_PARENTS: Partial<Record<View, View>> = {
  canvas: "workflows",
};

export function drilldownParent(view: View): View | null {
  return DRILLDOWN_PARENTS[view] ?? null;
}
