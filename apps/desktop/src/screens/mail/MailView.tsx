import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import { dataProvider } from "../../data/provider";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { InlineError } from "../../ui/InlineError";
import { Textarea } from "../../ui/Input";
import { PageProjectFilter } from "../PageProjectFilter";
import {
  MAIL_HANDOFF_COPY,
  MAIL_PARTICIPANT_NOTE,
  MAIL_STORAGE_COPY,
  MAIL_TABS,
  MAIL_VERBS,
  MAIL_WAIT_BANNER,
  MAIL_WAIT_INVARIANT,
  MAIL_WAIT_LIVE_LINE,
  SELECTED_MAIL_THREAD_ID,
} from "../../data/mail";
import type {
  MailMessage,
  MailStatus,
  MailThreadFixture,
} from "../../data/mail";
import {
  useMailMessages,
  useMailParticipants,
  useMailThreads,
} from "../../data/mail";

export interface MailViewProps {
  /** Optional page-owned project scope from a compatible deep link. */
  projectId?: string;
  /** Optional selection seam for a locator or an inbox preview. */
  threadId?: string;
}

const statusLabel = (status: MailStatus) => status;

/**
 * Mail is deliberately a separate three-pane view. It is stored communication,
 * not a transcript, so a harness swap does not make the thread disappear.
 */
