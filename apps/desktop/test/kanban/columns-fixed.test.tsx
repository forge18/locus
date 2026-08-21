import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { CATEGORIES } from '../../src/nav'
import { COLUMN_ORDER } from '../../src/data/board'
import { read } from '../css'

const mount = () => render(() => <KanbanView />)

describe('kanban/columns-fixed', () => {
  it('offers no add-column control', () => {
    const { getByTestId } = mount()
    const labels = [...getByTestId('kanban').querySelectorAll('button')].map((b) =>
      (b.textContent ?? '').toLowerCase() + (b.getAttribute('aria-label') ?? '').toLowerCase(),
    )
    for (const label of labels) {
      expect(label).not.toMatch(/add|new column|\+ column/)
    }
  })

  it('offers no remove or reorder control either', () => {
    const { getByTestId } = mount()
    const kanban = getByTestId('kanban')
    expect(kanban.querySelectorAll('[draggable="true"]').length).toBe(0)
    expect(kanban.querySelectorAll('[aria-label*="Remove"]').length).toBe(0)
    expect(kanban.querySelectorAll('[aria-label*="Reorder"]').length).toBe(0)
  })

  it('names the columns in one frozen constant, not per project', () => {
    expect(COLUMN_ORDER).toEqual([
      'ready', 'building', 'testing', 'reviewing', 'waiting_for_approval', 'done',
    ])
    // The same discipline as the seven categories: the list is closed.
    expect(CATEGORIES.length).toBe(7)
  })

  it('takes no column list as a prop — there is nothing to vary', () => {
    const source = read('screens/automate/KanbanView.tsx')
    expect(source).not.toMatch(/columns:|props\.columns/)
    expect(source).toContain('COLUMN_ORDER')
  })

  it('renders the same six whatever the project filter is', () => {
    const { getByTestId, unmount } = mount()
    const first = [...getByTestId('kanban-columns').querySelectorAll('.kanban-column')].map((c) =>
      c.getAttribute('data-testid'),
    )
    unmount()
    const second = mount()
    expect(
      [...second.getByTestId('kanban-columns').querySelectorAll('.kanban-column')].map((c) =>
        c.getAttribute('data-testid'),
      ),
    ).toEqual(first)
  })
})
