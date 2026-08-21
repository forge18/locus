import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunsView } from '../../src/screens/review/RunsView'
import { useRunStats } from '../../src/data/runs'
import { read, rules } from '../css'

const mount = () => render(() => <RunsView />)

describe('runs/stats', () => {
  it('shows exactly three', () => {
    const { getByTestId } = mount()
    expect(getByTestId('runs-stats').querySelectorAll('.run-stat').length).toBe(3)
    expect(useRunStats().length).toBe(3)
  })

  it('names spec-gap rate, noise reclassified and tokens per passing run', () => {
    expect(useRunStats().map((s) => s.label)).toEqual([
      'spec-gap rate',
      'noise reclassified',
      'tokens per passing run',
    ])
  })

  it('shows each value in mono above its label', () => {
    const { getByTestId } = mount()
    const stat = getByTestId('run-stat-spec-gap-rate')
    expect(stat.querySelector('.run-stat-value')!.textContent).toBe('11%')
    expect(stat.querySelector('.run-stat-label')!.textContent).toBe('spec-gap rate')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.run-stat-value')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('right-aligns the group', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.runs-stats')!.body,
    ).toContain('margin-left: auto')
  })

  it('reads each one off a column that already exists', () => {
    for (const stat of useRunStats()) {
      expect(stat.note.length, stat.label).toBeGreaterThan(0)
    }
  })
})
