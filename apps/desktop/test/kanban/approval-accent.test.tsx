import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { COLUMN_ORDER } from '../../src/data/board'
import { read, rules } from '../css'

const mount = () => render(() => <KanbanView />)

describe('kanban/approval-accent', () => {
  it('accents the Waiting For Approval head', () => {
    const { getByTestId } = mount()
    expect(getByTestId('kanban-column-head-waiting_for_approval').className).toContain(
      'kanban-column-head-approval',
    )
    expect(
      rules(read('screens/screens.css')).find(
        (r) => r.selector === '.kanban-column-head-approval',
      )!.body,
    ).toContain('color: var(--ac)')
  })

  it('accents that head and no other', () => {
    const { getByTestId } = mount()
    for (const column of COLUMN_ORDER) {
      if (column === 'waiting_for_approval') continue
      expect(
        getByTestId(`kanban-column-head-${column}`).className,
        column,
      ).not.toContain('kanban-column-head-approval')
    }
  })

  it('leaves the other heads in --mu', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.kanban-column-head')!.body,
    ).toContain('color: var(--mu)')
  })

  it('is the one column that means "a person is the blocker"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('kanban-column-head-waiting_for_approval').textContent).toContain(
      'Waiting For Approval',
    )
  })
})
