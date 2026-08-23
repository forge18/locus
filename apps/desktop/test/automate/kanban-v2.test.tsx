import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { KanbanView } from '../../src/screens/automate/KanbanView'

describe('selected project Kanban', () => {
  it('uses the fixed board columns', () => {
    const { getByTestId } = render(() => <KanbanView />)
    expect(getByTestId('kanban-columns').querySelectorAll('.kanban-column')).toHaveLength(6)
  })
})
