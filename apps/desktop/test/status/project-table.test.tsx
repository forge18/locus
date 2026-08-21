import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { StatusView } from '../../src/screens/status/StatusView'
import { useProjectRows } from '../../src/data/status'
import { read, rules } from '../css'

const mount = () => render(() => <StatusView />)
const table = () => mount().getByTestId('project-table')

describe('status/project-table', () => {
  it('has the eight documented columns, in order', () => {
    expect([...table().querySelectorAll('th')].map((th) => th.textContent)).toEqual([
      'Project',
      'Repos',
      'Running',
      'In review',
      'Verify',
      'Tokens today',
      'Cache',
      'Last event',
    ])
  })

  it('has one row per project', () => {
    expect(table().querySelectorAll('tbody tr').length).toBe(useProjectRows().length)
  })

  it('colors a passing verify --ok and a failing one --bad', () => {
    const rows = [...table().querySelectorAll('tbody tr')]
    const weaver = rows.find((r) => r.textContent?.includes('weaver'))!
    const tapestry = rows.find((r) => r.textContent?.includes('tapestry'))!
    expect(weaver.querySelector('.verify-bad')!.textContent).toBe('44%')
    expect(tapestry.querySelector('.verify-ok')!.textContent).toBe('78%')
    const css = rules(read('screens/screens.css'))
    expect(css.find((r) => r.selector === '.verify-ok')!.body).toContain('color: var(--ok)')
    expect(css.find((r) => r.selector === '.verify-bad')!.body).toContain('color: var(--bad)')
  })

  it('sets every numeric column in mono and right-aligned, by column type', () => {
    const row = table().querySelector('tbody tr')!
    const classes = [...row.querySelectorAll('td')].map((td) => td.className)
    expect(classes).toEqual([
      'col-text',
      'col-numeric',
      'col-numeric',
      'col-numeric',
      'col-numeric',
      'col-numeric',
      'col-numeric',
      'col-text',
    ])
    const ui = rules(read('ui/ui.css'))
    expect(
      ui.find((r) => r.selector === '.table td.col-numeric, .table td.col-mono')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('carries no inline style on a cell — the column type does the work', () => {
    for (const td of table().querySelectorAll('td')) {
      expect(td.getAttribute('style')).toBe(null)
    }
  })
})
