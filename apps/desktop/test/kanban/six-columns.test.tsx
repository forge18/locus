import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import {
  COLUMN_LABELS,
  COLUMN_ORDER,
  SECOND_COLUMN_ALTERNATIVE,
  SECOND_COLUMN_LABEL,
  useTasksByColumn,
} from '../../src/data/board'
import { read, rules } from '../css'

const mount = () => render(() => <KanbanView />)

describe('kanban/six-columns', () => {
  it('renders exactly six', () => {
    const { getByTestId } = mount()
    expect(getByTestId('kanban-columns').querySelectorAll('.kanban-column').length).toBe(6)
    expect(COLUMN_ORDER.length).toBe(6)
  })

  it('is a six-column grid at 9px gaps', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.kanban-columns',
    )!.body
    expect(body).toContain('grid-template-columns: repeat(6, minmax(0, 1fr))')
    expect(body).toContain('gap: var(--g-4)')
    expect(read('styles/tokens.css')).toContain('--g-4: 9px')
  })

  it('labels them in board order', () => {
    const { getByTestId } = mount()
    expect(
      COLUMN_ORDER.map((c) => getByTestId(`kanban-column-head-${c}`).textContent?.replace(/\d+$/, '')),
    ).toEqual([
      'Ready',
      'In Progress',
      'Testing',
      'Reviewing',
      'Waiting For Approval',
      'Done',
    ])
  })

  it('names the second column as PLAN.md does, not as the handoff drew it', () => {
    expect(SECOND_COLUMN_LABEL).toBe('In Progress')
    expect(SECOND_COLUMN_ALTERNATIVE).toBe('Building')
    expect(COLUMN_LABELS.building).toBe(SECOND_COLUMN_LABEL)
  })

  it('counts each column from the tasks in it', () => {
    const { getByTestId } = mount()
    const byColumn = useTasksByColumn()
    for (const column of COLUMN_ORDER) {
      expect(getByTestId(`kanban-count-${column}`).textContent, column).toBe(
        String(byColumn[column].length),
      )
    }
  })

  it('sets the counts in mono, dimmer than the label', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.kanban-column-count',
    )!.body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('color: var(--text-muted)')
  })
})
