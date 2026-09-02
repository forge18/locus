import { DESKTOP_ROUTES } from "./desktop-screen-inventory";

export const VIEWS = Object.freeze(DESKTOP_ROUTES.map((route) => route.id));
export type View = (typeof DESKTOP_ROUTES)[number]["id"];

export const CATEGORIES = Object.freeze([
  "projects",
  "workers",
  "telemetry",
  "plan",
  "manage",
  "review",
  "extensions",
  "plugins",
  "knowledge",
  "settings",
] as const);
export type Category = (typeof CATEGORIES)[number] | "pill" | (string & {});

const VIEW_CATEGORY = Object.fromEntries(
  DESKTOP_ROUTES.map((route) => [route.id, route.category]),
) as Record<View, Category>;

export function categoryOf(view: View): Category {
  return VIEW_CATEGORY[view];
}

export interface RailItem {
  category: Category;
  label: string;
  icon: string;
  firstView: View;
}

/** Production rail categories. Group headers are structural, not route items. */
export const RAIL_ITEMS: readonly RailItem[] = Object.freeze(
  [
    { category: "projects", label: "Projects", icon: "folder", firstView: "projects" },
    { category: "workers", label: "Workers", icon: "robot", firstView: "workers" },
    { category: "telemetry", label: "Telemetry", icon: "chart-bar", firstView: "telemetry" },
    { category: "plan", label: "Plan", icon: "compass", firstView: "plan" },
    { category: "manage", label: "Manage", icon: "kanban", firstView: "sessions" },
    { category: "review", label: "Review", icon: "check-square", firstView: "qa" },
    { category: "extensions", label: "Extensions", icon: "puzzle", firstView: "agents" },
    { category: "plugins", label: "Plugins", icon: "plug", firstView: "cli" },
    { category: "knowledge", label: "Knowledge", icon: "brain", firstView: "short" },
    { category: "settings", label: "Settings", icon: "sliders", firstView: "settings" },
  ].map((item) => Object.freeze(item)) as RailItem[],
);

export const CATEGORY_LABELS: Record<Category, string> = {
  projects: "Projects",
  workers: "Workers",
  telemetry: "Telemetry",
  plan: "Plan",
  manage: "Manage",
  review: "Review",
  extensions: "Extensions",
  plugins: "Plugins",
  knowledge: "Knowledge",
  settings: "Settings",
  pill: "Inbox",
};
