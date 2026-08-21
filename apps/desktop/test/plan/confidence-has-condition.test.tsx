import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Recommendation } from '../../src/screens/plan/Recommendation'
import { usePlanRecommendation } from '../../src/data/plan'
import { read } from '../css'

const recommendation = usePlanRecommendation()
const mount = () =>
  render(() => <Recommendation recommendation={recommendation} onApprove={() => {}} />)

describe('plan/confidence-has-condition', () => {
  it('shows the condition that would raise the figure', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recommendation-condition').textContent).toBe(
      'high once the provenance tie-break and the decay interaction are settled',
    )
  })

  it('names something specific, not "more information"', () => {
    expect(recommendation.condition).toMatch(/once/)
    expect(recommendation.condition).not.toMatch(/more information|further work/i)
  })

  it('shows the figure and the condition together, not one or the other', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recommendation-confidence').textContent).toBe('0.62')
    expect(getByTestId('recommendation-condition').textContent!.length).toBeGreaterThan(20)
  })

  it('types the condition as required, so a bare percentage cannot ship', () => {
    // @ts-expect-error — condition is not optional
    const bare: typeof recommendation = { confidence: 0.62, open: 2, ratchet: 'x', taskCount: 4 }
    void bare
  })

  it('keeps the condition in the fixture, so it is data and not decoration', () => {
    expect(read('fixtures/plan.ts')).toContain('condition:')
  })
})
