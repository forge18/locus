import { For, Show, createMemo, createSignal } from "solid-js";
import type { AgentEvent } from "../../types/event";
import { Avatar } from "../../avatars/Avatar";
import { AgentPane, type AgentPaneSession } from "../../panes/AgentPane";
import { destinationDesktop } from "../../nav/desktop-navigation";
import { createBot, type Bot } from "../../data/bots";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Input, Textarea } from "../../ui/Input";
import { Sheet } from "../../ui/Sheet";
import "./bots.css";

export type BotState = "working" | "idle";

export interface BotViewModel {
  id: string;
  name: string;
  description: string;
  harness: string;
  lastActivity: string;
  state: BotState;
  runId: string;
}

export interface RoutineViewModel {
  id: string;
  prompt: string;
  cron: string;
  enabled: boolean;
  skipped: number;
}

const BOTS: readonly BotViewModel[] = [
  {
    id: "keeper",
    name: "Keeper",
    description: "Curates durable project memory.",
    harness: "pi",
    lastActivity: "now",
    state: "working",
    runId: "bot-keeper-run",
  },
  {
    id: "night-watch",
    name: "Night Watch",
    description: "Checks the repository while the window is closed.",
    harness: "pi",
    lastActivity: "18m ago",
    state: "idle",
    runId: "bot-night-watch-run",
  },
];

const INITIAL_ROUTINES: readonly RoutineViewModel[] = [
  {
    id: "routine-health",
    prompt: "Check the repository health and report only actionable drift.",
    cron: "0 9 * * 1-5",
    enabled: true,
    skipped: 1,
  },
];

export interface BotsViewProps {
  projectId?: string;
  botId?: string;
  bots?: readonly BotViewModel[];
  initialRoutines?: readonly RoutineViewModel[];
}

function toBotViewModel(bot: Bot): BotViewModel {
  return {
    id: bot.id,
    name: bot.name,
    description: "Created from the Bots workspace.",
    harness: "pi",
    lastActivity: "now",
    state: bot.containerState === "running" ? "working" : "idle",
    runId: bot.homeSessionId,
  };
}

function botSession(bot: BotViewModel, project: string): AgentPaneSession {
  return {
    project,
    agent: bot.name,
    model: "claude-sonnet-4",
    harness: bot.harness,
    effort: "high",
    name: `${bot.name} home session`,
    context: { used: 12_400, total: 200_000 },
    cost: "$0.42",
    permissionPosture: "bypass",
    status: bot.state,
  };
}

function BotViewHeader(props: { bot: BotViewModel }) {
  return (
    <header class="bot-view-header" data-testid="bot-view-header">
      <Avatar
        seed={props.bot.id}
        alt={`${props.bot.name} avatar`}
        class="bot-avatar bot-avatar-header"
        testId="bot-header-avatar"
      />
      <div class="bot-view-header-copy">
        <h1>{props.bot.name}</h1>
        <span>{props.bot.harness}</span>
      </div>
    </header>
  );
}

function botEvents(bot: BotViewModel): AgentEvent[] {
  return [
    {
      id: `${bot.runId}-user`,
      runId: bot.runId,
      seq: 0,
      ts: "now",
      verb: "user",
      text: "Keep an eye on this project and tell me what matters.",
      raw: { source: "bot-home" },
    },
    {
      id: `${bot.runId}-assistant`,
      runId: bot.runId,
      seq: 1,
      ts: "now",
      verb: "assistant",
      text: `${bot.name} is ready. This is the durable home conversation.`,
      raw: { source: "bot-home" },
    },
  ];
}

