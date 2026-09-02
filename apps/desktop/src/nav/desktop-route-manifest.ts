// Production route authority. Demo inventories derive from this manifest.

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

/** Current production routes. Order is the palette and rail order. */
export const DESKTOP_ROUTES = Object.freeze([
  desktopRoute("inbox", "Inbox", "all", "pill"),
  desktopRoute("status", "Status", "all", "telemetry"),
  desktopRoute("telemetry", "Telemetry", "all", "telemetry"),
  desktopRoute("mail", "Mail", "all", "telemetry"),
  desktopRoute("projects", "Projects", "all", "projects"),
  desktopRoute("workers", "Workers", "all", "workers"),
  desktopRoute("plan", "Plan", "all", "plan"),
  desktopRoute("sessions", "Sessions", "all", "manage"),
  desktopRoute("qa", "QA", "all", "review"),
  desktopRoute("autorun", "Autorun", "all", "pill"),
  desktopRoute("schedule", "Schedules", "all", "pill"),
  desktopRoute("runs", "Runs", "all", "pill"),
  desktopRoute("short", "Short-term", "all", "knowledge"),
  desktopRoute("memory", "Long-term", "all", "knowledge"),
  desktopRoute("artifact", "Artifacts", "all", "knowledge"),
  desktopRoute("wiki", "Wiki", "all", "knowledge"),
  desktopRoute("settings", "Settings", "app", "settings"),
  desktopRoute("agents", "Agents", "app", "extensions"),
  desktopRoute("cli", "CLI", "app", "plugins"),
  desktopRoute("commands", "Commands", "app", "extensions"),
  desktopRoute("harnesses", "Harnesses", "app", "plugins"),
  desktopRoute("hooks", "Hooks", "app", "extensions"),
  desktopRoute("linters", "Linters", "app", "extensions"),
  desktopRoute("styles", "Output styles", "app", "extensions"),
  desktopRoute("providers", "Providers", "app", "plugins"),
  desktopRoute("rules", "Rules", "app", "extensions"),
  desktopRoute("skills", "Skills", "app", "extensions"),
  desktopRoute("canvas", "Canvas", "app", "extensions"),
  desktopRoute("workflows", "Workflows", "app", "extensions"),
] as const);
