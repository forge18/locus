import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { InboxCard } from "./InboxCard";
import { InboxDetail } from "./InboxDetail";
import { EmptyPane } from "../../ui/EmptyPane";
import { ProjectFilter } from "../../shell/ProjectFilter";
import {
  fetchInboxList,
  fetchInboxThroughput,
  fetchResolvedToday,
  resolveInboxDelivery,
  type InboxDelivery,
  type InboxThroughput,
  type ResolvedDelivery,
} from "../../data/inbox";
import { fetchProjects } from "../../data/core";
import type { Envelope } from "../../data/envelope";
import { notify } from "../../ui/Toast";
import type { NavStore } from "../../nav";

export interface InboxViewProps {
  nav: NavStore;
}

type InboxTab = "todo" | "completed";

const resolutionTime = (minutes: number) => {
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remainder = minutes % 60;
  return remainder === 0 ? `${hours}h` : `${hours}h ${remainder}m`;
};
const minutesSince = (at: string | null) =>
  at == null
    ? 0
    : Math.max(0, Math.floor((Date.now() - Date.parse(at)) / 60000));

const dayId = (day: string) => day.toLowerCase().replace(/[^a-z0-9]+/g, "-");

function groupByDay(items: ResolvedDelivery[]): { day: string; items: ResolvedDelivery[] }[] {
  const groups = new Map<string, ResolvedDelivery[]>();
  for (const item of items) {
    const day = item.resolvedAt
      ? new Date(item.resolvedAt).toLocaleDateString()
      : "Today";
    const group = groups.get(day);
    if (group) group.push(item);
    else groups.set(day, [item]);
  }
  return [...groups.entries()].map(([day, groupedItems]) => ({
    day,
    items: groupedItems,
  }));
}
/** What the person decided when they resolved an item — kept so it is auditable. */
interface InboxDecision {
  action: "approved" | "sent-back";
  comment: string;
}

/**
 * The only interruption surface. A decision resolves here; the work it is about
 * opens where that work lives, by locator — this screen never grows a second copy
 * of Plan, Interact or Review.
 */
