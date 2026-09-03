import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it, vi } from 'vitest'
import { VirtualTable } from '../../src/panes/VirtualTable'
import type { Column } from '../../src/ui/Table'

interface Row {
  id: string
  name: string
}

const ROWS: Row[] = [
  { id: 'r1', name: 'first' },
  { id: 'r2', name: 'second' },
  { id: 'r3', name: 'third' },
]

const COLUMNS: Column<Row>[] = [{ key: 'name', header: 'Name', cell: (r) => r.name }]

const mount = (onRowClick = vi.fn()) => ({
  onRowClick,
  view: render(() => (
    <VirtualTable
      columns={COLUMNS}
      rows={ROWS}
      rowKey={(r) => r.id}
      total={ROWS.length}
      rowHeight={32}
      height={96}
      onRowClick={onRowClick}
    />
  )),
})

describe('panes/virtual-table-a11y', () => {
  it('is one table: the wrapper carries the role and the full row count', () => {
    const { view } = mount()
    const table = view.getByTestId('table')
    expect(table.getAttribute('role')).toBe('table')
    // Header row plus every row in the store, not only the rendered window.
    expect(table.getAttribute('aria-rowcount')).toBe('4')
  })

  it('exposes the header as column headers the body rows can be read against', () => {
    const { view } = mount()
    const table = view.getByTestId('table')
    expect(table.querySelectorAll('th[role="columnheader"]')).toHaveLength(1)
    expect(table.querySelector('thead')!.getAttribute('role')).toBe('rowgroup')
  })

  it('indexes each body row in the sparse table — header is row 1', () => {
    const { view } = mount()
    const row = view.getByTestId('table').querySelector('tbody tr')!
    expect(row.getAttribute('role')).toBe('row')
    expect(row.getAttribute('aria-rowindex')).toBe('2')
    expect(row.querySelectorAll('td[role="cell"]')).toHaveLength(1)
  })

  it('activates a row with Enter or Space, and nothing else', () => {
    const { view, onRowClick } = mount()
    const row = view.getByTestId('table').querySelector('tbody tr') as HTMLElement
    expect(row.tabIndex).toBe(0)
    fireEvent.keyDown(row, { key: 'Enter' })
    expect(onRowClick).toHaveBeenCalledWith(ROWS[0])
    fireEvent.keyDown(row, { key: ' ' })
    expect(onRowClick).toHaveBeenCalledTimes(2)
    fireEvent.keyDown(row, { key: 'a' })
    expect(onRowClick).toHaveBeenCalledTimes(2)
  })

  it('leaves rows out of the tab order when they are not clickable', () => {
    const view = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={ROWS}
        rowKey={(r) => r.id}
        total={ROWS.length}
        rowHeight={32}
        height={96}
      />
    ))
    const row = view.getByTestId('table').querySelector('tbody tr') as HTMLElement
    expect(row.getAttribute('tabindex')).toBe(null)
  })
})
