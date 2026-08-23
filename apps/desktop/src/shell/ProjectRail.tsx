import { createMemo, createSignal, For } from "solid-js";
import { destinationDesktop } from "../nav/desktop-navigation";
import { Desktop_GLOBAL_ROUTE_KINDS } from "../nav/desktop-route-kinds";

export const RAIL_EXPANSION_STORAGE_KEY = "locus.rail-expansion";

const PROJECT_RAIL_LINKS = {
  Plan: "plan-conversation",
  Develop: "develop",
  Automate: "automate-kanban",
  Review: "review-telemetry",
} as const;
const MEMORY_ROUTES = Desktop_GLOBAL_ROUTE_KINDS.filter((route) =>
  route.id.startsWith("memory-"),
);
const WORKSHOP_ROUTES = Desktop_GLOBAL_ROUTE_KINDS.filter((route) =>
  route.id.startsWith("workshop-"),
);

export interface ProjectRailProps {
  selectedProject: string;
  inboxCount?: number;
  dispatchState?: "ready" | "working" | "blocked";
  projects?: readonly string[];
  onNavigate?: (locator: string) => void;
}

export function ProjectRail(props: ProjectRailProps) {
  const savedExpansion = () =>
    JSON.parse(
      localStorage.getItem(RAIL_EXPANSION_STORAGE_KEY) ?? "{}",
    ) as Record<string, boolean>;
  const persistExpansion = (name: "memory" | "workshop", value: boolean) => {
    localStorage.setItem(
      RAIL_EXPANSION_STORAGE_KEY,
      JSON.stringify({ ...savedExpansion(), [name]: value }),
    );
  };
  const [filter, setFilter] = createSignal("");
  const [activeProject, setActiveProject] = createSignal(0);
  const [activeGlobalRoute, setActiveGlobalRoute] = createSignal(0);
  const [memoryExpanded, setMemoryExpanded] = createSignal(
    savedExpansion().memory ?? false,
  );
  const [workshopExpanded, setWorkshopExpanded] = createSignal(
    savedExpansion().workshop ?? false,
  );
  const projects = createMemo(() => {
    const needle = filter().trim().toLowerCase();
    return (props.projects ?? [props.selectedProject]).filter((project) =>
      project.toLowerCase().includes(needle),
    );
  });

  const moveActiveProject = (direction: 1 | -1) => {
    const count = projects().length;
    if (!count) return;
    setActiveProject((current) => (current + direction + count) % count);
  };

  return (
    <nav
      aria-label="Application navigation"
      class="project-rail"
      data-testid="project-rail"
    >
      <div
        data-testid="global-rail-routes"
        onKeyDown={(event) => {
          if (event.key === "ArrowDown") {
            event.preventDefault();
            setActiveGlobalRoute(
              (index) => (index + 1) % Desktop_GLOBAL_ROUTE_KINDS.length,
            );
          }
        }}
      >
        <span
          data-testid="dispatch-dot"
          data-state={props.dispatchState ?? "ready"}
          aria-label="Dispatch status"
        />
        <For each={Desktop_GLOBAL_ROUTE_KINDS}>
          {(route, index) => (
            <button
              type="button"
              tabIndex={activeGlobalRoute() === index() ? 0 : -1}
              onClick={() => props.onNavigate?.(destinationDesktop(route.id))}
            >
              {route.label}
              {route.id === "inbox" && props.inboxCount ? (
                <span data-testid="global-rail-inbox-badge">
                  {props.inboxCount}
                </span>
              ) : null}
            </button>
          )}
        </For>
      </div>
      <section>
        <button
          type="button"
          aria-expanded={memoryExpanded()}
          onClick={() => {
            props.onNavigate?.(destinationDesktop("memory-short-term"));
            setMemoryExpanded((open) => {
              const next = !open;
              persistExpansion("memory", next);
              return next;
            });
          }}
        >
          Memory
        </button>
        <div data-testid="memory-rail-links" hidden={!memoryExpanded()}>
          <For each={MEMORY_ROUTES}>
            {(route) => (
              <button type="button" onClick={() => props.onNavigate?.(destinationDesktop(route.id))}>
                {route.label.replace("Memory ", "")}
              </button>
            )}
          </For>
        </div>
      </section>
      <section>
        <button
          type="button"
          aria-expanded={workshopExpanded()}
          onClick={() => {
            props.onNavigate?.(destinationDesktop("workshop-agents"));
            setWorkshopExpanded((open) => {
              const next = !open;
              persistExpansion("workshop", next);
              return next;
            });
          }}
        >
          Workshop
        </button>
        <div data-testid="workshop-rail-links" hidden={!workshopExpanded()}>
          <For each={WORKSHOP_ROUTES}>
            {(route) => (
              <button type="button" onClick={() => props.onNavigate?.(destinationDesktop(route.id))}>
                {route.label.replace("Workshop ", "")}
              </button>
            )}
          </For>
        </div>
      </section>
      <section data-testid="selected-project-card">
        <span>{props.selectedProject}</span>
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
              moveActiveProject(1);
            }
            if (event.key === "ArrowUp") {
              event.preventDefault();
              moveActiveProject(-1);
            }
          }}
        />
        <div data-testid="project-switcher-results">
          <For each={projects()}>
            {(project, index) => (
              <button
                type="button"
                data-testid={`project-switcher-option-${project}`}
                aria-selected={activeProject() === index()}
                onClick={() =>
                  props.onNavigate?.(destinationDesktop("plan-conversation", project))
                }
              >
                {project}
              </button>
            )}
          </For>
        </div>
        <div data-testid="project-rail-routes">
          <For each={Object.entries(PROJECT_RAIL_LINKS)}>
            {([label, route]) => (
              <button
                type="button"
                onClick={() =>
                  props.onNavigate?.(destinationDesktop(route, props.selectedProject))
                }
              >
                {label}
              </button>
            )}
          </For>
        </div>
      </section>
    </nav>
  );
}
