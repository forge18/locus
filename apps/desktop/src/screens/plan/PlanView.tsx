import { For, Match, Show, Switch, createMemo, createSignal, onMount } from 'solid-js'
import { Breadcrumb } from './Breadcrumb'
import { Message } from './Message'
import { PlanList } from './PlanList'
import { Recommendation } from './Recommendation'
import { ScopeDecision } from './ScopeDecision'
import { PlanSpecView } from './PlanSpecView'
import { PlanTasksView } from './PlanTasksView'
import { Icon } from '../../ui/Icon'
import { Tag } from '../../ui/Tag'
import {
  ACP_LABEL,
  useDefaultPlanId,
  usePlanConversation,
  usePlanLiveLine,
  usePlanOutputs,
  subscribePlanConversationFromCore,
  usePlanRecommendation,
  usePlanScopeDecision,
  usePlans,
} from '../../data/plan'

/**
 * A guided conversation that produces a reviewable plan. Nothing reaches the board
 * until one approval at the end, which is why the recommendation has to be legible
 * enough to approve honestly rather than just clickable.
 */
export function PlanView() {
  const [selectedId, setSelectedId] = createSignal(useDefaultPlanId())
  const [tab, setTab] = createSignal<'conversation' | 'spec' | 'tasks'>('conversation')
  const [plansOpen, setPlansOpen] = createSignal(true)
  const [outputsOpen, setOutputsOpen] = createSignal(true)
  const plans = usePlans()
  const selected = createMemo(() => plans.find((p) => p.id === selectedId()) ?? plans[0])

  const [messages, setMessages] = createSignal(usePlanConversation())
  const outputs = usePlanOutputs()

  onMount(() => {
    void subscribePlanConversationFromCore((message) => {
      setMessages((current) => current.some((item) => item.id === message.id) ? current : [...current, message])
    }).catch(() => {
      // Browser tests and the static preview have no Tauri IPC; retain the fixture.
    })
  })

  return (
    <div class="plan-workspace" data-testid="plan">
      <header class="plan-workspace-head">
        <button type="button" class="plan-workspace-toggle" data-testid="toggle-plans" onClick={() => setPlansOpen((open) => !open)}>
          All plans <span class="mono">{plans.length}</span>
        </button>
        <div class="plan-workspace-tabs" data-testid="plan-workspace-tabs" role="tablist" aria-label="Plan view">
          <button type="button" role="tab" data-testid="plan-tab-conversation" aria-selected={tab() === 'conversation'} onClick={() => setTab('conversation')}>Conversation</button>
          <button type="button" role="tab" data-testid="plan-tab-spec" aria-selected={tab() === 'spec'} onClick={() => setTab('spec')}>Spec</button>
          <button type="button" role="tab" data-testid="plan-tab-tasks" aria-selected={tab() === 'tasks'} onClick={() => setTab('tasks')}>Tasks &amp; cards</button>
        </div>
        <button type="button" class="plan-workspace-toggle" data-testid="toggle-outputs" onClick={() => setOutputsOpen((open) => !open)}>
          Outputs <span class="mono">4</span>
        </button>
      </header>
      <div class="plan-summary">
        <span class="plan-convo-title" data-testid="plan-title">{selected().title}</span>
        <span>{selected().project} · started 09:14</span>
        <Breadcrumb current={selected().step} />
      </div>

      <div class="plan">
        <Show when={plansOpen()}>
          <PlanList plans={plans} selectedId={selectedId()} onSelect={setSelectedId} onNewPlan={() => {}} />
        </Show>

        <Switch>
          <Match when={tab() === 'conversation'}>
            <section class="plan-convo" data-testid="plan-conversation">
              <header class="plan-convo-head">
                <span class="plan-stage-label" data-testid="plan-stage-progress">Stage 5 of 9</span>
                <span class="plan-convo-title">Audit</span>
                <span class="plan-convo-running"><span class="live-dot pulse" />running</span>
              </header>
              <div class="plan-messages" data-testid="plan-messages">
                <For each={messages()}>
                  {(message, i) => <>
                    <Message message={message} />
                    <Show when={i() === messages().length - 2}>
                      <ScopeDecision decision={usePlanScopeDecision()} onWiden={() => {}} onKeepOut={() => {}} />
                    </Show>
                  </>}
                </For>
                <div class="plan-live" data-testid="plan-live"><span class="live-dot pulse" data-testid="plan-live-dot" />{usePlanLiveLine()}</div>
              </div>
              <footer class="plan-convo-footer" data-testid="plan-footer">
                <div class="plan-input" data-testid="plan-input">Answer the interviewer…<span class="plan-caret blink" data-testid="plan-caret">|</span></div>
                <span class="plan-acp" data-testid="plan-acp">{ACP_LABEL}</span>
              </footer>
            </section>
          </Match>
          <Match when={tab() === 'spec'}><PlanSpecView /></Match>
          <Match when={tab() === 'tasks'}><PlanTasksView /></Match>
        </Switch>

        <Show when={outputsOpen()}>
          <aside class="plan-outputs" data-testid="plan-outputs">
            <span class="plan-outputs-title">Draft outputs</span>
            <section class="output-card" data-testid="output-spec">
              <div class="output-card-head"><Icon name="file-text" size={12} style={{ color: 'var(--text-secondary)' }} /><span class="mono">{outputs.spec.name}</span><button type="button" class="plan-output-edit" onClick={() => setTab('spec')}>Edit</button></div>
              <For each={outputs.spec.lines}>{(line) => <span class="output-line">{line}</span>}</For>
            </section>
            <section class="output-card" data-testid="output-tasks">
              <div class="output-card-head"><Icon name="list-checks" size={12} style={{ color: 'var(--text-secondary)' }} />tasks<button type="button" class="plan-output-edit" onClick={() => setTab('tasks')}>Edit &amp; decompose</button></div>
              <ol class="output-tasks" data-testid="output-task-list"><For each={outputs.tasks}>{(task) => <li>{task}</li>}</For></ol>
            </section>
            <section class="output-card" data-testid="output-tools">
              <div class="output-card-head"><Icon name="toolbox" size={12} style={{ color: 'var(--text-secondary)' }} />tool list</div>
              <div class="output-tools"><For each={outputs.tools}>{(tool) => <Tag variant="neutral">{tool}</Tag>}</For><For each={outputs.newTools}>{(tool) => <Tag variant="outline" data-testid={`new-tool-${tool.replace(/\W+/g, '')}`}>{tool}</Tag>}</For></div>
            </section>
            <Recommendation recommendation={usePlanRecommendation()} onApprove={() => setTab('tasks')} />
          </aside>
        </Show>
      </div>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default PlanView
