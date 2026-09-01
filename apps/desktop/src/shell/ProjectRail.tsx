import { createSignal, For } from "solid-js";
import { destinationDesktop } from "../nav/desktop-navigation";
import type { DesktopRouteId } from "../nav/desktop-locator";
import {
      Desktop_APP_ROUTE_KINDS,
      Desktop_PROJECT_ROUTE_KINDS,
} from "../nav/desktop-route-kinds";

export const RAIL_EXPANSION_STORAGE_KEY = "locus.rail-expansion";

const PRIMARY_LINKS = [
      ["Projects", "projects"],
      ["Workers", "workers"],
      ["Telemetry", "telemetry"],
] as const;
const AUTOMATION_LINKS = [
      ["Plan", "plan"],
      ["Manage", "sessions"],
      ["Review", "qa"],
] as const;
const MEMORY_ROUTES = Desktop_PROJECT_ROUTE_KINDS.filter(
      (route) => route.category === "knowledge",
);
const WORKSHOP_ROUTES = Desktop_APP_ROUTE_KINDS.filter(
      (route) => route.category === "plugins" || route.category === "extensions",
);
const WORKSHOP_PLUGIN_ROUTES = WORKSHOP_ROUTES.filter((route) =>
      ["cli", "harnesses", "providers"].includes(route.id),
);
const WORKSHOP_EXTENSION_LINKS = [
      ["Agents", "agents"],
      ["Commands", "commands"],
      ["Context", "projects"],
      ["Hooks", "hooks"],
      ["Linters", "linters"],
      ["Output styles", "styles"],
      ["Rules", "rules"],
      ["Skills", "skills"],
      ["Workflows", "workflows"],
] as const;

const readExpansion = (): Record<string, boolean> => {
      try {
            const value: unknown = JSON.parse(
                  localStorage.getItem(RAIL_EXPANSION_STORAGE_KEY) ?? "{}",
            );
            return typeof value === "object" &&
                  value !== null &&
                  !Array.isArray(value)
                  ? (value as Record<string, boolean>)
                  : {};
      } catch {
            return {};
      }
};

export interface ProjectRailProps {
      /** Used only by page-owned project filters and project object links. */
      selectedProject: string;
      inboxCount?: number;
      dispatchState?: "ready" | "working" | "blocked";
      /** Deprecated compatibility inputs; scope is no longer shell-owned. */
      projects?: readonly string[];
      projectDetails?: readonly {
            id: string;
            running: number;
            spend: string;
      }[];
      onNavigate?: (locator: string) => void;
}

function routeLocator(route: DesktopRouteId, project: string): string {
      return route === "sessions" ||
            route === "qa" ||
            route === "short" ||
            route === "memory" ||
            route === "artifact" ||
            route === "wiki"
            ? destinationDesktop(route, project)
            : destinationDesktop(route);
}

function WorkshopRailLinks(
      props: Pick<ProjectRailProps, "selectedProject" | "onNavigate"> & {
            hidden: boolean;
      },
) {
      const pluginLabel = (id: string) =>
            id === "cli" ? "CLI Tool" : id === "harnesses" ? "Harness" : "Provider";

      return (
            <div data-testid="workshop-rail-links" hidden={props.hidden}>
                  <section data-testid="workshop-plugins-group">
                        <div class="rail-subgroup-label">Plugins</div>
                        <div data-testid="workshop-plugin-links">
                              <For each={WORKSHOP_PLUGIN_ROUTES}>
                                    {(route) => {
                                          const locator = routeLocator(
                                                route.id,
                                                props.selectedProject,
                                          );
                                          return (
                                                <button
                                                      type="button"
                                                      data-locator={locator}
                                                      onClick={() =>
                                                            props.onNavigate?.(locator)
                                                      }
                                                >
                                                      {pluginLabel(route.id)}
                                                </button>
                                          );
                                    }}
                              </For>
                        </div>
                  </section>
                  <section data-testid="workshop-extensions-group">
                        <div class="rail-subgroup-label">Extensions</div>
                        <div data-testid="workshop-extension-links">
                              <For each={WORKSHOP_EXTENSION_LINKS}>
                                    {(link) => {
                                          const locator = routeLocator(
                                                link[1],
                                                props.selectedProject,
                                          );
                                          return (
                                                <button
                                                      type="button"
                                                      data-locator={locator}
                                                      onClick={() =>
                                                            props.onNavigate?.(locator)
                                                      }
                                                >
                                                      {link[0]}
                                                </button>
                                          );
                                    }}
                              </For>
                        </div>
                  </section>
            </div>
      );
}

export function ProjectRail(props: ProjectRailProps) {
      const saved = readExpansion();
      const [memoryExpanded, setMemoryExpanded] = createSignal(
            saved.memory ?? false,
      );
      const [workshopExpanded, setWorkshopExpanded] = createSignal(
            saved.workshop ?? false,
      );
      const persist = (name: "memory" | "workshop", value: boolean) =>
            localStorage.setItem(
                  RAIL_EXPANSION_STORAGE_KEY,
                  JSON.stringify({ ...readExpansion(), [name]: value }),
            );
      const navigate = (route: DesktopRouteId) =>
            props.onNavigate?.(routeLocator(route, props.selectedProject));

      return (
            <nav
                  aria-label="Application navigation"
                  class="project-rail"
                  data-testid="project-rail"
            >
                  <section data-testid="primary-group">
                        <div class="rail-group-label">Projects</div>
                        <For each={PRIMARY_LINKS}>
                              {([label, route]) => (
                                    <button
                                          type="button"
                                          onClick={() => navigate(route)}
                                    >
                                          {label}
                                    </button>
                              )}
                        </For>
                  </section>

                  <section data-testid="automation-group">
                        <div class="rail-group-label">Automation</div>
                        <For each={AUTOMATION_LINKS}>
                              {([label, route]) => (
                                    <button
                                          type="button"
                                          onClick={() => navigate(route)}
                                    >
                                          {label}
                                    </button>
                              )}
                        </For>
                  </section>

                  <section data-testid="workshop-group">
                        <div class="rail-group-label">Workshop</div>
                        <button
                              type="button"
                              aria-expanded={memoryExpanded()}
                              onClick={() => {
                                    navigate("short");
                                    const next = !memoryExpanded();
                                    setMemoryExpanded(next);
                                    persist("memory", next);
                              }}
                        >
                              Knowledge
                        </button>
                        <div data-testid="memory-rail-links" hidden={!memoryExpanded()}>
                              <For each={MEMORY_ROUTES}>
                                    {(route) => (
                                          <button
                                                type="button"
                                                onClick={() => navigate(route.id)}
                                          >
                                                {route.label}
                                          </button>
                                    )}
                              </For>
                        </div>
                        <button type="button" onClick={() => navigate("settings")}>
                              Settings
                        </button>
                        <button
                              type="button"
                              aria-expanded={workshopExpanded()}
                              onClick={() => {
                                    navigate("agents");
                                    const next = !workshopExpanded();
                                    setWorkshopExpanded(next);
                                    persist("workshop", next);
                              }}
                        >
                              Extensions / Plugins
                        </button>
                        <WorkshopRailLinks
                              hidden={!workshopExpanded()}
                              selectedProject={props.selectedProject}
                              onNavigate={props.onNavigate}
                        />
                  </section>
            </nav>
      );
}
