import { createSignal } from 'solid-js'
import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Table, type Column } from '../../src/ui/Table'

interface Row { project: string; tokens: string }

const COLUMNS: Column<Row>[] = [
  { key: 'project', header: 'Project', width: '1.4fr', cell: (r) => r.project },
  { key: 'tokens', header: 'Tokens today', type: 'numeric', width: '80px', cell: (r) => r.tokens },
]

const ROWS: Row[] = [{ project: 'tapestry', tokens: '1.71M' }]
const ROW_HEIGHT = 30

describe('ui/table-skeleton', () => {
  it('draws skeleton rows instead of body rows while loading', () => {
    const { getByTestId, queryByTestId } = render(() => (
      <Table columns={COLUMNS} rows={[]} rowKey={(r) => r.project} loading skeletonRows={4} />
    ))
    expect(getByTestId('skeleton-rows').querySelectorAll('.skeleton-row').length).toBe(4)
    expect(queryByTestId('table')!.querySelector('tbody')).toBe(null)
  })

  it('keeps the header up, so the columns do not appear from nowhere', () => {
    const { getByTestId } = render(() => (
      <Table columns={COLUMNS} rows={[]} rowKey={(r) => r.project} loading />
    ))
    expect(getByTestId('table').querySelectorAll('th').length).toBe(2)
  })

  it('draws the skeleton at the same row height the data will use', () => {
    const [loading, setLoading] = createSignal(true)
    const { getByTestId, container } = render(() => (
      <Table
        columns={COLUMNS}
        rows={ROWS}
        rowKey={(r) => r.project}
        loading={loading()}
        rowHeight={ROW_HEIGHT}
      />
    ))
    const skeletonHeight = (
      getByTestId('skeleton-rows').querySelector('.skeleton-row') as HTMLElement
    ).style.height

    setLoading(false)
    const dataHeight = (container.querySelector('tbody tr') as HTMLElement).style.height

    expect(skeletonHeight).toBe(`${ROW_HEIGHT}px`)
    expect(dataHeight).toBe(skeletonHeight)
  })

  it('lays the skeleton out on the column widths the table declared', () => {
    const { getByTestId } = render(() => (
      <Table columns={COLUMNS} rows={[]} rowKey={(r) => r.project} loading />
    ))
    const row = getByTestId('skeleton-rows').querySelector('.skeleton-row') as HTMLElement
    expect(row.style.gridTemplateColumns).toBe('1.4fr 80px')
  })
})
