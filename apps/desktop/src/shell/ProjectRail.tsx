import { createMemo, createSignal, For } from "solid-js";
import { V2_GLOBAL_ROUTE_KINDS } from "../nav/v2-route-kinds";

export const RAIL_EXPANSION_STORAGE_KEY = "locus.rail-expansion";

const PROJECT_RAIL_LINKS = ["Plan", "Develop", "Automate", "Review"] as const;
const MEMORY_ROUTES = V2_GLOBAL_ROUTE_KINDS.filter((route) =>
  route.id.startsWith("memory-"),
);
const WORKSHOP_ROUTES = V2_GLOBAL_ROUTE_KINDS.filter((route) =>
  route.id.startsWith("workshop-"),
);

export interface ProjectRailProps {
  selectedProject: string;
  inboxCount?: number;
  projects?: readonly string[];
}

export function ProjectRail(props: ProjectRailProps) {
  const savedExpansion = () => JSON.parse(localStorage.getItem(RAIL_EXPANSION_STORAGE_KEY) ?? "{}") as Record<string, boolean>;
  const persistExpansion = (name: "memory" | "workshop", value: boolean) => {
    localStorage.setItem(RAIL_EXPANSION_STORAGE_KEY, JSON.stringify({ ...savedExpansion(), [name]: value }));
  };
  const [filter, setFilter] = createSignal("");
  const [activeProject, setActiveProject] = createSignal(0);
  const [memoryExpanded, setMemoryExpanded] = createSignal(savedExpansion().memory ?? false);
  const [workshopExpanded, setWorkshopExpanded] = createSignal(savedExpansion().workshop ?? false);
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
      <div data-testid="global-rail-routes">
        <For each={V2_GLOBAL_ROUTE_KINDS}>
          {(route) => (
            <button type="button">
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
          onClick={() => setMemoryExpanded((open) => {
            const next = !open;
            persistExpansion("memory", next);
            return next;
          })}
        >
          Memory
        </button>
        <div data-testid="memory-rail-links" hidden={!memoryExpanded()}>
          <For each={MEMORY_ROUTES}>
            {(route) => (
              <button type="button">
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
          onClick={() => setWorkshopExpanded((open) => {
            const next = !open;
            persistExpansion("workshop", next);
            return next;
          })}
        >
          Workshop
        </button>
        <div data-testid="workshop-rail-links" hidden={!workshopExpanded()}>
          <For each={WORKSHOP_ROUTES}>
            {(route) => (
              <button type="button">
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
              >
                {project}
              </button>
            )}
          </For>
        </div>
        <div data-testid="project-rail-routes">
          <For each={PROJECT_RAIL_LINKS}>
            {(label) => <button type="button">{label}</button>}
          </For>
        </div>
      </section>
    </nav>
  );
}
