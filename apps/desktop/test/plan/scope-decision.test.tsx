import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ScopeDecision } from '../../src/screens/plan/ScopeDecision'
import { usePlanScopeDecision } from '../../src/data/plan'
import { read, rules } from '../css'

const decision = usePlanScopeDecision()
const mount = (onWiden = () => {}, onKeepOut = () => {}) =>
  render(() => <ScopeDecision decision={decision} onWiden={onWiden} onKeepOut={onKeepOut} />)

describe('plan/scope-decision', () => {
  it('says it resolves inline, not as a separate gate', () => {
    const { getByTestId } = mount()
    expect(getByTestId('scope-decision-title').textContent).toBe(
      'Scope decision — resolves inline, not as a separate gate',
    )
  })

  it('carries the accent ring and the arrows-split glyph', () => {
    const { getByTestId } = mount()
    expect(getByTestId('scope-decision').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-arrows-split',
    )
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.scope-decision')!.body,
    ).toContain('box-shadow: var(--ring-sel-soft)')
  })

  it('explains what is being widened', () => {
    const { getByTestId } = mount()
    expect(getByTestId('scope-decision').textContent).toContain('confidence column')
    expect(getByTestId('scope-decision').querySelector('code')!.textContent).toBe('memory.fact')
  })

  it('offers both answers, and only those two', () => {
    const { getByTestId } = mount()
    expect(getByTestId('scope-widen').textContent).toBe('Widen scope')
    expect(getByTestId('scope-keep-out').textContent).toBe('Keep out, note as open')
    expect(getByTestId('scope-decision').querySelectorAll('button').length).toBe(2)
  })

  it('reports which was chosen', () => {
    let widened = 0
    let keptOut = 0
    const { getByTestId } = mount(
      () => widened++,
      () => keptOut++,
    )
    getByTestId('scope-widen').click()
    getByTestId('scope-keep-out').click()
    expect(widened).toBe(1)
    expect(keptOut).toBe(1)
  })
})
