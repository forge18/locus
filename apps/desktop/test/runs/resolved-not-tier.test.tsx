import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunsView } from '../../src/screens/review/RunsView'
import { MODEL_TIERS, useRuns } from '../../src/data/runs'
import { read } from '../css'

const mount = () => render(() => <RunsView />)

describe('runs/resolved-not-tier', () => {
  it('names the column "Model resolved", not "Model tier"', () => {
    const headers = [...mount().getByTestId('runs-table').querySelectorAll('th')].map(
      (th) => th.textContent,
    )
    expect(headers).toContain('Model resolved')
    expect(headers).not.toContain('Model tier')
  })

  it('holds a model id in every row', () => {
    for (const run of useRuns()) {
      expect(run.model, run.id).toMatch(/^(claude|gpt|gemini)-/)
    }
  })

  it('holds a tier name in no row', () => {
    for (const run of useRuns()) {
      expect(MODEL_TIERS, run.id).not.toContain(run.model)
    }
  })

  it('says why in the source: spend and pass rate attach to what really answered', () => {
    expect(read('fixtures/runs.ts')).toContain('an id, never a tier')
  })

  it('renders the id, not the tier, on screen', () => {
    const cells = [...mount().getByTestId('runs-table').querySelectorAll('tbody tr')]
      .slice(0, 20)
      .map((r) => [...r.querySelectorAll('td')][4].textContent!)
    for (const cell of cells) {
      expect(MODEL_TIERS).not.toContain(cell)
      expect(cell).toMatch(/-/)
    }
  })
})
