import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import { Avatar } from "../../avatars/Avatar";
import { AgentPane, type AgentPaneSession } from "../../panes/AgentPane";
import { destinationDesktop } from "../../nav/desktop-navigation";
import {
  botRoutines,
  botsList,
  createBot,
  deleteBotRoutine,
  sendBotPrompt,
  setBotRoutineEnabled,
  testBotRoutine,
  updateBotRoutine,
  type Bot,
  type BotRoutine,
} from "../../data/bots";
import { dataProvider } from "../../data/provider";
import { ready, type Envelope } from "../../data/envelope";
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
  activeRunId?: string | null;
  cost?: string | null;
}

export interface RoutineViewModel {
  id: string;
  prompt: string;
  cron: string;
  enabled: boolean;
  skipped: number;
}

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
    description: "Created from the Workers workspace.",
    harness: bot.harness ?? "unknown",
    lastActivity: bot.lastActivityAt ?? "never",
    state: bot.containerState === "running" ? "working" : "idle",
    runId: bot.homeSessionId,
    activeRunId: bot.activeRunId,
    cost:
      bot.totalCostMicros == null
        ? null
        : `$${(bot.totalCostMicros / 1_000_000).toFixed(2)}`,
  };
}

function botSession(bot: BotViewModel, project: string): AgentPaneSession {
  return {
    project,
    agent: bot.name,
    model: "unavailable",
    harness: bot.harness,
    effort: "unavailable",
    name: `${bot.name} home session`,
    cost: bot.cost ?? undefined,
    permissionPosture: "bypass",
    status: bot.state,
  };
}

function routineViewModel(routine: BotRoutine): RoutineViewModel {
  return {
    id: routine.id,
    prompt: routine.prompt,
    cron: routine.cronExpression,
    enabled: routine.enabled,
    skipped: routine.skippedCount,
  };
}

