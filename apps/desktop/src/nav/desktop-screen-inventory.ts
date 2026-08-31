// schema: core.projects + core.settings
// replaced by: invoke("desktop_route_inventory")

export type DesktopRouteScope = "all" | "app" | "project" | (string & {});

export interface DesktopRoute {
  /** Stable route identity consumed by the shared desktop resolver. */
  id: string;
  label: string;
  scope: DesktopRouteScope;
  category: string;
  screen: string;
  screenshot: string;
}

function desktopRoute(
  id: string,
  label: string,
  scope: DesktopRouteScope,
  category: string,
): DesktopRoute {
  return Object.freeze({
    id,
    label,
    scope,
    category,
    screen: id,
    screenshot: `${id}.png`,
  });
}

/** The current 30-view desktop inventory. Order is the palette and rail order. */
export const DESKTOP_ROUTES = Object.freeze([
  desktopRoute("inbox", "Inbox", "all", "pill"),
  desktopRoute("status", "Status", "all", "analytics"),
  desktopRoute("telemetry", "Telemetry", "all", "analytics"),
  desktopRoute("mail", "Mail", "all", "analytics"),
  desktopRoute("projects", "Projects", "project", "setup"),
  desktopRoute("plan", "Plan", "project", "plan"),
  desktopRoute("sessions", "Sessions", "project", "manage"),
  desktopRoute("interact", "Interact", "project", "interact"),
  desktopRoute("bots", "Bots", "project", "bots"),
  desktopRoute("qa", "QA", "project", "review"),
  desktopRoute("autorun", "Autorun", "all", "pill"),
  desktopRoute("schedule", "Schedules", "all", "pill"),
  desktopRoute("runs", "Runs", "all", "pill"),
  desktopRoute("short", "Short-term", "project", "memory"),
  desktopRoute("memory", "Long-term", "project", "memory"),
  desktopRoute("artifact", "Artifacts", "project", "memory"),
  desktopRoute("wiki", "Wiki", "project", "memory"),
  desktopRoute("settings", "Settings", "app", "settings"),
  desktopRoute("agents", "Agents", "app", "workshop"),
  desktopRoute("cli", "CLI", "app", "workshop"),
  desktopRoute("commands", "Commands", "app", "workshop"),
  desktopRoute("harnesses", "Harnesses", "app", "workshop"),
  desktopRoute("hooks", "Hooks", "app", "workshop"),
  desktopRoute("linters", "Linters", "app", "workshop"),
  desktopRoute("styles", "Output styles", "app", "workshop"),
  desktopRoute("providers", "Providers", "app", "workshop"),
  desktopRoute("rules", "Rules", "app", "workshop"),
  desktopRoute("skills", "Skills", "app", "workshop"),
  desktopRoute("canvas", "Canvas", "app", "workshop"),
  desktopRoute("workflows", "Workflows", "app", "workshop"),
] as const);
