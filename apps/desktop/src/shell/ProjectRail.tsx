import { createMemo, createSignal, For } from "solid-js";
import { destinationDesktop } from "../nav/desktop-navigation";
import {
      Desktop_ALL_ROUTE_KINDS,
      Desktop_APP_ROUTE_KINDS,
} from "../nav/desktop-route-kinds";

export const RAIL_EXPANSION_STORAGE_KEY = "locus.rail-expansion";

const PROJECT_RAIL_LINKS = [
      ["Setup", "projects"],
      ["Plan", "plan"],
      ["Manage", "sessions"],
      ["Interact", "interact"],
      ["Bots", "bots"],
      ["Review", "qa"],
] as const;
const CROSS_PROJECT_LINKS = [["Analytics", "status"]] as const;
const MEMORY_ROUTES = Desktop_ALL_ROUTE_KINDS.filter(
      (route) => route.category === "memory",
);
const WORKSHOP_ROUTES = Desktop_APP_ROUTE_KINDS.filter(
      (route) => route.category === "workshop",
);
const WORKSHOP_PLUGIN_ROUTES = WORKSHOP_ROUTES.filter((route) =>
      ["cli", "harnesses", "providers"].includes(route.id),
);
const WORKSHOP_EXTENSION_LINKS = [
      ["Agents", "agents"],
      ["Commands", "commands"],
      ["Base context", "projects"],
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
      selectedProject: string;
      inboxCount?: number;
      dispatchState?: "ready" | "working" | "blocked";
      projects?: readonly string[];
      projectDetails?: readonly {
            id: string;
            running: number;
            spend: string;
      }[];
      onNavigate?: (locator: string) => void;
}

