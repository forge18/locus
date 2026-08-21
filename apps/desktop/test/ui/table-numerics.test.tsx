import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Table, type Column } from '../../src/ui/Table'
import { read, rules } from '../css'

interface Row {
  project: string
  branch: string
  tokens: string
  verify: string
}

const ROWS: Row[] = [
  { project: 'tapestry', branch: 'agent/8f21-notify', tokens: '1.71M', verify: '78%' },
  { project: 'loom-db', branch: 'agent/3c04-index', tokens: '1.09M', verify: '74%' },
]

const COLUMNS: Column<Row>[] = [
  { key: 'project', header: 'Project', cell: (r) => r.project },
  { key: 'branch', header: 'Branch', type: 'mono', cell: (r) => r.branch },
  { key: 'tokens', header: 'Tokens today', type: 'numeric', cell: (r) => r.tokens },
  { key: 'verify', header: 'Verify', type: 'numeric', cell: (r) => r.verify },
]

const mount = () =>
  render(() => <Table columns={COLUMNS} rows={ROWS} rowKey={(r) => r.project} />)

describe('ui/table-numerics', () => {
  it('tags every cell with its column type, header included', () => {
    const { getByTestId } = mount()
    const table = getByTestId('table')
    expect([...table.querySelectorAll('th')].map((th) => th.className)).toEqual([
      'col-text',
      'col-mono',
      'col-numeric',
      'col-numeric',
    ])
    const firstRow = table.querySelector('tbody tr')!
    expect([...firstRow.querySelectorAll('td')].map((td) => td.className)).toEqual([
      'col-text',
      'col-mono',
      'col-numeric',
      'col-numeric',
    ])
  })

  it('right-aligns numerics and leaves mono and text flush left', () => {
    const css = read('ui/ui.css')
    const rule = (sel: string) => rules(css).find((r) => r.selector === sel)
    expect(rule('.table th.col-numeric, .table td.col-numeric')!.body).toContain(
      'text-align: right',
    )
    expect(rule('.table th')!.body).toContain('text-align: left')
    expect(rule('.table th.col-mono')!.body).toContain('text-align: left')
  })

  it('sets numeric and mono cells in --fm without per-screen styling', () => {
    const css = read('ui/ui.css')
    const rule = rules(css).find(
      (r) => r.selector === '.table td.col-numeric, .table td.col-mono',
    )!
    expect(rule.body).toContain('font-family: var(--fm)')
  })

  it('carries no inline style on any cell — the column type does the work', () => {
    const { getByTestId } = mount()
    for (const cell of getByTestId('table').querySelectorAll('td')) {
      expect(cell.getAttribute('style')).toBe(null)
    }
  })

  it('keys rows by identity so a re-sort does not rebuild them', () => {
    const { getByTestId } = mount()
    expect(
      [...getByTestId('table').querySelectorAll('tbody tr')].map((r) =>
        r.getAttribute('data-row-key'),
      ),
    ).toEqual(['tapestry', 'loom-db'])
  })
})
