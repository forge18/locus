import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'

const STAGED = 'crates/locus-core/src/store/notify.rs'
const UNSTAGED = 'migrations/0042_notify.sql'
const mount = () => render(() => <DevelopView />)

const sectionOf = (root: HTMLElement, path: string) => {
  const row = root.querySelector(`[data-testid="git-row-${path}"]`)!
  let el: Element | null = row
  while ((el = el.previousElementSibling)) {
    if (el.classList.contains('git-section')) return el.getAttribute('data-testid')
  }
  return null
}

describe('develop/stage-file', () => {
  it('starts with two staged and two unstaged', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-section-staged').textContent).toContain('2')
    expect(getByTestId('git-section-unstaged').textContent).toContain('2')
  })

  it('unstages a staged file, moving it to the other section', () => {
    const { getByTestId } = mount()
    expect(sectionOf(getByTestId('git-panel'), STAGED)).toBe('git-section-staged')
    getByTestId(`git-row-${STAGED}`).click()
    expect(sectionOf(getByTestId('git-panel'), STAGED)).toBe('git-section-unstaged')
  })

  it('stages an unstaged file', () => {
    const { getByTestId } = mount()
    expect(sectionOf(getByTestId('git-panel'), UNSTAGED)).toBe('git-section-unstaged')
    getByTestId(`git-row-${UNSTAGED}`).click()
    expect(sectionOf(getByTestId('git-panel'), UNSTAGED)).toBe('git-section-staged')
  })

  it('moves only the file that was clicked', () => {
    const { getByTestId } = mount()
    getByTestId(`git-row-${STAGED}`).click()
    expect(sectionOf(getByTestId('git-panel'), 'crates/locus-core/src/store/mod.rs')).toBe(
      'git-section-staged',
    )
  })

  it('updates both counts together', () => {
    const { getByTestId } = mount()
    getByTestId(`git-row-${UNSTAGED}`).click()
    expect(getByTestId('git-section-staged').textContent).toContain('3')
    expect(getByTestId('git-section-unstaged').textContent).toContain('1')
  })

  it('stages everything at once when asked', () => {
    const { getByTestId } = mount()
    getByTestId('git-stage-all').click()
    expect(getByTestId('git-section-staged').textContent).toContain('4')
    expect(getByTestId('git-section-unstaged').textContent).toContain('0')
  })

  it('unstages everything at once too', () => {
    const { getByTestId } = mount()
    getByTestId('git-unstage-all').click()
    expect(getByTestId('git-section-staged').textContent).toContain('0')
    expect(getByTestId('git-section-unstaged').textContent).toContain('4')
  })
})
