import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WantsAttention } from '../../src/screens/status/WantsAttention'
import { useWantsAttention } from '../../src/data/status'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)
const mount = (onAction = () => {}) =>
  render(() => <WantsAttention rows={useWantsAttention()} onAction={onAction} />)

describe('status/wants-attention', () => {
  it('shows the three rows', () => {
    const { getByTestId } = mount()
    for (const kind of ['stuck', 'idle', 'waiting']) {
      expect(getByTestId(`attention-${kind}`), kind).toBeTruthy()
    }
  })

  it('gives stuck a red inset ring and the filled warning-octagon', () => {
    const { getByTestId } = mount()
    const row = getByTestId('attention-stuck')
    expect(row.className).toContain('attention-row-stuck')
    expect(row.querySelector('use')!.getAttribute('href')).toBe('#ph-warning-octagon-fill')
    expect(rule('.attention-row-stuck')!.body).toContain('inset 0 0 0 1px var(--bad)')
  })

  it('gives idle the moon', () => {
    const { getByTestId } = mount()
    expect(getByTestId('attention-idle').querySelector('use')!.getAttribute('href')).toBe('#ph-moon')
  })

  it('gives waiting the hourglass', () => {
    const { getByTestId } = mount()
    expect(getByTestId('attention-waiting').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-hourglass-medium',
    )
  })

  it('offers Reassign on the stuck row, in accent', () => {
    const { getByTestId } = mount()
    const action = getByTestId('attention-stuck-action')
    expect(action.textContent).toBe('Reassign')
    expect(rule('.attention-action')!.body).toContain('color: var(--ac)')
  })

  it('offers no action on idle or waiting — neither needs a person yet', () => {
    const { queryByTestId } = mount()
    expect(queryByTestId('attention-idle-action')).toBe(null)
    expect(queryByTestId('attention-waiting-action')).toBe(null)
  })

  it('reports the row when the action is taken', () => {
    let taken = 0
    const { getByTestId } = mount(() => taken++)
    getByTestId('attention-stuck-action').click()
    expect(taken).toBe(1)
  })
})