export function InboxView(_props: InboxViewProps) {
  const [tab, setTab] = createSignal<InboxTab>("todo");
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [selectedProjects, setSelectedProjects] = createSignal<string[]>([]);
  const [decisions, setDecisions] = createSignal<Record<string, InboxDecision>>(
    {},
  );
  const [resolvedIds, setResolvedIds] = createSignal<Set<string>>(new Set());

  // The live inbox (slice 7): pending human deliveries, today's resolved list,
  // and the real counts. Every read is an envelope; a failure is visible.
  const [itemsEnvelope, setItemsEnvelope] = createSignal<Envelope<InboxDelivery[]>>({
    status: "loading",
  });
  const [resolvedEnvelope, setResolvedEnvelope] = createSignal<
    Envelope<ResolvedDelivery[]>
  >({ status: "loading" });
  const [throughputEnvelope, setThroughputEnvelope] = createSignal<
    Envelope<InboxThroughput>
  >({ status: "loading" });
  const [projects, setProjects] = createSignal<{ id: string; name: string }[]>(
    [],
  );

  async function refreshInbox() {
    const [list, today, counts] = await Promise.all([
      fetchInboxList(),
      fetchResolvedToday(),
      fetchInboxThroughput(),
    ]);
    setItemsEnvelope(list);
    setResolvedEnvelope(today);
    setThroughputEnvelope(counts);
    for (const failed of [list, today, counts]) {
      if (failed.status === "failed") {
        notify({
          title: "Inbox unavailable",
          description: failed.error.message,
          type: "error",
        });
      }
    }
  }

  onMount(() => {
    void refreshInbox();
    void fetchProjects().then((envelope) => {
      if (envelope.status === "ready") setProjects(envelope.data);
    });
  });

  const [recentlyResolved, setRecentlyResolved] = createSignal<
    { delivery: InboxDelivery; comment: string; resolvedAt: string }[]
  >([]);
  const throughput = createMemo<InboxThroughput>(() => {
    const envelope = throughputEnvelope();
    return envelope.status === "ready"
      ? envelope.data
      : { pending: 0, resolvedToday: 0 };
  });
  const projectNames = createMemo(() => {
    const selected = selectedProjects();
    if (selected.length === 0) return null;
    return new Set(
      projects()
        .filter((project) => selected.includes(project.id))
        .map((project) => project.name),
    );
  });
  const matchesProject = (project: string) => {
    const names = projectNames();
    return names === null || names.has(project);
  };
  const items = createMemo<InboxDelivery[]>(() => {
    const envelope = itemsEnvelope();
    if (envelope.status !== "ready") return [];
    return envelope.data.filter(
      (delivery) =>
        !resolvedIds().has(delivery.id) && matchesProject(delivery.project),
    );
  });
  const completedItems = createMemo<ResolvedDelivery[]>(() => {
    const envelope = resolvedEnvelope();
    const rows = envelope.status === "ready" ? envelope.data : [];
    // Locally resolved rows arrive from the pending list (InboxDelivery), so
    // normalize them into the resolved shape the completed list renders.
    const local = recentlyResolved().map(({ delivery, resolvedAt }) => ({
      id: delivery.id,
      subject: delivery.subject,
      body: delivery.body,
      project: delivery.project,
      resolvedAt,
    }));
    return [...local, ...rows].filter((row) => matchesProject(row.project));
  });
  const completedGroups = createMemo(() => groupByDay(completedItems()));
  const selected = createMemo(() => {
    const list = items();
    return list.find((item) => item.id === selectedId()) ?? list[0] ?? null;
  });

  const approveItem = (id: string, comment: string) => {
    const item = items().find((candidate) => candidate.id === id);
    if (!item) return;
    void resolveInboxDelivery(id, comment.trim()).then((envelope) => {
      if (envelope.status === "failed") {
        notify({
          title: "Resolve failed",
          description: envelope.error.message,
          type: "error",
        });
        return;
      }
      setDecisions((current) => ({
        ...current,
        [id]: { action: "approved", comment: comment.trim() },
      }));
      setRecentlyResolved((current) => [
        ...current,
        { delivery: item, comment: comment.trim(), resolvedAt: new Date().toISOString() },
      ]);
      setResolvedIds((current) => new Set(current).add(id));
      void refreshInbox();
    });
  };

  /**
   * Send back returns the work to the agent that made it — the comment is the
   * response, so an empty one is blocked at the detail pane and never resolves.
   */
  const sendBackItem = (id: string, comment: string) => {
    const reason = comment.trim();
    if (!reason) return;
    const item = items().find((candidate) => candidate.id === id);
    setDecisions((current) => ({
      ...current,
      [id]: { action: "sent-back", comment: reason },
    }));
    void resolveInboxDelivery(id, reason).then((envelope) => {
      if (envelope.status === "failed") {
        notify({
          title: "Resolve failed",
          description: envelope.error.message,
          type: "error",
        });
        return;
      }
      setDecisions((current) => ({
        ...current,
        [id]: { action: "sent-back", comment: reason },
      }));
      setRecentlyResolved((current) => [
        ...current,
        { delivery: item!, comment: reason, resolvedAt: new Date().toISOString() },
      ]);
      setResolvedIds((current) => new Set(current).add(id));
      void refreshInbox();
    });
  };

  /** Locally resolved rows carry the comment recorded at decision time; a
   * decision recorded this session survives the refresh. */
  const decisionFor = (row: ResolvedDelivery): string | undefined =>
    decisions()[row.id]?.comment ??
    recentlyResolved().find((entry) => entry.delivery.id === row.id)?.comment;

  return (
    <div class="inbox" data-testid="inbox" data-desktop-route="inbox">
      <div
        class="inbox-list"
        data-testid="inbox-list"
        aria-live="polite"
        aria-atomic="false"
      >
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
          <span data-testid="inbox-throughput-value">
            {throughput().pending} pending · {throughput().resolvedToday}{" "}
            resolved today
          </span>
          <span
            class="inbox-throughput-status"
            data-testid="inbox-throughput-status"
          >
            {throughput().pending === 0 ? "clear" : "waiting on you"}
          </span>
        </div>

        <div class="inbox-filter" data-testid="inbox-project-filter">
          <ProjectFilter
            projects={projects()}
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
                              <div style={{ flex: 1, "min-width": 0 }}>
                                <span class="inbox-completed-title">
                                  {row.subject}
                                </span>
                                <Show when={decisionFor(row)}>
                                  {(comment) => (
                                    <span
                                      class="inbox-completed-decision"
                                      data-testid={`resolved-decision-${row.id}`}
                                      style={{
                                        color: "var(--text-muted)",
                                        "font-size": "var(--t-meta)",
                                      }}
                                    >
                                      {decisions()[row.id]?.action ===
                                      "sent-back"
                                        ? `Sent back: ${comment()}`
                                        : comment()}
                                    </span>
                                  )}
                                </Show>
                              </div>
                              <span
                                class="inbox-resolution inbox-resolution-time"
                                data-testid={`resolved-time-${row.id}`}
                                data-resolution-minutes={minutesSince(
                                  row.resolvedAt,
                                )}
                                data-resolution-time={resolutionTime(
                                  minutesSince(row.resolvedAt),
                                )}
                              >
                                Resolved in{" "}
                                {resolutionTime(minutesSince(row.resolvedAt))}
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
                    <span>{row.subject}</span>
                    <span
                      style={{
                        "margin-left": "auto",
                        color: "var(--text-muted)",
                      }}
                    >
                      {resolutionTime(minutesSince(row.resolvedAt))} ago
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
          />
        )}
      </Show>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default InboxView;