function botViewModels(bots: Bot[]): BotViewModel[] {
  return bots.map(toBotViewModel);
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

function RoutinesSheet(props: {
  routines: readonly RoutineViewModel[];
  onChange: (routines: RoutineViewModel[]) => void;
  onClose: () => void;
  live: boolean;
  projectId: string;
}) {
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [draftPrompt, setDraftPrompt] = createSignal("");
  const [testRun, setTestRun] = createSignal<string | null>(null);
  const [mutationError, setMutationError] = createSignal<string | null>(null);

  const report = async <T,>(operation: Promise<Envelope<T>>) => {
    const result = await operation;
    if (result.status === "failed") setMutationError(result.error.message);
  };
  const beginEdit = (routine: RoutineViewModel) => {
    setEditingId(routine.id);
    setDraftPrompt(routine.prompt);
    setMutationError(null);
  };
  const saveEdit = (routine: RoutineViewModel) => {
    const prompt = draftPrompt() || routine.prompt;
    props.onChange(
      props.routines.map((candidate) =>
        candidate.id === routine.id ? { ...candidate, prompt } : candidate,
      ),
    );
    setEditingId(null);
    if (props.live)
      void report(
        updateBotRoutine(props.projectId, routine.id, prompt, routine.cron),
      );
  };
  const toggle = (routine: RoutineViewModel) => {
    const enabled = !routine.enabled;
    props.onChange(
      props.routines.map((candidate) =>
        candidate.id === routine.id ? { ...candidate, enabled } : candidate,
      ),
    );
    if (props.live)
      void report(setBotRoutineEnabled(props.projectId, routine.id, enabled));
  };
  const remove = (routine: RoutineViewModel) => {
    props.onChange(
      props.routines.filter((candidate) => candidate.id !== routine.id),
    );
    if (props.live) void report(deleteBotRoutine(props.projectId, routine.id));
  };
  const test = (routine: RoutineViewModel) => {
    setMutationError(null);
    if (!props.live) {
      setTestRun(routine.id);
      return;
    }
    void testBotRoutine(props.projectId, routine.id).then((result) => {
      if (result.status === "failed") setMutationError(result.error.message);
      if (result.status === "ready") setTestRun(routine.id);
    });
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
      <Show when={mutationError()}>
        {(error) => <p role="alert">{error()}</p>}
      </Show>
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
                    onClick={() => test(routine)}
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
  const liveMode = dataProvider().kind === "live";
  const demoBots =
    dataProvider().read?.<Bot[]>("bots_list", {
      projectId: project(),
    }) ?? [];
  const [botEnvelope, setBotEnvelope] = createSignal<Envelope<Bot[]>>(
    liveMode ? { status: "loading" } : ready(demoBots),
  );
  const [createdBots, setCreatedBots] = createSignal<BotViewModel[]>([]);
  const loadedBots = createMemo(() => {
    if (props.bots) return props.bots;
    const envelope = botEnvelope();
    return envelope.status === "ready" ? botViewModels(envelope.data) : [];
  });
  const bots = createMemo(() => [...loadedBots(), ...createdBots()]);
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
  const [panelError, setPanelError] = createSignal<string | null>(null);
  const demoRoutines =
    dataProvider().read?.<BotRoutine[]>("bot_routines", {
      botId: selectedId(),
    }) ?? [];
  const [routines, setRoutines] = createSignal<RoutineViewModel[]>([
    ...(props.initialRoutines ?? demoRoutines.map(routineViewModel)),
  ]);
  const selected = createMemo(
    () => bots().find((bot) => bot.id === selectedId()) ?? bots()[0],
  );
  const botError = createMemo(() => {
    const envelope = botEnvelope();
    return envelope.status === "failed" ? envelope.error.message : null;
  });

  createEffect(() => {
    const projectId = project();
    if (!liveMode || props.bots) return;
    void botsList(projectId).then(setBotEnvelope);
  });

  createEffect(() => {
    const available = bots();
    const requested = props.botId;
    if (requested && available.some((bot) => bot.id === requested)) {
      if (selectedId() !== requested) setSelectedId(requested);
    } else if (!available.some((bot) => bot.id === selectedId())) {
      setSelectedId(available[0]?.id);
    }
  });

  createEffect(() => {
    const botId = selected()?.id;
    if (!botId || !liveMode || props.initialRoutines) return;
    void botRoutines(project(), botId).then((result) => {
      if (result.status === "ready")
        setRoutines(result.data.map(routineViewModel));
      if (result.status === "empty") setRoutines([]);
    });
  });

  const sendPrompt = (bot: BotViewModel, prompt: string) => {
    if (!liveMode) return;
    setPanelError(null);
    void sendBotPrompt(project(), bot.id, prompt).then((result) => {
      if (result.status === "failed") setPanelError(result.error.message);
    });
  };

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
      const result = await createBot(project(), markdown);
      if (result.status === "failed") throw new Error(result.error.message);
      if (result.status !== "ready") {
        throw new Error("The bot was not returned by the store.");
      }
      const viewModel = toBotViewModel(result.data);
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
      <aside class="bots-list-rail" aria-label="Workers">
        <header class="bots-list-head">
          <div>
            <span class="bots-eyebrow">teammates</span>
            <h1>Workers</h1>
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
            + New worker
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
                  data-locator={destinationDesktop("workers", project(), bot.id)}
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
        <Show when={!liveMode && props.bots === undefined}>
          <FixtureNotice surface="Workers" command='invoke("bots_list")' />
        </Show>
        <Show when={liveMode && botEnvelope().status === "loading"}>
          <p data-testid="bots-loading">Loading workers…</p>
        </Show>
        <Show when={botError()}>
          {(error) => <p role="alert">{error()}</p>}
        </Show>
        <Show when={selected()}>
          {(bot) => (
            <>
              <BotViewHeader bot={bot()} />
              <Show when={panelError()}>
                {(error) => <p role="alert">{error()}</p>}
              </Show>
              <AgentPane
                runId={bot().activeRunId ?? bot().runId}
                live={liveMode && Boolean(bot().activeRunId)}
                showCost
                session={botSession(bot(), project())}
                onSend={(prompt) => sendPrompt(bot(), prompt)}
              />
            </>
          )}
        </Show>
      </main>
      <Sheet open={newBotOpen()} onOpenChange={setNewBotOpen} title="New worker">
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
          live={liveMode}
          projectId={project()}
        />
      </Sheet>
    </div>
  );
}
