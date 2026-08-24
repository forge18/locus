import { Desktop_FIXTURE_ROUTES } from "../fixtures/desktop-screen-inventory";

export const VIEWS = Object.freeze(
  Desktop_FIXTURE_ROUTES.map((route) => route.id),
);
export type View = (typeof Desktop_FIXTURE_ROUTES)[number]["id"];

export const CATEGORIES = Object.freeze([
  "setup",
  "plan",
  "manage",
  "interact",
  "review",
  "analytics",
  "memory",
  "settings",
  "workshop",
] as const);
export type Category = (typeof CATEGORIES)[number] | "pill" | (string & {});

const VIEW_CATEGORY = Object.fromEntries(
  Desktop_FIXTURE_ROUTES.map((route) => [route.id, route.category]),
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

/** Project and cross-project rail categories. Pill views intentionally have no rail item. */
export const RAIL_ITEMS: readonly RailItem[] = Object.freeze(
  [
    { category: "setup", label: "Setup", icon: "gear", firstView: "projects" },
    { category: "plan", label: "Plan", icon: "compass", firstView: "plan" },
    {
      category: "manage",
      label: "Manage",
      icon: "kanban",
      firstView: "sessions",
    },
    {
      category: "interact",
      label: "Interact",
      icon: "chat-circle",
      firstView: "interact",
    },
    {
      category: "review",
      label: "Review",
      icon: "check-square",
      firstView: "qa",
    },
    {
      category: "analytics",
      label: "Analytics",
      icon: "chart-bar",
      firstView: "status",
    },
    { category: "memory", label: "Memory", icon: "brain", firstView: "short" },
    {
      category: "settings",
      label: "Settings",
      icon: "sliders",
      firstView: "settings",
    },
    {
      category: "workshop",
      label: "Workshop",
      icon: "wrench",
      firstView: "agents",
    },
  ].map((item) => Object.freeze(item)) as RailItem[],
);

export const CATEGORY_LABELS: Record<Category, string> = {
  setup: "Setup",
  plan: "Plan",
  manage: "Manage",
  interact: "Interact",
  review: "Review",
  analytics: "Analytics",
  memory: "Memory",
  settings: "Settings",
  workshop: "Workshop",
  pill: "Inbox",
};
