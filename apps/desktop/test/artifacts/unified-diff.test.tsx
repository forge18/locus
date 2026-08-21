import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { useUnifiedDiff } from '../../src/data/artifacts'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <ArtifactsView />)

describe('artifacts/unified-diff', () => {
  it('is unified — one column, not two', () => {
    const { getByTestId } = mount()
    expect(getByTestId('udiff').querySelectorAll('.diff-side').length).toBe(0)
    expect(getByTestId('udiff').querySelectorAll('.udiff-row').length).toBe(
      useUnifiedDiff().length,
    )
  })

  it('dims the @@ headers', () => {
    const { getByTestId } = mount()
    expect(getByTestId('udiff-hunk--18,7').textContent).toContain('@@ -18,7 +18,9 @@')
    expect(rule('.udiff-hunk').body).toContain('color: var(--mu2)')
  })

  it('gives the gutter 26px, right-aligned', () => {
    const body = rule('.udiff-gutter').body
    expect(body).toContain('width: 26px')
    expect(body).toContain('text-align: right')
  })

  it('uses the same three tints as Develop', () => {
    expect(rule('.udiff-added').body).toContain('background: var(--diff-add)')
    expect(rule('.udiff-removed').body).toContain('background: var(--diff-del)')
    expect(rule('.udiff-hunk').body).toContain('background: var(--diff-fold)')
  })

  it('is mono at 14px', () => {
    const body = rule('.udiff').body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('font-size: var(--t-body)')
  })

  it('numbers every line but the hunk headers', () => {
    const { getByTestId } = mount()
    const rows = [...getByTestId('udiff').querySelectorAll('.udiff-row')]
    for (const row of rows) {
      const gutter = row.querySelector('.udiff-gutter')!.textContent
      if (row.getAttribute('data-kind') === 'hunk') expect(gutter).toBe('')
      else expect(gutter).not.toBe('')
    }
  })

  it('marks each row with the kind that tinted it', () => {
    const { getByTestId } = mount()
    for (const row of getByTestId('udiff').querySelectorAll('.udiff-row')) {
      const kind = row.getAttribute('data-kind')!
      if (kind === 'context') continue
      expect(row.className, kind).toContain(`udiff-${kind}`)
    }
  })
})
