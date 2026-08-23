import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { KanbanView } from '../../src/screens/automate/KanbanView'

describe('task cards', () => {
  it('renders task metadata and review state', () => {
    const { getByTestId } = render(() => <KanbanView />)
    expect(getByTestId('task-card-t-001').textContent).toContain('Gate:')
  })
})
