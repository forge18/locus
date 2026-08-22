import { For, createMemo, createSignal } from 'solid-js'
import { Button } from '../../ui/Button'
import { Input } from '../../ui/Input'
import { PLAN_GRANULARITY_OPTIONS, PLAN_TASKS } from '../../data/plan'
import type { PlanGranularity, PlanTask } from '../../data/plan'

function cardCount(granularity: PlanGranularity, carvedOut: string[]): number {
  if (granularity === 'spec') return 1
  if (granularity === 'every-task') return PLAN_TASKS.length
  return carvedOut.length + 1
}

export function PlanTasksView() {
  const [granularity, setGranularity] = createSignal<PlanGranularity>('spec-carve-outs')
  const [tasks, setTasks] = createSignal(PLAN_TASKS)
  const [carvedOut, setCarvedOut] = createSignal(['T-02', 'T-03'])
  const count = createMemo(() => cardCount(granularity(), carvedOut()))

  const updateTask = (id: string, field: keyof PlanTask, value: string) => {
    setTasks((current) => current.map((task) => (task.id === id ? { ...task, [field]: value } : task)))
  }

  const ownsCard = (id: string) => granularity() === 'every-task' || (granularity() === 'spec-carve-outs' && carvedOut().includes(id))

  const toggleTask = (id: string) => {
    if (granularity() !== 'spec-carve-outs') return
    setCarvedOut((current) => current.includes(id) ? current.filter((item) => item !== id) : [...current, id])
  }

  const cardSummary = () => {
    if (granularity() === 'spec') return 'one card, decomposed by the agent at run time'
    if (granularity() === 'every-task') return 'dependencies carried onto the board'
    return 'the spec, plus the ones you want to watch'
  }

  return (
    <section class="plan-tasks" data-testid="plan-tasks">
      <header class="plan-tasks-head">
        <div>
          <span class="plan-stage-label">Stage 8 of 9</span>
          <h2>Decompose</h2>
          <span class="plan-stage-note">decided once, before approval</span>
        </div>
        <p>A card is what the board schedules and what you watch progress on. Too coarse and you cannot see where a run got stuck; too fine and you are managing a to-do list instead of a plan.</p>
      </header>

      <div class="plan-tasks-body">
        <section>
          <h3 class="plan-group-title">What becomes a card</h3>
          <div class="plan-granularity-options" role="radiogroup" aria-label="Card granularity">
            <For each={PLAN_GRANULARITY_OPTIONS}>
              {(option) => (
                <button
                  type="button"
                  class="plan-granularity-option"
                  data-testid={`granularity-${option.id}`}
                  role="radio"
                  aria-checked={granularity() === option.id}
                  data-selected={granularity() === option.id || undefined}
                  onClick={() => setGranularity(option.id)}
                >
                  <span class="plan-radio" aria-hidden="true" />
                  <span class="plan-granularity-label">{option.label}</span>
                  <span class="plan-granularity-detail">{option.detail}</span>
                  <span class="plan-granularity-yield">{option.yield} · {option.id === 'spec' ? '1 card' : option.id === 'every-task' ? `${PLAN_TASKS.length} cards` : `${count()} cards`}</span>
                </button>
              )}
            </For>
          </div>
        </section>

        <section>
          <div class="plan-task-table-heading">
            <h3 class="plan-group-title">Tasks from the spec</h3>
            <span>{granularity() === 'spec' ? 'all four ride along on the spec card' : granularity() === 'every-task' ? 'every task gets its own card' : `${carvedOut().length} carved out, ${tasks().length - carvedOut().length} riding along`}</span>
            <Button variant="ghost">+ Add task</Button>
          </div>
          <div class="plan-task-table">
            <div class="plan-task-row plan-task-columns plan-task-table-labels">
              <span /> <span>Task</span><span>Role</span><span>Estimate</span><span>After</span><span>On the board as</span>
            </div>
            <For each={tasks()}>
              {(task) => (
                <div class="plan-task-row plan-task-columns" data-testid={`plan-task-${task.id}`}>
                  <span class="mono plan-task-id">{task.id}</span>
                  <Input value={task.title} aria-label={`${task.id} title`} onInput={(event) => updateTask(task.id, 'title', event.currentTarget.value)} />
                  <Input mono value={task.role} aria-label={`${task.id} role`} onInput={(event) => updateTask(task.id, 'role', event.currentTarget.value)} />
                  <Input mono value={task.estimate} aria-label={`${task.id} estimate`} onInput={(event) => updateTask(task.id, 'estimate', event.currentTarget.value)} />
                  <Input mono value={task.dependency} aria-label={`${task.id} dependency`} onInput={(event) => updateTask(task.id, 'dependency', event.currentTarget.value)} />
                  <button type="button" class="plan-task-card-toggle" data-testid={`task-card-toggle-${task.id}`} disabled={granularity() !== 'spec-carve-outs'} onClick={() => toggleTask(task.id)}>
                    {granularity() === 'spec' ? 'inside the spec card' : ownsCard(task.id) ? 'its own card' : 'rides on the spec card'}
                  </button>
                </div>
              )}
            </For>
            <footer class="plan-task-summary">
              <span class="plan-card-total" data-testid="task-card-count">{count()} {count() === 1 ? 'card' : 'cards'}</span>
              <span>{cardSummary()}</span>
              <Button variant="primary">Approve — {count()} to the board</Button>
            </footer>
          </div>
          <p class="plan-tasks-note">Anything not carved out rides along on the spec card, and the agent decomposes it at run time. Carve out the ones you want to watch separately — usually the long ones and the ones you expect to get stuck.</p>
        </section>
      </div>
    </section>
  )
}
