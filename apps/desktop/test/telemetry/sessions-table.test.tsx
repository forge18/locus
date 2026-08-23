import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { PAGE_SIZE, SESSION_TOTAL, useSessionRowCount } from '../../src/data/telemetry'
import { read, rules } from '../css'

const mount = () => render(() => <TelemetryView />)
const table = () => mount().getByTestId('tm-sessions')

describe('telemetry/sessions-table', () => {
  it('is headed SESSIONS with the total', () => {
    expect(table().textContent).toContain(`Sessions (${SESSION_TOTAL})`)
  })

  it('has the eleven documented columns, in order', () => {
    expect([...table().querySelectorAll('th')].map((th) => th.textContent)).toEqual([
      'When ↓',
      'Harness',
      'Project · repo',
      'Agent · role',
      'Model(s)',
      'Runs',
      'Events',
      'Errors',
      'Tokens',
      'Status',
      'Id',
    ])
  })

  it('sets every numeric column in mono and right-aligned, by column type', () => {
    const row = table().querySelector('tbody tr')!
    const classes = [...row.querySelectorAll('td')].map((td) => td.className)
    expect(classes).toEqual([
      'col-mono', 'col-text', 'col-text', 'col-mono', 'col-mono',
      'col-numeric', 'col-numeric', 'col-numeric', 'col-numeric',
      'col-text', 'col-mono',
    ])
    expect(
      rules(read('ui/ui.css')).find((r) => r.selector === '.table th.col-numeric, .table td.col-numeric')!.body,
    ).toContain('text-align: right')
  })

  it('colours the status: accent running, --bad stuck, --ok closed, --mu waiting', () => {
    const rows = [...table().querySelectorAll('tbody tr')]
    const statusOf = (id: string) =>
      rows.find((r) => r.textContent?.includes(id))!.querySelector('[class^="status-"]')!.className
    expect(statusOf('9cd39051')).toBe('status-running')
    expect(statusOf('a708eae2')).toBe('status-stuck')
    expect(statusOf('a5abc2c9')).toBe('status-closed')
    expect(statusOf('3dc1e427')).toBe('status-waiting')

    const css = rules(read('screens/screens.css'))
    expect(css.find((r) => r.selector === '.status-running')!.body).toContain('color: var(--action-attention)')
    expect(css.find((r) => r.selector === '.status-closed')!.body).toContain('color: var(--status-success)')
    expect(css.find((r) => r.selector === '.status-waiting')!.body).toContain('color: var(--mu)')
  })

  it('reads unknown, not zero, where the harness reported no usage', () => {
    const rows = [...table().querySelectorAll('tbody tr')]
    const texere = rows.find((r) => r.textContent?.includes('texere'))!
    expect(texere.querySelector('.unknown')!.textContent).toBe('unknown')
  })

  it('marks a non-zero error count in --bad', () => {
    const rows = [...table().querySelectorAll('tbody tr')]
    const running = rows.find((r) => r.textContent?.includes('9cd39051'))!
    expect(running.querySelector('.verify-bad')!.textContent).toBe('18')
  })

  it('loads one page of the 300 and renders only the window of it', () => {
    const rows = mount().getByTestId('tm-sessions-table-rows')
    expect(rows.getAttribute('data-total')).toBe(String(useSessionRowCount()))
    expect(rows.getAttribute('data-loaded')).toBe(String(PAGE_SIZE))
    expect(rows.querySelectorAll('tbody tr').length).toBeLessThan(PAGE_SIZE)
    expect(useSessionRowCount()).toBe(SESSION_TOTAL)
  })
})
