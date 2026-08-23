import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GitPanel } from '../../src/screens/develop/GitPanel'
import { useGitPanel } from '../../src/data/develop'
import { read, rules } from '../css'

const git = useGitPanel()
const mount = () =>
  render(() => (
    <GitPanel git={git} currentFile="" onToggleFile={() => {}} onStageAll={() => {}} onUnstageAll={() => {}} />
  ))

describe('develop/git-branch-block', () => {
  it('shows the branch in mono behind the git-branch glyph', () => {
    const { getByTestId } = mount()
    const block = getByTestId('git-branch-block')
    expect(block.querySelector('use')!.getAttribute('href')).toBe('#ph-git-branch')
    expect(block.textContent).toContain('agent/8f21-notify')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.git-branch-line')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('says where it came from and who pushed it, and when', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-branch-from').textContent).toBe(
      'from main · pushed by builder@4 6m ago',
    )
  })

  it('takes the provenance from the data', () => {
    const { getByTestId } = render(() => (
      <GitPanel
        git={{ ...git, from: 'release', pushedBy: 'reviewer@2', pushedAgo: '2h ago' }}
        currentFile=""
        onToggleFile={() => {}}
        onStageAll={() => {}}
        onUnstageAll={() => {}}
      />
    ))
    expect(getByTestId('git-branch-from').textContent).toBe(
      'from release · pushed by reviewer@2 2h ago',
    )
  })

  it('accents the glyph — the branch is the agent’s', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('git-branch-block').querySelector('svg')!.getAttribute('style'),
    ).toContain('var(--action-attention)')
  })

  it('sits under the header, above the sections', () => {
    const { getByTestId } = mount()
    const panel = getByTestId('git-panel')
    const order = [...panel.children].map((c) => c.className)
    expect(order[0]).toContain('git-head')
    expect(order[1]).toContain('git-branch-block')
  })
})
