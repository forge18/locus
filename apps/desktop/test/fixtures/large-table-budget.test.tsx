import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Table, type Column } from '../../src/ui/Table'
import { VirtualTable } from '../../src/panes/VirtualTable'
import { PAGE_SIZE, VIRTUALIZATION_NEEDED, useRunCount, useRunsPage } from '../../src/data/runs'
import { RUN_ROWS } from '../../src/fixtures/runs'
import type { RunRow } from '../../src/fixtures/runs'

const ROW_HEIGHT = 26
const BODY_HEIGHT = 420

const COLUMNS: Column<RunRow>[] = [
  { key: 'id', header: 'Run', type: 'mono', cell: (r) => r.id },
  { key: 'project', header: 'Project', cell: (r) => r.project },
  { key: 'agent', header: 'Agent', cell: (r) => r.agent },
  { key: 'branch', header: 'Branch', type: 'mono', cell: (r) => r.branch },
  { key: 'status', header: 'Status', cell: (r) => r.status },
  { key: 'model', header: 'Model', type: 'mono', cell: (r) => r.model },
  { key: 'tokens', header: 'Tokens', type: 'numeric', cell: (r) => r.tokens ?? 'unknown' },
  { key: 'duration', header: 'Duration', type: 'numeric', cell: (r) => `${r.durationSec}s` },
  { key: 'at', header: 'At', type: 'mono', cell: (r) => r.at },
]

const virtual = () =>
  render(() => (
    <VirtualTable
      columns={COLUMNS}
      rows={useRunsPage(0)}
      total={useRunCount()}
      rowKey={(r) => r.id}
      rowHeight={ROW_HEIGHT}
      height={BODY_HEIGHT}
    />
  ))

describe('fixtures/large-table-budget', () => {
  it('records that virtualization is on', () => {
    expect(VIRTUALIZATION_NEEDED).toBe(true)
  })

  it('opens on one page, not on all 612 rows', () => {
    const { getByTestId } = virtual()
    const rows = getByTestId('table-rows')
    expect(rows.getAttribute('data-total')).toBe('612')
    expect(rows.getAttribute('data-loaded')).toBe(String(PAGE_SIZE))
  })

  it('renders only the window, which is a fraction of the page', () => {
    const { getByTestId } = virtual()
    const rendered = getByTestId('table-rows').querySelectorAll('tbody tr').length
    const window_ = Math.ceil(BODY_HEIGHT / ROW_HEIGHT)
    expect(rendered).toBeGreaterThan(0)
    expect(rendered).toBeLessThan(window_ * 2 + 20)
    expect(rendered).toBeLessThan(RUN_ROWS.length / 4)
  })

  it('costs a fraction of the nodes the full table would', () => {
    const windowed = virtual()
    const windowedNodes = windowed.getByTestId('table').querySelectorAll('*').length
    windowed.unmount()

    const full = render(() => (
      <Table columns={COLUMNS} rows={RUN_ROWS} rowKey={(r) => r.id} />
    ))
    const fullNodes = full.getByTestId('table').querySelectorAll('*').length

    // Both numbers on the record, and the ratio is why the window exists.
    expect(windowedNodes).toBeLessThan(fullNodes / 4)
  })

  it('keeps the scrollbar honest with spacers for the rows it did not render', () => {
    const { getByTestId } = virtual()
    const rows = getByTestId('table-rows')
    const first = Number(rows.getAttribute('data-first'))
    const last = Number(rows.getAttribute('data-last'))
    const top = getByTestId('virtual-spacer-top') as HTMLElement
    const bottom = getByTestId('virtual-spacer-bottom') as HTMLElement

    expect(top.style.height).toBe(`${first * ROW_HEIGHT}px`)
    expect(bottom.style.height).toBe(`${(612 - last) * ROW_HEIGHT}px`)

    // Spacers plus rendered rows add up to the whole list, so the scrollbar is
    // the size it would be if every row were there.
    const total = first * ROW_HEIGHT + (last - first) * ROW_HEIGHT + (612 - last) * ROW_HEIGHT
    expect(total).toBe(612 * ROW_HEIGHT)
  })

  it('says how much is loaded while pages are still coming', () => {
    const { getByTestId } = virtual()
    expect(getByTestId('table-loading').textContent).toContain('100 of 612 loaded')
  })

  it('distinguishes an empty result from an initial loading page', () => {
    const empty = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={[]}
        total={0}
        rowKey={(r) => r.id}
        rowHeight={ROW_HEIGHT}
        height={BODY_HEIGHT}
      />
    ))
    expect(empty.getByTestId('table').getAttribute('data-state')).toBe('empty')
    expect(empty.getByTestId('table-empty').textContent).toContain('No rows to display.')
    empty.unmount()

    const loading = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={[]}
        total={0}
        loading
        rowKey={(r) => r.id}
        rowHeight={ROW_HEIGHT}
        height={BODY_HEIGHT}
      />
    ))
    expect(loading.getByTestId('table').getAttribute('data-state')).toBe('loading')
    expect(loading.getByTestId('table-loading').textContent).toBe('Loading…')
    expect(loading.getByTestId('skeleton-rows')).toBeTruthy()
  })

  it('keeps a backend error distinct from an empty result', () => {
    const { getByTestId, queryByTestId } = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={[]}
        total={0}
        error="Runs unavailable"
        rowKey={(r) => r.id}
        rowHeight={ROW_HEIGHT}
        height={BODY_HEIGHT}
      />
    ))
    expect(getByTestId('table').getAttribute('data-state')).toBe('error')
    expect(getByTestId('inline-error-cause').textContent).toBe('Runs unavailable')
    expect(queryByTestId('table-empty')).toBe(null)
  })
})
