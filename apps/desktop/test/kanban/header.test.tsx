import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { BLOCKED_NOTE, HEADER_NOTE } from '../../src/data/board'

const mount = () => render(() => <KanbanView />)

describe('kanban/header', () => {
  it('says the columns are fixed across every project', () => {
    const { getByTestId } = mount()
    expect(getByTestId('kanban-title').textContent).toBe('Fixed columns across every project')
    expect(HEADER_NOTE).toBe('Fixed columns across every project')
  })

  it('says blocked is a status, not a column', () => {
    const { getByTestId } = mount()
    expect(getByTestId('kanban-blocked-note').textContent).toContain(
      'blocked is a status, not a column',
    )
    expect(BLOCKED_NOTE).toBe('blocked is a status, not a column')
  })

  it('carries the prohibit-inset glyph beside that note', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('kanban-blocked-note').querySelector('use')!.getAttribute('href'),
    ).toBe('#ph-prohibit-inset')
  })

  it('puts both notes above the columns', () => {
    const { getByTestId } = mount()
    const kanban = getByTestId('kanban')
    expect(getByTestId('kanban-head').compareDocumentPosition(getByTestId('kanban-columns')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })
})
