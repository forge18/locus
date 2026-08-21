import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { TaskCard } from '../../src/screens/automate/TaskCard'
import { useTasks } from '../../src/data/board'

const blocked = useTasks().find((t) => t.status === 'blocked')!

describe('kanban/blocked-in-place', () => {
  it('marks the card with the prohibit-inset glyph', () => {
    const { getByTestId } = render(() => <TaskCard task={blocked} />)
    expect(
      getByTestId(`task-card-${blocked.id}`).querySelector('use')!.getAttribute('href'),
    ).toBe('#ph-prohibit-inset')
  })

  it('draws it in --bad', () => {
    const { getByTestId } = render(() => <TaskCard task={blocked} />)
    expect(
      getByTestId(`task-card-${blocked.id}`).querySelector('svg')!.getAttribute('style'),
    ).toContain('var(--bad)')
  })

  it('names the state for a reader who cannot see the glyph', () => {
    const { getByLabelText } = render(() => <TaskCard task={blocked} />)
    expect(getByLabelText('Blocked')).toBeTruthy()
  })

  it('leaves the card in the column its progress puts it in', () => {
    const { getByTestId } = render(() => <KanbanView />)
    const card = getByTestId(`task-card-${blocked.id}`)
    expect(card.getAttribute('data-column')).toBe(blocked.column)
    expect(getByTestId(`kanban-column-${blocked.column}`).contains(card)).toBe(true)
  })

  it('marks nothing on a card that is not blocked', () => {
    const ok = useTasks().find((t) => t.status === 'ok')!
    const { getByTestId } = render(() => <TaskCard task={ok} />)
    const icons = [...getByTestId(`task-card-${ok.id}`).querySelectorAll('use')].map((u) =>
      u.getAttribute('href'),
    )
    expect(icons).not.toContain('#ph-prohibit-inset')
  })
})
