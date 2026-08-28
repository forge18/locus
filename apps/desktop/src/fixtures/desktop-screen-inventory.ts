export type FixtureScope = "all" | "app" | "project" | (string & {});

export interface DesktopFixtureRoute {
  /** Stable route identity consumed by the shared desktop resolver. */
  id: string;
  label: string;
  scope: FixtureScope;
  category: string;
  screen: string;
  screenshot: string;
}

function fixtureRoute(
  id: string,
  label: string,
  scope: FixtureScope,
  category: string,
): DesktopFixtureRoute {
  return Object.freeze({
    id,
    label,
    scope,
    category,
    screen: id,
    screenshot: `${id}.png`,
  });
}

/** The current 30-view desktop inventory. Order is the palette and fixture order. */
export const Desktop_FIXTURE_ROUTES = Object.freeze([
  fixtureRoute("inbox", "Inbox", "all", "pill"),
  fixtureRoute("status", "Status", "all", "analytics"),
  fixtureRoute("telemetry", "Telemetry", "all", "analytics"),
  fixtureRoute("mail", "Mail", "all", "analytics"),
  fixtureRoute("projects", "Projects", "project", "setup"),
  fixtureRoute("plan", "Plan", "project", "plan"),
  fixtureRoute("sessions", "Sessions", "project", "manage"),
  fixtureRoute("interact", "Interact", "project", "interact"),
  fixtureRoute("bots", "Bots", "project", "bots"),
  fixtureRoute("qa", "QA", "project", "review"),
  fixtureRoute("autorun", "Autorun", "all", "pill"),
  fixtureRoute("schedule", "Schedules", "all", "pill"),
  fixtureRoute("runs", "Runs", "all", "pill"),
  fixtureRoute("short", "Short-term", "all", "memory"),
  fixtureRoute("memory", "Long-term", "all", "memory"),
  fixtureRoute("artifact", "Artifacts", "all", "memory"),
  fixtureRoute("wiki", "Wiki", "all", "memory"),
  fixtureRoute("settings", "Settings", "app", "settings"),
  fixtureRoute("agents", "Agents", "app", "workshop"),
  fixtureRoute("cli", "CLI", "app", "workshop"),
  fixtureRoute("commands", "Commands", "app", "workshop"),
  fixtureRoute("harnesses", "Harnesses", "app", "workshop"),
  fixtureRoute("hooks", "Hooks", "app", "workshop"),
  fixtureRoute("linters", "Linters", "app", "workshop"),
  fixtureRoute("styles", "Output styles", "app", "workshop"),
  fixtureRoute("providers", "Providers", "app", "workshop"),
  fixtureRoute("rules", "Rules", "app", "workshop"),
  fixtureRoute("skills", "Skills", "app", "workshop"),
  fixtureRoute("canvas", "Canvas", "app", "workshop"),
  fixtureRoute("workflows", "Workflows", "app", "workshop"),
] as const);
