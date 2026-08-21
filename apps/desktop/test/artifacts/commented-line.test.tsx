import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { useUnifiedDiff } from '../../src/data/artifacts'
import { read, rules } from '../css'

const mount = () => render(() => <ArtifactsView />)

describe('artifacts/commented-line', () => {
  it('marks the commented line with the accent left inset', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.udiff-commented')!.body,
    ).toContain('box-shadow: inset 3px 0 0 var(--ac)')
  })

  it('marks exactly the line the comment hangs off', () => {
    const { getByTestId } = mount()
    const marked = getByTestId('udiff').querySelectorAll('[data-commented="true"]')
    expect(marked.length).toBe(useUnifiedDiff().filter((r) => r.commented).length)
    expect(marked.length).toBe(1)
  })

  it('marks the right line', () => {
    const { getByTestId } = mount()
    const marked = getByTestId('udiff').querySelector('[data-commented="true"]')!
    expect(marked.textContent).toContain('sqlx::query("SELECT pg_notify($1, $2)")')
    expect(marked.querySelector('.udiff-gutter')!.textContent).toBe('21')
  })

  it("keeps the line's own tint alongside the mark", () => {
    const { getByTestId } = mount()
    const marked = getByTestId('udiff').querySelector('[data-commented="true"]')!
    expect(marked.className).toContain('udiff-added')
    expect(marked.className).toContain('udiff-commented')
  })

  it('marks nothing else', () => {
    const { getByTestId } = mount()
    for (const row of getByTestId('udiff').querySelectorAll('.udiff-row')) {
      if (row.getAttribute('data-commented') === 'true') continue
      expect(row.className).not.toContain('udiff-commented')
    }
  })
})
