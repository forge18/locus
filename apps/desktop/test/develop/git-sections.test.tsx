import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GitPanel } from '../../src/screens/develop/GitPanel'
import { useGitPanel } from '../../src/data/develop'
import { read, rules } from '../css'

const git = useGitPanel()
const mount = (onStageAll = () => {}, onUnstageAll = () => {}) =>
  render(() => (
    <GitPanel
      git={git}
      currentFile=""
      onToggleFile={() => {}}
      onStageAll={onStageAll}
      onUnstageAll={onUnstageAll}
    />
  ))

describe('develop/git-sections', () => {
  it('heads the two sections with their counts', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-section-staged').textContent).toContain('Staged')
    expect(getByTestId('git-section-staged').textContent).toContain(String(git.staged.length))
    expect(getByTestId('git-section-unstaged').textContent).toContain('Unstaged')
    expect(getByTestId('git-section-unstaged').textContent).toContain(String(git.unstaged.length))
  })

  it('offers Unstage all on staged and Stage all on unstaged', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-unstage-all').textContent).toBe('Unstage all')
    expect(getByTestId('git-stage-all').textContent).toBe('Stage all')
  })

  it('sets both bulk links in accent', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.git-bulk')!.body,
    ).toContain('color: var(--action-attention)')
  })

  it('reports each bulk action', () => {
    let staged = 0
    let unstaged = 0
    const { getByTestId } = mount(
      () => staged++,
      () => unstaged++,
    )
    getByTestId('git-stage-all').click()
    getByTestId('git-unstage-all').click()
    expect(staged).toBe(1)
    expect(unstaged).toBe(1)
  })

  it('orders staged above unstaged above history', () => {
    const { getByTestId } = mount()
    const sections = [...getByTestId('git-panel').querySelectorAll('.git-section')].map((s) =>
      s.getAttribute('data-testid'),
    )
    expect(sections).toEqual(['git-section-staged', 'git-section-unstaged', 'git-section-history'])
  })
})
