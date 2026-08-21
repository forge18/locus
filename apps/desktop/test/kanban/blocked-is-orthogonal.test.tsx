import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { TaskCard } from '../../src/screens/automate/TaskCard'
import { COLUMN_ORDER, useTasks } from '../../src/data/board'
import type { BoardColumn } from '../../src/data/board'

const blocked = useTasks().find((t) => t.status === 'blocked')!

describe('kanban/blocked-is-orthogonal', () => {
  it('renders a blocked card in every one of the six columns', () => {
    for (const column of COLUMN_ORDER) {
      const { getByTestId, unmount } = render(() => (
        <TaskCard task={{ ...blocked, column: column as BoardColumn }} />
      ))
      const card = getByTestId(`task-card-${blocked.id}`)
      expect(card.getAttribute('data-column'), column).toBe(column)
      expect(card.getAttribute('data-status'), column).toBe('blocked')
      expect(card.querySelector('use')!.getAttribute('href'), column).toBe('#ph-prohibit-inset')
      unmount()
    }
  })

  it('has no blocked column to move it to', () => {
    expect(COLUMN_ORDER as readonly string[]).not.toContain('blocked')
  })

  it('keeps status and column as separate fields', () => {
    for (const task of useTasks()) {
      expect(COLUMN_ORDER as readonly string[], task.id).toContain(task.column)
      expect(['ok', 'blocked', 'stuck'], task.id).toContain(task.status)
    }
  })

  it('leaves the count of the column it sits in unchanged by being blocked', () => {
    const { getByTestId } = render(() => <KanbanView />)
    const inColumn = useTasks().filter((t) => t.column === blocked.column).length
    expect(getByTestId(`kanban-count-${blocked.column}`).textContent).toBe(String(inColumn))
  })
})
