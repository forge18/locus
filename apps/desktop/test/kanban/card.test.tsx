import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TaskCard } from '../../src/screens/automate/TaskCard'
import { useTasks } from '../../src/data/board'
import { read, rules } from '../css'

const task = useTasks().find((t) => t.id === 't-004')!
const mount = () => render(() => <TaskCard task={task} />)

describe('kanban/card', () => {
  it('shows the title at 14px', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`task-card-${task.id}`).textContent).toContain(task.title)
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.task-card-title')!.body,
    ).toContain('font-size: var(--t-body)')
  })

  it('names the project in accent and the repo beside it', () => {
    const { getByTestId } = mount()
    const card = getByTestId(`task-card-${task.id}`)
    expect(card.querySelector('.task-card-project')!.textContent).toBe('tapestry')
    expect(card.textContent).toContain('tapestry-app')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.task-card-project')!.body,
    ).toContain('color: var(--action-attention)')
  })

  it('shows the verify command in mono — it is what decides done', () => {
    const { getByTestId } = mount()
    const mono = getByTestId(`task-card-${task.id}`).querySelector('.mono')!
    expect(mono.textContent).toBe('cargo test -p tapestry-core supervisor::')
  })

  it('names the assignee and what it may reach', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`task-card-${task.id}`).textContent).toContain('builder@4 · read-only tools')
  })

  it('names the gate', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`task-card-${task.id}`).textContent).toContain('Gate: reviewer agent')
  })

  it('sets the meta line at 13px in --mu2', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.task-card-meta',
    )!.body
    expect(body).toContain('font-size: var(--t-meta)')
    expect(body).toContain('color: var(--text-muted)')
  })

  it('omits the assignee line where there is none', () => {
    const unassigned = useTasks().find((t) => t.assignee === null)!
    const { getByTestId } = render(() => <TaskCard task={unassigned} />)
    expect(getByTestId(`task-card-${unassigned.id}`).textContent).not.toContain('read-only tools')
  })
})
