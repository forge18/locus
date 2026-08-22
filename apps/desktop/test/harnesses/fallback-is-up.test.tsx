import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { TIERS, TIER_FALLBACK, fallbackMarker, fallbackTierFor, resolveTier } from '../../src/data/settings'
import { useHarnesses } from '../../src/data/harnesses'
import { read } from '../css'

const mount = () => render(() => <HarnessesView />)

describe('harnesses/fallback-is-up', () => {
  it('marks an unmapped tier with ↑ high', () => {
    const { getByTestId } = mount()
    // claude has no xhigh mapping, so xhigh falls up to high.
    expect(getByTestId('hn-fallback-claude-xhigh').textContent).toBe('↑ high')
    expect(fallbackMarker('high')).toBe('↑ high')
  })

  it("takes the fallback from that harness's settings, not from a global rule", () => {
    for (const harness of useHarnesses()) {
      const configured = fallbackTierFor(harness.name)
      expect(configured, harness.name).toBe(TIER_FALLBACK[harness.name] ?? null)
      for (const tier of TIERS) {
        const resolved = resolveTier(harness.name, tier)
        if (!resolved.fellBackTo) continue
        expect(resolved.fellBackTo, `${harness.name}/${tier}`).toBe(configured)
      }
    }
  })

  it('lets two harnesses nominate different fallbacks', () => {
    // Their missing tiers land differently because the fallback is configuration.
    expect(fallbackTierFor('claude')).toBe('high')
    expect(fallbackTierFor('aider')).toBe('medium')
    expect(resolveTier('claude', 'xhigh').fellBackTo).toBe('high')
    expect(resolveTier('aider', 'low').fellBackTo).toBe('medium')

    // And pi, which has every tier mapped, never falls back at all.
    expect(fallbackTierFor('pi')).toBe('xhigh')
    expect(resolveTier('pi', 'low').fellBackTo).toBe(null)
  })

  it('resolves nothing where no fallback is configured — the harness keeps its own default', () => {
    expect(resolveTier('unregistered', 'high')).toEqual({ model: null, fellBackTo: null })
  })

  it('renders no down-fallback marker anywhere', () => {
    const { container } = mount()
    expect(container.textContent).not.toContain('↓')
    for (const marker of container.querySelectorAll('.hn-tier-fallback')) {
      expect(marker.textContent).toMatch(/^↑ /)
    }
  })

  it('marks the tier in the DOM with what it fell back to', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hn-tier-claude-xhigh').getAttribute('data-fallback')).toBe('high')
    expect(getByTestId('hn-tier-claude-high').getAttribute('data-fallback')).toBe(null)
  })

  it('fills a hole in the middle from the configured tier too, not only the top', () => {
    // aider has no `low`; its settings nominate `medium`, so that is what it gets.
    expect(resolveTier('aider', 'low').fellBackTo).toBe('medium')
    expect(resolveTier('aider', 'low').model).toBe('sonnet-4.6')
  })

  it('nominates a tier the harness actually has, so the fallback is not a dead end', () => {
    for (const harness of useHarnesses()) {
      const configured = fallbackTierFor(harness.name)
      if (!configured) continue
      expect(
        resolveTier(harness.name, configured).model,
        `${harness.name} falls back to an unmapped ${configured}`,
      ).not.toBe(null)
    }
  })

  it('says why in the source: the landing place depends on what the harness has', () => {
    expect(read('data/settings.ts')).toContain('The fallback is configuration, not a rule in the resolver')
  })
})
