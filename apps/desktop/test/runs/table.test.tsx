import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunsView } from '../../src/screens/review/RunsView'
import { PAGE_SIZE, useRunCount } from '../../src/data/runs'

const ROW_HEIGHT = 26
const mount = () => render(() => <RunsView />)
const table = () => mount().getByTestId('runs-table')

describe('runs/table', () => {
  it('is headed RUNS with the full count, not the loaded one', () => {
    const { getByTestId } = mount()
    expect(getByTestId('runs-panel').textContent).toContain(`Runs (${useRunCount()})`)
    expect(useRunCount()).toBe(612)
  })

  it('has the twelve documented columns, in order', () => {
    expect([...table().querySelectorAll('th')].map((th) => th.textContent)).toEqual([
      'When ↓',
      'Harness',
      'Project · repo',
      'Agent · role',
      'Model resolved',
      'Events',
      'Errors',
      'Tokens',
      'Cache',
      'Spend',
      'Verify',
      'Id',
    ])
  })

  it('loads one page and renders only the window of it', () => {
    const rows = mount().getByTestId('runs-table-rows')
    expect(rows.getAttribute('data-total')).toBe('612')
    expect(rows.getAttribute('data-loaded')).toBe(String(PAGE_SIZE))
    expect(rows.querySelectorAll('tbody tr').length).toBeLessThan(PAGE_SIZE)
  })

  it('spaces out the rows it did not render, so the scrollbar is the whole list', () => {
    const { getByTestId } = mount()
    const last = Number(getByTestId('runs-table-rows').getAttribute('data-last'))
    expect((getByTestId('virtual-spacer-bottom') as HTMLElement).style.height).toBe(
      `${(612 - last) * ROW_HEIGHT}px`,
    )
  })

  it('loads the next page as the window reaches the end of what it has', () => {
    const { getByTestId } = mount()
    const rows = getByTestId('runs-table-rows')
    expect(rows.getAttribute('data-loaded')).toBe(String(PAGE_SIZE))

    // Scroll past the loaded page; the table asks for the next one.
    Object.defineProperty(rows, 'scrollTop', { value: 90 * ROW_HEIGHT, writable: true })
    rows.dispatchEvent(new Event('scroll'))
    expect(Number(rows.getAttribute('data-loaded'))).toBeGreaterThan(PAGE_SIZE)
  })

  it('sets the model column in mono, because a model id is an identifier', () => {
    const row = table().querySelector('tbody tr')!
    expect([...row.querySelectorAll('td')][4].className).toBe('col-mono')
  })

  it('colours a passing verify --ok and a failing one --bad', () => {
    const rows = [...table().querySelectorAll('tbody tr')]
    expect(rows.some((r) => r.querySelector('.verify-ok'))).toBe(true)
    expect(rows.some((r) => r.querySelector('.verify-bad'))).toBe(true)
  })

  it('reads unknown, not zero, where a run reported no usage', () => {
    const rows = [...table().querySelectorAll('tbody tr')]
    expect(rows.some((r) => r.querySelector('.unknown'))).toBe(true)
  })
})