function RoutinesSheet(props: {
  routines: readonly RoutineViewModel[];
  onChange: (routines: RoutineViewModel[]) => void;
  onClose: () => void;
}) {
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [draftPrompt, setDraftPrompt] = createSignal("");
  const [testRun, setTestRun] = createSignal<string | null>(null);

  const beginEdit = (routine: RoutineViewModel) => {
    setEditingId(routine.id);
    setDraftPrompt(routine.prompt);
  };
  const saveEdit = (routine: RoutineViewModel) => {
    props.onChange(
      props.routines.map((candidate) =>
        candidate.id === routine.id
          ? { ...candidate, prompt: draftPrompt() || candidate.prompt }
          : candidate,
      ),
    );
    setEditingId(null);
  };
  const toggle = (routine: RoutineViewModel) => {
    props.onChange(
      props.routines.map((candidate) =>
        candidate.id === routine.id
          ? { ...candidate, enabled: !candidate.enabled }
          : candidate,
      ),
    );
  };
  const remove = (routine: RoutineViewModel) => {
    props.onChange(
      props.routines.filter((candidate) => candidate.id !== routine.id),
    );
  };

  return (
    <div class="bots-routines-sheet" data-testid="bots-routines-sheet">
      <header class="bots-routines-head">
        <div>
          <span class="bots-eyebrow">ongoing</span>
          <h2>Routines</h2>
        </div>
        <Button variant="secondary" onClick={props.onClose}>
          Done
        </Button>
      </header>
      <p class="bots-routines-note">
        Cron prompts post into this bot&apos;s home conversation. Overlap is
        skipped, never queued.
      </p>
      <section class="bots-routine-list" aria-label="Bot routines">
        <Show
          when={props.routines.length > 0}
          fallback={
            <p class="bots-empty-routines" data-testid="bots-empty-routines">
              No routines yet.
            </p>
          }
        >
          <For each={props.routines}>
            {(routine) => (
              <article
                class="bots-routine-card"
                data-testid={`bot-routine-${routine.id}`}
                data-enabled={routine.enabled ? "true" : "false"}
              >
                <div class="bots-routine-card-head">
                  <span class="bots-routine-status">
                    {routine.enabled ? "enabled" : "paused"}
                  </span>
                  <code>{routine.cron}</code>
                  <span class="bots-routine-skips">
                    skipped {routine.skipped}
                  </span>
                </div>
                <Show
                  when={editingId() === routine.id}
                  fallback={<p>{routine.prompt}</p>}
                >
                  <Input
                    aria-label="Routine prompt"
                    value={draftPrompt()}
                    onInput={(event) =>
                      setDraftPrompt(event.currentTarget.value)
                    }
                  />
                </Show>
                <div class="bots-routine-actions">
                  <Button variant="ghost" onClick={() => toggle(routine)}>
                    {routine.enabled ? "Pause" : "Enable"}
                  </Button>
                  <Show
                    when={editingId() === routine.id}
                    fallback={
                      <Button
                        variant="ghost"
                        onClick={() => beginEdit(routine)}
                      >
                        Edit
                      </Button>
                    }
                  >
                    <Button variant="ghost" onClick={() => saveEdit(routine)}>
                      Save
                    </Button>
                  </Show>
                  <Button variant="ghost" onClick={() => remove(routine)}>
                    Delete
                  </Button>
                  <Button
                    variant="secondary"
                    data-testid={`bot-routine-test-${routine.id}`}
                    onClick={() => setTestRun(routine.id)}
                  >
                    Test run
                  </Button>
                </div>
                <Show when={testRun() === routine.id}>
                  <output
                    class="bots-test-run"
                    data-testid="bot-routine-test-result"
                    data-test-run="true"
                  >
                    Test run sent · attributed as test-run
                  </output>
                </Show>
              </article>
            )}
          </For>
        </Show>
      </section>
    </div>
  );
}

