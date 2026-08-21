import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GitPanel } from '../../src/screens/develop/GitPanel'
import { useGitPanel } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const git = useGitPanel()
const mount = () =>
  render(() => (
    <GitPanel git={git} currentFile="" onToggleFile={() => {}} onStageAll={() => {}} onUnstageAll={() => {}} />
  ))

describe('develop/git-history', () => {
  it('is headed HISTORY · this branch', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-section-history').textContent).toContain('History')
    expect(getByTestId('git-section-history').textContent).toContain('this branch')
  })

  it('lists the commits newest first', () => {
    const { getByTestId } = mount()
    const subjects = [...getByTestId('git-panel').querySelectorAll('.git-commit-subject')].map(
      (s) => s.textContent,
    )
    expect(subjects).toEqual(git.history.map((c) => c.subject))
    expect(subjects[0]).toBe('cap NOTIFY payload at the row id')
  })

  it('draws 7px dots', () => {
    const body = rule('.git-dot').body
    expect(body).toContain('width: 7px')
    expect(body).toContain('height: 7px')
    expect(body).toContain('border-radius: 50%')
  })

  it('haloes the newest dot, and only the newest', () => {
    const { getByTestId } = mount()
    const dots = [...getByTestId('git-panel').querySelectorAll('.git-dot')]
    expect(dots[0].getAttribute('data-newest')).toBe('true')
    expect(dots.filter((d) => d.getAttribute('data-newest')).length).toBe(1)
    expect(rule('.git-dot-newest').body).toContain('box-shadow: 0 0 0 3px var(--ac-wash)')
    expect(rule('.git-dot-newest').body).toContain('background: var(--ac)')
  })

  it('dims the older dots', () => {
    expect(rule('.git-dot').body).toContain('background: var(--mu2)')
  })

  it('sets sha · author · age in mono', () => {
    const { getByTestId } = mount()
    const meta = getByTestId('git-commit-8f21a4c').querySelector('.git-commit-meta')!
    expect(meta.textContent).toBe('8f21a4c · builder@4 · 6m')
    expect(rule('.git-commit-meta').body).toContain('font-family: var(--fm)')
  })
})
