import { For, Show, createMemo, createSignal } from 'solid-js'
import { Button } from '../../ui/Button'
import { Textarea } from '../../ui/Input'
import {
  MAIL_HANDOFF_COPY,
  MAIL_MESSAGES,
  MAIL_PARTICIPANT_NOTE,
  MAIL_PARTICIPANTS,
  MAIL_STORAGE_COPY,
  MAIL_TABS,
  MAIL_THREADS,
  MAIL_VERBS,
  MAIL_WAIT_BANNER,
  MAIL_WAIT_INVARIANT,
  MAIL_WAIT_LIVE_LINE,
  SELECTED_MAIL_THREAD_ID,
} from '../../data/mail'
import type { MailStatus } from '../../data/mail'

export interface MailViewProps {
  /** Optional selection seam for a locator or an inbox preview. */
  threadId?: string
}

const statusLabel = (status: MailStatus) => status

/**
 * Mail is deliberately a separate three-pane view. It is stored communication,
 * not a transcript, so a harness swap does not make the thread disappear.
 */
export function MailView(props: MailViewProps = {}) {
  const [selectedId, setSelectedId] = createSignal(props.threadId ?? SELECTED_MAIL_THREAD_ID)
  const [tab, setTab] = createSignal<(typeof MAIL_TABS)[number]>('All')
  const selected = createMemo(() => MAIL_THREADS.find((thread) => thread.id === selectedId()) ?? MAIL_THREADS[0])
  const visibleThreads = createMemo(() => {
    const currentTab = tab()
    if (currentTab === 'Waiting') return MAIL_THREADS.filter((thread) => thread.status === 'waiting')
    if (currentTab === 'To you') return MAIL_THREADS.filter((thread) => thread.status === 'you' || thread.to === 'you')
    return MAIL_THREADS
  })
  const messages = createMemo(() => MAIL_MESSAGES.filter((message) => message.threadId === selected().id))
  const selectedIsWaiting = () => selected().status === 'waiting'
  const selectedIsDrained = () => selected().status === 'drained'

  return (
    <main class="mail" data-testid="mail" data-three-pane="true">
      <aside class="mail-left">
        <nav class="mail-tabs" aria-label="Mail filters">
          <For each={MAIL_TABS}>
            {(item) => <button type="button" class="mail-tab" aria-selected={tab() === item} onClick={() => setTab(item)}>{item}</button>}
          </For>
        </nav>
        <div class="mail-thread-list" aria-label="Mail threads">
          <For each={visibleThreads()}>
            {(thread) => <button type="button" class="mail-thread" data-testid={`mail-thread-${thread.id}`} data-status={thread.status} aria-selected={selectedId() === thread.id} onClick={() => setSelectedId(thread.id)}>
              <span class="mail-thread-head"><strong>{thread.subject}</strong><span class={`mail-status mail-status-${thread.status}`}>{statusLabel(thread.status)}</span></span>
              <span class="mail-thread-project">#{thread.project} · {thread.from} → {thread.to}</span>
              <small>{thread.blocking ?? `${thread.messageCount} messages`}</small>
            </button>}
          </For>
        </div>
      </aside>

      <section class="mail-center" data-testid="mail-thread-view">
        <header class="mail-center-head"><span>#{selected().project}</span><h1>{selected().subject}</h1><small>{selected().from} → {selected().to}</small></header>
        <Show when={selectedIsWaiting()}>
          <div class="mail-wait-banner" data-testid="mail-wait-banner">
            <strong>{MAIL_WAIT_BANNER}</strong>
            <span>{MAIL_WAIT_INVARIANT}</span>
          </div>
        </Show>
        <div class="mail-messages">
          <For each={messages()}>
            {(message) => <article class="mail-message" data-verb={message.verb}>
              <header><strong>{message.from}</strong><span>mail {message.verb}</span></header>
              <p>{message.body}</p>
            </article>}
          </For>
          <Show when={selectedIsWaiting()}><p class="mail-live-line">{MAIL_WAIT_LIVE_LINE}</p></Show>
        </div>
        <footer class="mail-composer">
          <span>Reply as yourself</span>
          <Textarea disabled={selectedIsDrained()} placeholder={selectedIsDrained() ? 'This thread is a handoff and accepts no new mail verbs.' : 'Write a reply…'} />
          <div><Button variant="primary" disabled={selectedIsDrained()} data-testid="mail-send">Reply</Button><Button disabled={selectedIsDrained()} data-testid="mail-drain">Drain</Button><Show when={selectedIsWaiting()}><Button data-testid="mail-unblock">Unblock</Button></Show></div>
        </footer>
      </section>

      <aside class="mail-right">
        <section class="mail-side-section"><h2>Participants</h2><p>{MAIL_PARTICIPANT_NOTE}</p><For each={MAIL_PARTICIPANTS}>{(participant) => <div class="mail-participant"><strong>{participant.name}</strong><code>{participant.runId}</code><span class={`mail-status mail-status-${participant.state}`}>{participant.state}</span></div>}</For></section>
        <section class="mail-side-section"><h2>Verbs used</h2><div class="mail-verbs"><For each={MAIL_VERBS}>{(verb) => <span> {verb} </span>}</For><span>drain</span></div></section>
        <section class="mail-side-section"><h2>What this becomes</h2><p>Mail is a message between agents that both keep working. {MAIL_HANDOFF_COPY}.</p><Show when={selectedIsDrained()}><div class="mail-handoff" data-testid="mail-handoff-artifact"><strong>handoff artifact · payload ready</strong><small>done · remaining · attempted · decisions · open</small></div></Show></section>
        <section class="mail-side-section"><h2>Why you can read this</h2><p>{MAIL_STORAGE_COPY}</p></section>
      </aside>
    </main>
  )
}

export { MailView as MailFixture }
export default MailView
