import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { SideBySideDiff } from '../../src/screens/develop/SideBySideDiff'
import { useHunks } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <SideBySideDiff hunks={useHunks()} onToggleHunk={() => {}} />)

describe('develop/diff-tints', () => {
  it('tints added rows at rgba(79, 160, 127, 0.16)', () => {
    expect(rule('.diff-row-added').body).toContain('background: var(--diff-add)')
    expect(read('styles/tokens.css')).toContain('--diff-add: rgba(79, 160, 127, 0.16)')
  })

  it('tints removed rows at rgba(212, 97, 79, 0.14)', () => {
    expect(rule('.diff-row-removed').body).toContain('background: var(--diff-del)')
    expect(read('styles/tokens.css')).toContain('--diff-del: rgba(212, 97, 79, 0.14)')
  })

  it('tints folded regions at rgba(238, 242, 246, 0.03)', () => {
    expect(rule('.diff-row-fold').body).toContain('background: var(--diff-fold)')
    expect(read('styles/tokens.css')).toContain('--diff-fold: rgba(238, 242, 246, 0.03)')
  })

  it('tints an added row only on the side it exists', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('diff-side-right').querySelectorAll('.diff-row-added').length,
    ).toBeGreaterThan(0)
    expect(getByTestId('diff-side-left').querySelectorAll('.diff-row-added').length).toBe(0)
  })

  it('tints a removed row only on the left', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('diff-side-left').querySelectorAll('.diff-row-removed').length,
    ).toBeGreaterThan(0)
    expect(getByTestId('diff-side-right').querySelectorAll('.diff-row-removed').length).toBe(0)
  })

  it('leaves a context row untinted on both sides', () => {
    const { getByTestId } = mount()
    for (const side of ['left', 'right']) {
      for (const row of getByTestId(`diff-side-${side}`).querySelectorAll('[data-kind="context"]')) {
        expect(row.className, side).toBe('diff-row')
      }
    }
  })
})
