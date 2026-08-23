import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Recommendation } from '../../src/screens/plan/Recommendation'
import { usePlanRecommendation } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const recommendation = usePlanRecommendation()
const mount = (onApprove = () => {}) =>
  render(() => <Recommendation recommendation={recommendation} onApprove={onApprove} />)

describe('plan/recommendation', () => {
  it('shows the confidence at 26px in accent', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recommendation-confidence').textContent).toBe('0.62')
    const body = rule('.recommendation-confidence').body
    expect(body).toContain('font-size: var(--t-metric)')
    expect(body).toContain('color: var(--action-attention)')
  })

  it('always shows two decimal places, so 0.6 and 0.60 do not read differently', () => {
    const { getByTestId } = render(() => (
      <Recommendation recommendation={{ ...recommendation, confidence: 0.6 }} onApprove={() => {}} />
    ))
    expect(getByTestId('recommendation-confidence').textContent).toBe('0.60')
  })

  it('shows the open count in mono, as open[N]', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recommendation-open').textContent).toBe('open[2]')
    expect(rule('.recommendation-open').body).toContain('font-family: var(--fm)')
  })

  it('carries the ratchet note', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recommendation-ratchet').textContent).toContain('Ratchet')
  })

  it('rings the card in accent, because it is the one thing to decide here', () => {
    expect(rule('.recommendation').body).toContain('box-shadow: var(--ring-sel-soft)')
  })

  it('reports the approval', () => {
    let approved = 0
    const { getByTestId } = mount(() => approved++)
    getByTestId('recommendation-approve').click()
    expect(approved).toBe(1)
  })
})
