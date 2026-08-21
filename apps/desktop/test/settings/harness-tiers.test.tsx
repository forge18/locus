import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import * as settings from '../../src/data/settings'
import type { ModelTier } from '../../src/types/core'

interface HarnessTierRow {
  name: string
  models: string[] | null
  tiers: { tier: ModelTier; model: string | null }[]
}

const useHarnessTierGrid = (settings as unknown as {
  useHarnessTierGrid: () => HarnessTierRow[]
}).useHarnessTierGrid

const mount = () => render(() => <HarnessesView />)

describe('settings/harness-tiers', () => {
  it('reads one four-tier row per registered harness through the tier-grid adapter', () => {
    const { getByTestId } = mount()
    const grid = useHarnessTierGrid()

    expect(grid).toHaveLength(12)
    for (const harness of grid) {
      expect(getByTestId(`settings-harness-${harness.name}`)).toBeTruthy()
      expect(harness.tiers.map(({ tier }) => tier)).toEqual(['low', 'medium', 'high', 'xhigh'])
    }
  })

  it('uses discovered choices only where list_argv is available and preserves free text elsewhere', () => {
    const { getByTestId } = mount()
    const grid = useHarnessTierGrid()

    expect(grid.find((harness) => harness.name === 'aider')?.models).not.toBe(null)
    expect(grid.find((harness) => harness.name === 'claude')?.models).toBe(null)
    expect(getByTestId('settings-tier-aider-low').getAttribute('data-editor')).toBe('combobox')
    expect(getByTestId('settings-tier-claude-low').getAttribute('data-editor')).toBe('free-text')
  })

  it('keeps an unset tier null so the harness retains its own default', () => {
    const grid = useHarnessTierGrid()
    expect(
      grid.find((harness) => harness.name === 'claude')?.tiers.find(({ tier }) => tier === 'xhigh')
        ?.model,
    ).toBe(null)
  })
})