export function MailView(props: MailViewProps = {}) {
  const [selectedProjectId, setSelectedProjectId] = createSignal(
    props.projectId,
  );
  const threadRows = useMailThreads();
  const participantRows = useMailParticipants();

  if (dataProvider().kind === "live") {
    return (
      <main
        class="mail-view"
        data-testid="mail"
        data-live-state="unavailable"
        data-scope={selectedProjectId() ?? "all"}
      >
        <PageProjectFilter
          value={selectedProjectId()}
          onChange={(projectId) => setSelectedProjectId(projectId)}
          allLabel="All mail"
        />
        <InlineError
          cause="Mail is unavailable"
          next="mail_threads has no persisted desktop contract yet."
        />
      </main>
    );
  }

  const scopedThreads = createMemo(() => {
    const projectId = selectedProjectId();
    return projectId
      ? threadRows.filter((thread) => thread.project === projectId)
      : threadRows;
  });
  const [selectedId, setSelectedId] = createSignal(
    props.threadId ?? scopedThreads()[0]?.id ?? SELECTED_MAIL_THREAD_ID,
  );
  const [tab, setTab] = createSignal<(typeof MAIL_TABS)[number]>("All");
  // The backend does not exist yet (see the fixture notice), so composer actions
  // mutate view-local state on top of the fixture instead of a Tauri command.
  const [draft, setDraft] = createSignal("");
  const [sentReplies, setSentReplies] = createSignal<MailMessage[]>([]);
  const [statusOverrides, setStatusOverrides] = createSignal<
    Partial<Record<string, MailStatus>>
  >({});
  const [composerError, setComposerError] = createSignal<string | null>(null);
  const selected = createMemo<MailThreadFixture>(() =>
    scopedThreads().find((thread) => thread.id === selectedId()) ??
    scopedThreads()[0] ?? {
      id: "",
      project: "",
      status: "open",
      subject: "No mail thread",
      from: "",
      to: "",
      messageCount: 0,
      blocking: null,
    },
  );
  const statusFor = (thread: MailThreadFixture) =>
    statusOverrides()[thread.id] ?? thread.status;
  const selectedStatus = () =>
    statusOverrides()[selectedId()] ?? selected().status;
  const visibleThreads = createMemo(() => {
    const currentTab = tab();
    if (currentTab === "Waiting")
      return scopedThreads().filter((thread) => statusFor(thread) === "waiting");
    if (currentTab === "To you")
      return scopedThreads().filter(
        (thread) => statusFor(thread) === "you" || thread.to === "you",
      );
    return scopedThreads();
  });
  const messages = createMemo(() =>
    [...useMailMessages(selected().id), ...sentReplies()],
  );
  createEffect(() => {
    const current = scopedThreads();
    if (current.length > 0 && !current.some((thread) => thread.id === selectedId())) {
      setSelectedId(current[0].id);
      setDraft("");
      setComposerError(null);
    }
  });
  const selectedIsWaiting = () => selectedStatus() === "waiting";
  const selectedIsDrained = () => selectedStatus() === "drained";

  const selectThread = (threadId: string) => {
    setSelectedId(threadId);
    setDraft("");
    setComposerError(null);
  };

  const sendReply = () => {
    const body = draft().trim();
    if (!body) {
      setComposerError("Write a reply before sending it.");
      return;
    }
    const thread = selected();
    setSentReplies((current) => [
      ...current,
      {
        id: `reply-${current.length + 1}`,
        threadId: thread.id,
        from: "you",
        to: [thread.from],
        body,
        artifactIds: [],
        state: "delivered",
        sentAt: new Date().toISOString(),
        verb: "reply",
      },
    ]);
    setDraft("");
    setComposerError(null);
  };

  const drainThread = () => {
    setStatusOverrides((current) => ({
      ...current,
      [selected().id]: "drained",
    }));
    setComposerError(null);
  };

  const unblockThread = () => {
    setStatusOverrides((current) => ({ ...current, [selected().id]: "open" }));
    setComposerError(null);
  };

  return (
    <main
      class="mail"
      data-testid="mail"
      data-three-pane="true"
      data-scope={selectedProjectId() ?? "all"}
    >
      <aside class="mail-left">
        <PageProjectFilter
          value={selectedProjectId()}
          onChange={(projectId) => setSelectedProjectId(projectId)}
          allLabel="All mail"
        />
        <nav class="mail-tabs" aria-label="Mail filters">
          <For each={MAIL_TABS}>
            {(item) => (
              <button
                type="button"
                class="mail-tab"
                aria-selected={tab() === item}
                onClick={() => setTab(item)}
              >
                {item}
              </button>
            )}
          </For>
        </nav>
        <div class="mail-thread-list" aria-label="Mail threads">
          <For each={visibleThreads()}>
            {(thread) => (
              <button
                type="button"
                class="mail-thread"
                data-testid={`mail-thread-${thread.id}`}
                data-status={statusFor(thread)}
                aria-selected={selectedId() === thread.id}
                onClick={() => selectThread(thread.id)}
              >
                <span class="mail-thread-head">
                  <strong>{thread.subject}</strong>
                  <span class={`mail-status mail-status-${statusFor(thread)}`}>
                    {statusLabel(statusFor(thread))}
                  </span>
                </span>
                <span class="mail-thread-project">
                  #{thread.project} · {thread.from} → {thread.to}
                </span>
                <small>
                  {thread.blocking ?? `${thread.messageCount} messages`}
                </small>
              </button>
            )}
          </For>
        </div>
      </aside>

      <section class="mail-center" data-testid="mail-thread-view">
        <FixtureNotice surface="Mail" command='invoke("mail_threads")' />
        <header class="mail-center-head">
          <span>#{selected().project}</span>
          <h1>{selected().subject}</h1>
          <small>
            {selected().from} → {selected().to}
          </small>
        </header>
        <Show when={selectedIsWaiting()}>
          <div class="mail-wait-banner" data-testid="mail-wait-banner">
            <strong>{MAIL_WAIT_BANNER}</strong>
            <span>{MAIL_WAIT_INVARIANT}</span>
          </div>
        </Show>
        <div class="mail-messages">
          <For each={messages()}>
            {(message) => (
              <article class="mail-message" data-verb={message.verb}>
                <header>
                  <strong>{message.from}</strong>
                  <span>mail {message.verb}</span>
                </header>
                <p>{message.body}</p>
              </article>
            )}
          </For>
          <Show when={selectedIsWaiting()}>
            <p class="mail-live-line">{MAIL_WAIT_LIVE_LINE}</p>
          </Show>
        </div>
        <footer class="mail-composer">
          <span>Reply as yourself</span>
          <Textarea
            value={draft()}
            onInput={(event) => {
              setDraft(event.currentTarget.value);
              setComposerError(null);
            }}
            disabled={selectedIsDrained()}
            placeholder={
              selectedIsDrained()
                ? "This thread is a handoff and accepts no new mail verbs."
                : "Write a reply…"
            }
          />
          <Show when={composerError()}>
            <InlineError
              cause={composerError()!}
              next="Type a reply above, then send it as mail reply from you."
            />
          </Show>
          <div>
            <Button
              variant="primary"
              disabled={selectedIsDrained()}
              data-testid="mail-send"
              onClick={sendReply}
            >
              Reply
            </Button>
            <Button
              disabled={selectedIsDrained()}
              data-testid="mail-drain"
              onClick={drainThread}
            >
              Drain
            </Button>
            <Show when={selectedIsWaiting()}>
              <Button data-testid="mail-unblock" onClick={unblockThread}>
                Unblock
              </Button>
            </Show>
          </div>
        </footer>
      </section>

      <aside class="mail-right">
        <section class="mail-side-section">
          <h2>Participants</h2>
          <p>{MAIL_PARTICIPANT_NOTE}</p>
          <For each={participantRows}>
            {(participant) => (
              <div class="mail-participant">
                <strong>{participant.name}</strong>
                <code>{participant.runId}</code>
                <span class={`mail-status mail-status-${participant.state}`}>
                  {participant.state}
                </span>
              </div>
            )}
          </For>
        </section>
        <section class="mail-side-section">
          <h2>Verbs used</h2>
          <div class="mail-verbs">
            <For each={MAIL_VERBS}>{(verb) => <span> {verb} </span>}</For>
            <span>drain</span>
          </div>
        </section>
        <section class="mail-side-section">
          <h2>What this becomes</h2>
          <p>
            Mail is a message between agents that both keep working.{" "}
            {MAIL_HANDOFF_COPY}.
          </p>
          <Show when={selectedIsDrained()}>
            <div class="mail-handoff" data-testid="mail-handoff-artifact">
              <strong>handoff artifact · payload ready</strong>
              <small>done · remaining · attempted · decisions · open</small>
            </div>
          </Show>
        </section>
        <section class="mail-side-section">
          <h2>Why you can read this</h2>
          <p>{MAIL_STORAGE_COPY}</p>
        </section>
      </aside>
    </main>
  );
}

export default MailView;