function WorkshopRailLinks(
      props: Pick<ProjectRailProps, "selectedProject" | "onNavigate"> & {
            hidden: boolean;
      },
) {
      const pluginLabel = (id: string) =>
            id === "cli"
                  ? "CLI Tool"
                  : id === "harnesses"
                    ? "Harness"
                    : "Provider";
      const extensionLocator = (
            route: (typeof WORKSHOP_EXTENSION_LINKS)[number][1],
      ) =>
            route === "projects"
                  ? destinationDesktop(route, props.selectedProject)
                  : destinationDesktop(route);

      return (
            <div data-testid="workshop-rail-links" hidden={props.hidden}>
                  <section data-testid="workshop-plugins-group">
                        <div class="rail-subgroup-label">Plugins</div>
                        <div data-testid="workshop-plugin-links">
                              <For each={WORKSHOP_PLUGIN_ROUTES}>
                                    {(route) => {
                                          const locator = destinationDesktop(
                                                route.id,
                                          );
                                          return (
                                                <button
                                                      type="button"
                                                      data-locator={locator}
                                                      onClick={() =>
                                                            props.onNavigate?.(
                                                                  locator,
                                                            )
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
                                          const locator = extensionLocator(
                                                link[1],
                                          );
                                          return (
                                                <button
                                                      type="button"
                                                      data-locator={locator}
                                                      onClick={() =>
                                                            props.onNavigate?.(
                                                                  locator,
                                                            )
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
      const [filter, setFilter] = createSignal("");
      const [activeProject, setActiveProject] = createSignal(0);
      const [memoryExpanded, setMemoryExpanded] = createSignal(
            saved.memory ?? false,
      );
      const [workshopExpanded, setWorkshopExpanded] = createSignal(
            saved.workshop ?? false,
      );
      const projects = createMemo(() => {
            const needle = filter().trim().toLowerCase();
            return (props.projects ?? [props.selectedProject]).filter(
                  (project) => project.toLowerCase().includes(needle),
            );
      });
      const persist = (name: "memory" | "workshop", value: boolean) =>
            localStorage.setItem(
                  RAIL_EXPANSION_STORAGE_KEY,
                  JSON.stringify({ ...readExpansion(), [name]: value }),
            );
      const moveProject = (delta: 1 | -1) => {
            const count = projects().length;
            if (count)
                  setActiveProject((index) => (index + delta + count) % count);
      };

      return (
            <nav
                  aria-label="Application navigation"
                  class="project-rail"
                  data-testid="project-rail"
            >
                  <section data-testid="project-group">
                        <div class="rail-group-label">Project</div>
                        <button
                              type="button"
                              class="project-switcher"
                              data-testid="project-switcher"
                        >
                              #{props.selectedProject}
                        </button>
                        <input
                              type="search"
                              data-testid="project-switcher-filter"
                              aria-label="Filter projects"
                              value={filter()}
                              onInput={(event) => {
                                    setFilter(event.currentTarget.value);
                                    setActiveProject(0);
                              }}
                              onKeyDown={(event) => {
                                    if (event.key === "ArrowDown") {
                                          event.preventDefault();
                                          moveProject(1);
                                    }
                                    if (event.key === "ArrowUp") {
                                          event.preventDefault();
                                          moveProject(-1);
                                    }
                              }}
                        />
                        <div data-testid="project-switcher-results">
                              <For each={projects()}>
                                    {(project, index) => {
                                          const detail = () =>
                                                props.projectDetails?.find(
                                                      (entry) =>
                                                            entry.id ===
                                                            project,
                                                );
                                          const selected = () =>
                                                project ===
                                                props.selectedProject;
                                          return (
                                                <button
                                                      type="button"
                                                      data-testid={`project-switcher-option-${project}`}
                                                      data-selected-project={
                                                            selected()
                                                                  ? "true"
                                                                  : "false"
                                                      }
                                                      data-project-running={
                                                            detail()?.running ??
                                                            0
                                                      }
                                                      data-project-spend={
                                                            detail()?.spend ??
                                                            ""
                                                      }
                                                      aria-selected={
                                                            activeProject() ===
                                                            index()
                                                      }
                                                      aria-current={
                                                            selected()
                                                                  ? "true"
                                                                  : undefined
                                                      }
                                                      onClick={() =>
                                                            props.onNavigate?.(
                                                                  destinationDesktop(
                                                                        "projects",
                                                                        project,
                                                                  ),
                                                            )
                                                      }
                                                >
                                                      <span>{project}</span>
                                                      <small
                                                            data-testid={`project-meta-${project}`}
                                                      >
                                                            {detail()
                                                                  ? `${detail()!.running} running · ${detail()!.spend}`
                                                                  : selected()
                                                                    ? "selected"
                                                                    : ""}
                                                      </small>
                                                </button>
                                          );
                                    }}
                              </For>
                        </div>
                        <button
                              type="button"
                              class="new-project"
                              onClick={() =>
                                    props.onNavigate?.(
                                          destinationDesktop(
                                                "projects",
                                                props.selectedProject,
                                          ),
                                    )
                              }
                        >
                              + New project
                        </button>
                        <div data-testid="project-rail-routes">
                              <For each={PROJECT_RAIL_LINKS}>
                                    {([label, route]) => (
                                          <button
                                                type="button"
                                                onClick={() =>
                                                      props.onNavigate?.(
                                                            destinationDesktop(
                                                                  route,
                                                                  props.selectedProject,
                                                            ),
                                                      )
                                                }
                                          >
                                                {label}
                                          </button>
                                    )}
                              </For>
                        </div>
                        <span
                              data-testid="dispatch-dot"
                              data-state={props.dispatchState ?? "ready"}
                              aria-label={`Dispatch ${props.dispatchState ?? "ready"}`}
                        />
                  </section>

                  <section data-testid="cross-project-group">
                        <div class="rail-group-label">Cross-Project</div>
                        <For each={CROSS_PROJECT_LINKS}>
                              {([label, route]) => (
                                    <button
                                          type="button"
                                          onClick={() =>
                                                props.onNavigate?.(
                                                      destinationDesktop(route),
                                                )
                                          }
                                    >
                                          {label}
                                    </button>
                              )}
                        </For>
                        <button
                              type="button"
                              aria-expanded={memoryExpanded()}
                              onClick={() => {
                                    props.onNavigate?.(
                                          destinationDesktop("short"),
                                    );
                                    const next = !memoryExpanded();
                                    setMemoryExpanded(next);
                                    persist("memory", next);
                              }}
                        >
                              Memory
                        </button>
                        <div
                              data-testid="memory-rail-links"
                              hidden={!memoryExpanded()}
                        >
                              <For each={MEMORY_ROUTES}>
                                    {(route) => (
                                          <button
                                                type="button"
                                                onClick={() =>
                                                      props.onNavigate?.(
                                                            destinationDesktop(
                                                                  route.id,
                                                            ),
                                                      )
                                                }
                                          >
                                                {route.label}
                                          </button>
                                    )}
                              </For>
                        </div>
                        <button
                              type="button"
                              onClick={() =>
                                    props.onNavigate?.(
                                          destinationDesktop("settings"),
                                    )
                              }
                        >
                              Settings
                        </button>
                        <button
                              type="button"
                              aria-expanded={workshopExpanded()}
                              onClick={() => {
                                    props.onNavigate?.(
                                          destinationDesktop("agents"),
                                    );
                                    const next = !workshopExpanded();
                                    setWorkshopExpanded(next);
                                    persist("workshop", next);
                              }}
                        >
                              Workshop
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
