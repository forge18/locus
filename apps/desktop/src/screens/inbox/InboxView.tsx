import { For, Show, createMemo, createSignal } from "solid-js";
import { InboxCard } from "./InboxCard";
import { InboxDetail } from "./InboxDetail";
import { EmptyPane } from "../../ui/EmptyPane";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Icon } from "../../ui/Icon";
import { ProjectFilter } from "../../shell/ProjectFilter";
import {
  useInboxItems,
  useInboxThroughput,
  useResolvedToday,
} from "../../data/inbox";
import { useProjects } from "../../data/core";
import type { ResolvedItem } from "../../data/inbox";
import type { NavStore } from "../../nav";

export interface InboxViewProps {
  nav: NavStore;
}

type InboxTab = "todo" | "completed";

interface ResolvedDay {
  day: string;
  items: ResolvedItem[];
}

/** What the person decided when they resolved an item — kept so it is auditable. */
interface InboxDecision {
  action: "approved" | "sent-back";
  comment: string;
}

const RESOLVED_ICON = {
  gate: "seal-check",
  ask: "question",
  guardrail: "warning-octagon",
  reflection: "sparkle",
} as const;
const age = (minutes: number) =>
  minutes < 60 ? `${minutes}m` : `${Math.floor(minutes / 60)}h`;
const resolutionTime = (minutes: number) => {
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remainder = minutes % 60;
  return remainder === 0 ? `${hours}h` : `${hours}h ${remainder}m`;
};
const dayId = (day: string) => day.toLowerCase().replace(/[^a-z0-9]+/g, "-");

function groupByDay(items: ResolvedItem[]): ResolvedDay[] {
  const groups = new Map<string, ResolvedItem[]>();
  for (const item of items) {
    const group = groups.get(item.resolvedDay);
    if (group) group.push(item);
    else groups.set(item.resolvedDay, [item]);
  }
  return [...groups.entries()].map(([day, groupedItems]) => ({
    day,
    items: groupedItems,
  }));
}

/**
 * The only interruption surface. A decision resolves here; the work it is about
 * opens where that work lives, by locator — this screen never grows a second copy
 * of Plan, Interact or Review.
 */
