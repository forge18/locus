import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { TaskCard } from '../../src/screens/automate/TaskCard'
import { APPROVAL_NOTE, useTasks } from '../../src/data/board'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const stuck = useTasks().find((t) => t.status === 'stuck')!
const approval = useTasks().find((t) => t.column === 'waiting_for_approval')!
const done = useTasks().find((t) => t.column === 'done')!

describe('kanban/card-variants', () => {
  it('rings a stuck card in red and states the count and the spend', () => {
    const { getByTestId } = render(() => <TaskCard task={stuck} />)
    expect(getByTestId(`task-card-${stuck.id}`).className).toContain('task-card-stuck')
    expect(getByTestId(`task-stuck-${stuck.id}`).textContent).toBe('stuck 3/3 · 102.3k')
    expect(rule('.task-card-stuck').body).toContain('inset 0 0 0 1px var(--status-danger)')
  })

  it('rings a waiting-approval card in accent and says where it belongs', () => {
    const { getByTestId } = render(() => <TaskCard task={approval} />)
    expect(getByTestId(`task-card-${approval.id}`).className).toContain('task-card-approval')
    expect(getByTestId(`task-approval-${approval.id}`).textContent).toBe(APPROVAL_NOTE)
    expect(APPROVAL_NOTE).toBe('an inbox item, not a place to go looking')
    expect(rule('.task-card-approval').body).toContain('box-shadow: var(--ring-sel-soft)')
  })

  it('dims a done card and shows what proves it', () => {
    const { getByTestId } = render(() => (
      <TaskCard task={done} evidence={{ runs: 2, events: 41 }} />
    ))
    expect(getByTestId(`task-card-${done.id}`).className).toContain('task-card-done')
    expect(getByTestId(`task-evidence-${done.id}`).textContent).toBe('evidence: 2 runs, 41 events')
    expect(rule('.task-card-done').body).toMatch(/opacity:\s*\.86/)
    expect(rule('.task-card-evidence').body).toContain('color: var(--status-success)')
  })

  it('wires the evidence through from the board data', () => {
    const { getByTestId } = render(() => <KanbanView />)
    expect(getByTestId(`task-evidence-${done.id}`).textContent).toBe('evidence: 2 runs, 41 events')
  })

  it('shows no evidence line where there is none to show', () => {
    const { queryByTestId } = render(() => <TaskCard task={stuck} />)
    expect(queryByTestId(`task-evidence-${stuck.id}`)).toBe(null)
  })

  it('leaves a plain card plain', () => {
    const plain = useTasks().find((t) => t.id === 't-004')!
    const { getByTestId } = render(() => <TaskCard task={plain} />)
    expect(getByTestId(`task-card-${plain.id}`).className).toBe('task-card')
  })
})
