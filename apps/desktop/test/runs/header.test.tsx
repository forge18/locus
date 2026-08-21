import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunsView } from '../../src/screens/review/RunsView'
import { DEFAULT_RANGE, RANGES, SEARCH_NOTE, useRuns } from '../../src/data/runs'

const mount = () => render(() => <RunsView />)

describe('runs/header', () => {
  it('says what the search covers', () => {
    const { getByTestId } = mount()
    expect(getByTestId('runs-search-note').textContent).toBe(SEARCH_NOTE)
    expect(SEARCH_NOTE).toBe('a path, a tool name, an event verb')
  })

  it('offers the three ranges as a segmented control', () => {
    const { getByTestId, container } = mount()
    expect(container.querySelector('.seg')).not.toBe(null)
    for (const range of RANGES) {
      expect(getByTestId('runs-head').textContent, range.label).toContain(range.label)
    }
  })

  it('opens on 30d', () => {
    const { container } = mount()
    const checked = container.querySelector('.seg-opt[data-checked]')!
    expect(checked.textContent).toContain('30d')
    expect(DEFAULT_RANGE).toBe('30d')
  })

  it('changes range when another segment is picked', () => {
    const { container } = mount()
    const today = container.querySelector('input[value="today"]') as HTMLInputElement
    today.click()
    expect(container.querySelector('.seg-opt[data-checked]')!.textContent).toContain('Today')
  })

  it('counts the runs from the data', () => {
    const { getByTestId } = mount()
    expect(getByTestId('runs-count').textContent).toBe(
      `${useRuns().length.toLocaleString('en-US')} runs`,
    )
  })
})