export default function BotsView(props: BotsViewProps) {
  const project = () => props.projectId ?? "tapestry";
  const [createdBots, setCreatedBots] = createSignal<BotViewModel[]>([]);
  const bots = createMemo(() => [...(props.bots ?? BOTS), ...createdBots()]);
  const [selectedId, setSelectedId] = createSignal(
    props.botId && bots().some((bot) => bot.id === props.botId)
      ? props.botId
      : bots()[0]?.id,
  );
  const [collapsed, setCollapsed] = createSignal(false);
  const [routinesOpen, setRoutinesOpen] = createSignal(false);
  const [newBotOpen, setNewBotOpen] = createSignal(false);
  const [newBotMarkdown, setNewBotMarkdown] = createSignal("");
  const [newBotError, setNewBotError] = createSignal<string | null>(null);
  const [creatingBot, setCreatingBot] = createSignal(false);
  const [routines, setRoutines] = createSignal(
    props.initialRoutines ?? INITIAL_ROUTINES,
  );
  const selected = createMemo(
    () => bots().find((bot) => bot.id === selectedId()) ?? bots()[0],
  );

  const openNewBot = () => {
    setNewBotError(null);
    setNewBotMarkdown("");
    setNewBotOpen(true);
  };

  const submitNewBot = async (event: SubmitEvent) => {
    event.preventDefault();
    const markdown = newBotMarkdown().trim();
    if (!markdown) {
      setNewBotError("Enter a bot definition before creating the bot.");
      return;
    }

    setCreatingBot(true);
    setNewBotError(null);
    try {
      const bot = await createBot(project(), markdown);
      const viewModel = toBotViewModel(bot);
      setCreatedBots((current) => [...current, viewModel]);
      setSelectedId(viewModel.id);
      setNewBotOpen(false);
    } catch (error) {
      setNewBotError(
        error instanceof Error
          ? error.message
          : "The bot could not be created.",
      );
    } finally {
      setCreatingBot(false);
    }
  };

  return (
    <div
      class="bots-view"
      data-testid="bots-view"
      data-project={project()}
      data-collapsed={collapsed() ? "true" : "false"}
    >
      <aside class="bots-list-rail" aria-label="Bots">
        <header class="bots-list-head">
          <div>
            <span class="bots-eyebrow">teammates</span>
            <h1>Bots</h1>
          </div>
          <button
            type="button"
            class="bots-collapse"
            aria-label={collapsed() ? "Expand bot list" : "Collapse bot list"}
            onClick={() => setCollapsed((value) => !value)}
          >
            {collapsed() ? "→" : "←"}
          </button>
        </header>
        <div class="bots-list-actions">
          <Button variant="primary" data-testid="new-bot" onClick={openNewBot}>
            + New bot
          </Button>
          <Button
            variant="secondary"
            data-testid="open-routines"
            onClick={() => setRoutinesOpen(true)}
          >
            Routines
          </Button>
        </div>
        <div class="bots-list" data-testid="bot-list">
          <Show
            when={bots().length > 0}
            fallback={
              <p class="bots-empty-state" data-testid="bots-empty-state">
                No bots yet. Create one to have a standing agent you can message
                any time and hand recurring work to.
              </p>
            }
          >
            <For each={bots()}>
              {(bot) => (
                <button
                  type="button"
                  class="bot-list-row"
                  data-testid={`bot-row-${bot.id}`}
                  data-selected={selectedId() === bot.id ? "true" : "false"}
                  data-locator={destinationDesktop("bots", project(), bot.id)}
                  onClick={() => setSelectedId(bot.id)}
                >
                  <Avatar
                    seed={bot.id}
                    alt={`${bot.name} avatar`}
                    class="bot-avatar bot-avatar-row"
                    testId={`bot-avatar-${bot.id}`}
                  />
                  <span
                    class={`bot-live-dot bot-${bot.state}`}
                    data-live-state={bot.state}
                    title={bot.state}
                  />
                  <Show when={!collapsed()}>
                    <span class="bot-list-copy">
                      <strong>{bot.name}</strong>
                      <small>
                        {bot.harness} · {bot.lastActivity}
                      </small>
                    </span>
                  </Show>
                </button>
              )}
            </For>
          </Show>
        </div>
        <footer class="bots-list-footer">
          A bot is a named teammate with one conversation and one workspace. It
          is not a task and never touches the board.
        </footer>
      </aside>
      <main class="bots-home-pane" data-testid="bot-home-pane">
        <Show when={props.bots === undefined}>
          <FixtureNotice surface="Bots" command='invoke("bots_list")' />
        </Show>
        <Show when={selected()}>
          {(bot) => (
            <>
              <BotViewHeader bot={bot()} />
              <AgentPane
                runId={bot().runId}
                live={false}
                session={botSession(bot(), project())}
                events={botEvents(bot())}
              />
            </>
          )}
        </Show>
      </main>
      <Sheet open={newBotOpen()} onOpenChange={setNewBotOpen} title="New bot">
        <form
          class="bots-new-bot-form"
          data-testid="new-bot-form"
          onSubmit={submitNewBot}
        >
          <p class="bots-routines-note">
            Add the markdown definition for a standing teammate. The core will
            create its workspace and home session.
          </p>
          <label>
            Bot definition
            <Textarea
              aria-label="Bot definition"
              value={newBotMarkdown()}
              onInput={(event) => setNewBotMarkdown(event.currentTarget.value)}
              rows={8}
            />
          </label>
          <Show when={newBotError()}>
            {(error) => (
              <p role="alert" class="bots-new-bot-error">
                {error()}
              </p>
            )}
          </Show>
          <div class="bots-routine-actions">
            <Button
              type="button"
              variant="ghost"
              onClick={() => setNewBotOpen(false)}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              disabled={creatingBot()}
              aria-busy={creatingBot()}
            >
              {creatingBot() ? "Creating…" : "Create bot"}
            </Button>
          </div>
        </form>
      </Sheet>
      <Sheet
        open={routinesOpen()}
        onOpenChange={setRoutinesOpen}
        title="Routines"
      >
        <RoutinesSheet
          routines={routines()}
          onChange={setRoutines}
          onClose={() => setRoutinesOpen(false)}
        />
      </Sheet>
    </div>
  );
}
