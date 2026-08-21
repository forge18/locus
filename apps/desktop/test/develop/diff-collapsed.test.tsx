import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { SideBySideDiff } from '../../src/screens/develop/SideBySideDiff'
import { useHunks } from '../../src/data/develop'

const mount = () => render(() => <SideBySideDiff hunks={useHunks()} onToggleHunk={() => {}} />)
const folds = useHunks().flatMap((h) => h.rows.filter((r) => r.kind === 'fold'))

describe('develop/diff-collapsed', () => {
  it('reads "⋯ N unchanged lines"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff-fold-left-18').textContent).toBe('⋯ 18 unchanged lines')
  })

  it('draws one strip per collapsed region, on both sides', () => {
    const { getByTestId } = mount()
    for (const side of ['left', 'right']) {
      expect(
        getByTestId(`diff-side-${side}`).querySelectorAll('.diff-row-fold').length,
        side,
      ).toBe(folds.length)
    }
  })

  it('takes the count from the fold, not from the label', () => {
    const { getByTestId } = mount()
    for (const fold of folds) {
      expect(getByTestId(`diff-fold-left-${fold.foldCount}`).textContent).toContain(
        `${fold.foldCount} unchanged`,
      )
    }
  })

  it('carries no line number — a fold is not a line', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff-fold-left-18').querySelector('.diff-gutter')).toBe(null)
  })

  it('has folds to draw at all, so the path is exercised', () => {
    expect(folds.length).toBeGreaterThan(2)
  })
})