export function InboxView(props: InboxViewProps) {
  const [tab, setTab] = createSignal<InboxTab>("todo");
  const [resolved, setResolved] = createSignal<string[]>([]);
  const [recentlyResolved, setRecentlyResolved] = createSignal<ResolvedItem[]>(
    [],
  );
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [selectedProjects, setSelectedProjects] = createSignal<string[]>([]);
  const [decisions, setDecisions] = createSignal<Record<string, InboxDecision>>(
    {},
  );

  const projects = useProjects().map((project) => ({
    id: project.id,
    name: project.name,
  }));
  const throughput = useInboxThroughput();
  const projectNames = createMemo(() => {
    const selected = selectedProjects();
    if (selected.length === 0) return null;
    return new Set(
      projects
        .filter((project) => selected.includes(project.id))
        .map((project) => project.name),
    );
  });
  const matchesProject = (project: string) => {
    const names = projectNames();
    return names === null || names.has(project);
  };
  const items = createMemo(() =>
    useInboxItems().filter(
      (item) => !resolved().includes(item.id) && matchesProject(item.project),
    ),
  );
  const completedItems = createMemo(() =>
    [...recentlyResolved(), ...useResolvedToday()].filter((item) =>
      matchesProject(item.project),
    ),
  );
  const completedGroups = createMemo(() => groupByDay(completedItems()));
  const selected = createMemo(
    () =>
      items().find((item) => item.id === selectedId()) ?? items()[0] ?? null,
  );

  /**
   * Resolving is in place: nothing about where you are changes. Named
   * `resolveItem` rather than `resolve` because `resolve` is the navigation
   * resolver, and one word meaning two things here would be a trap.
   */
  const resolveItem = (id: string) => {
    const item = useInboxItems().find((candidate) => candidate.id === id);
    if (!item || resolved().includes(id)) return;
    setResolved((current) => [...current, id]);
    setRecentlyResolved((current) => [
      ...current,
      {
        id: `resolved-${item.id}`,
        kind: item.kind,
        title: item.title,
        project: item.project,
        resolutionMinutes: item.ageMinutes,
        resolvedDay: "Today",
        ageMinutes: 0,
      },
    ]);
  };

  const recordDecision = (id: string, decision: InboxDecision) => {
    setDecisions((current) => ({ ...current, [id]: decision }));
  };

  /** Approve releases the loop as-is; the comment is optional steering. */
  const approveItem = (id: string, comment: string) => {
    recordDecision(id, { action: "approved", comment: comment.trim() });
    resolveItem(id);
  };

  /**
   * Send back returns the work to the agent that made it — the comment is the
   * response, so an empty one is blocked at the detail pane and never resolves.
   */
  const sendBackItem = (id: string, comment: string) => {
    const reason = comment.trim();
    if (!reason) return;
    recordDecision(id, { action: "sent-back", comment: reason });
    resolveItem(id);
  };

  /** Locally resolved rows carry a `resolved-` prefix; decisions key on the item. */
  const decisionFor = (row: ResolvedItem): InboxDecision | undefined =>
    decisions()[row.id.replace(/^resolved-/, "")];

  return (
    <div class="inbox" data-testid="inbox" data-desktop-route="inbox">
      <div
        class="inbox-list"
        data-testid="inbox-list"
        aria-live="polite"
        aria-atomic="false"
      >
        <FixtureNotice surface="Inbox" command='invoke("inbox_list")' />
        <div class="inbox-tabs" data-testid="inbox-tabs" role="tablist">
          <button
            type="button"
            role="tab"
            data-testid="inbox-tab-todo"
            data-inbox-tab="todo"
            data-inbox-group="action-required"
            aria-selected={tab() === "todo" ? "true" : "false"}
            aria-current={tab() === "todo" ? "page" : undefined}
            aria-controls="inbox-todo-panel"
            onClick={() => setTab("todo")}
          >
            To do <span data-testid="inbox-todo-count">{items().length}</span>
          </button>
          <button
            type="button"
            role="tab"
            data-testid="inbox-tab-completed"
            data-inbox-tab="completed"
            data-inbox-group="completed"
            aria-selected={tab() === "completed" ? "true" : "false"}
            aria-current={tab() === "completed" ? "page" : undefined}
            aria-controls="inbox-completed-panel"
            onClick={() => setTab("completed")}
          >
            Completed{" "}
            <span data-testid="inbox-completed-count">
              {completedItems().length}
            </span>
          </button>
        </div>

        <div
          class="inbox-throughput"
          data-testid="inbox-throughput"
          data-inbox-budget="true"
        >
          <span
            class="inbox-throughput-meter"
            aria-hidden="true"
            style={{
              "--inbox-throughput-width": `${Math.min(
                (throughput.resolvedThisHour / throughput.hourlyBudget) * 100,
                100,
              )}%`,
            }}
          >
            <i />
          </span>
          <span data-testid="inbox-throughput-value">
            {throughput.resolvedThisHour} / {throughput.hourlyBudget}{" "}
            {throughput.periodLabel}
          </span>
          <span
            class="inbox-throughput-status"
            data-testid="inbox-throughput-status"
          >
            {throughput.resolvedThisHour < throughput.hourlyBudget
              ? "under budget"
              : "at budget"}
          </span>
        </div>

        <div class="inbox-filter" data-testid="inbox-project-filter">
          <ProjectFilter
            projects={projects}
            selected={selectedProjects()}
            onChange={setSelectedProjects}
          />
          <span data-testid="inbox-project-filter-note">
            Filters this list only. Every other screen keeps its own.
          </span>
        </div>

        <Show
          when={tab() === "todo"}
          fallback={
            <section
              id="inbox-completed-panel"
              role="tabpanel"
              aria-labelledby="inbox-tab-completed"
              data-testid="inbox-completed-panel"
              data-inbox-completed="true"
            >
              <div class="inbox-section">
                <span class="inbox-section-title" data-testid="completed-title">
                  Completed
                </span>
                <span class="inbox-section-note">
                  Kept so the resolution is auditable — what you decided, and
                  how long a loop waited on you for it.
                </span>
              </div>
              <Show
                when={completedGroups().length > 0}
                fallback={
                  <EmptyPane icon="check" reason="Nothing resolved yet" />
                }
              >
                <div
                  class="inbox-completed-items"
                  data-testid="inbox-completed-items"
                  role="log"
                  aria-live="polite"
                  aria-atomic="false"
                >
                  <For each={completedGroups()}>
                    {(group) => (
                      <section
                        class="inbox-completed-day inbox-resolved-day"
                        data-testid={`inbox-completed-day-${dayId(group.day)}`}
                        data-day={group.day}
                        data-resolved-day={group.day}
                      >
                        <h2 class="inbox-completed-day-title">{group.day}</h2>
                        <For each={group.items}>
                          {(row) => (
                            <div
                              class="inbox-completed-row inbox-resolved-row"
                              data-testid={`inbox-completed-row-${row.id}`}
                            >
                              <Icon name={RESOLVED_ICON[row.kind]} size={11} />
                              <div style={{ flex: 1, "min-width": 0 }}>
                                <span class="inbox-completed-title">
                                  {row.title}
                                </span>
                                <Show when={decisionFor(row)?.comment}>
                                  {(comment) => (
                                    <span
                                      class="inbox-completed-decision"
                                      data-testid={`resolved-decision-${row.id}`}
                                      style={{
                                        color: "var(--text-muted)",
                                        "font-size": "var(--t-meta)",
                                      }}
                                    >
                                      {comment()}
                                    </span>
                                  )}
                                </Show>
                              </div>
                              <span
                                class="inbox-resolution inbox-resolution-time"
                                data-testid={`resolved-time-${row.id}`}
                                data-resolution-minutes={row.resolutionMinutes}
                                data-resolution-time={resolutionTime(
                                  row.resolutionMinutes,
                                )}
                              >
                                Resolved in{" "}
                                {resolutionTime(row.resolutionMinutes)}
                              </span>
                            </div>
                          )}
                        </For>
                      </section>
                    )}
                  </For>
                </div>
              </Show>
            </section>
          }
        >
          <section
            id="inbox-todo-panel"
            role="tabpanel"
            aria-labelledby="inbox-tab-todo"
            data-testid="inbox-todo-panel"
          >
            <div class="inbox-section">
              <span class="inbox-section-title" data-testid="needs-you-title">
                Needs you
              </span>
              <span class="inbox-section-note" data-testid="needs-you-note">
                {items().length} {items().length === 1 ? "item" : "items"} ·
                silence is the default
              </span>
            </div>

            <div
              class="inbox-items"
              data-testid="inbox-items"
              role="log"
              aria-live="polite"
              aria-atomic="false"
            >
              <Show
                when={items().length > 0}
                fallback={<EmptyPane icon="tray" reason="Nothing needs you" />}
              >
                <For each={items()}>
                  {(item) => (
                    <InboxCard
                      item={item}
                      selected={selected()?.id === item.id}
                      onSelect={() => setSelectedId(item.id)}
                    />
                  )}
                </For>
              </Show>
            </div>

            <p class="inbox-note" data-testid="inbox-note">
              Every item type documents the response it wants. Items without a
              response belong in Activity.
            </p>

            <div class="inbox-section inbox-resolved-section">
              <span
                class="inbox-section-title"
                style={{ color: "var(--text-muted)" }}
                data-testid="resolved-title"
              >
                Resolved today
              </span>
            </div>
            <div class="inbox-resolved" data-testid="inbox-resolved">
              <For each={completedItems()}>
                {(row) => (
                  <div
                    class="inbox-resolved-row"
                    data-testid={`resolved-${row.id}`}
                  >
                    <Icon name={RESOLVED_ICON[row.kind]} size={11} />
                    <span>{row.title}</span>
                    <span
                      style={{
                        "margin-left": "auto",
                        color: "var(--text-muted)",
                      }}
                    >
                      {age(row.ageMinutes)}
                    </span>
                  </div>
                )}
              </For>
            </div>
          </section>
        </Show>
      </div>

      <Show
        when={selected()}
        keyed
        fallback={
          <EmptyPane reason="Nothing needs you — approve something and it resolves right here." />
        }
      >
        {(item) => (
          <InboxDetail
            item={item}
            onApprove={(comment) => approveItem(item.id, comment)}
            onSendBack={(comment) => sendBackItem(item.id, comment)}
            onOpenWork={(locator) => props.nav.open(locator)}
          />
        )}
      </Show>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default InboxView;
