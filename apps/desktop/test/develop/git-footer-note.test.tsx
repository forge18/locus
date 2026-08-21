import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GitPanel } from '../../src/screens/develop/GitPanel'
import { COMMIT_PLACEHOLDER, OWNERSHIP_NOTE, useGitPanel } from '../../src/data/develop'

const git = useGitPanel()
const mount = () =>
  render(() => (
    <GitPanel git={git} currentFile="" onToggleFile={() => {}} onStageAll={() => {}} onUnstageAll={() => {}} />
  ))

describe('develop/git-footer-note', () => {
  it('carries the ownership note verbatim', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-foot-note').textContent).toBe(
      'Working tree is your own checkout — the agent pushed to the branch, you decide what lands.',
    )
  })

  it('states it from one constant, so it cannot drift from the model', () => {
    expect(OWNERSHIP_NOTE).toContain('your own checkout')
    expect(OWNERSHIP_NOTE).toContain('you decide what lands')
  })

  it('offers a commit message field', () => {
    const { getByTestId } = mount()
    const field = getByTestId('git-commit-message') as HTMLInputElement
    expect(field.className).toContain('input')
    expect(field.placeholder).toBe(COMMIT_PLACEHOLDER)
  })

  it('offers Commit as the primary and Push as the secondary', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-commit').className).toContain('btn-primary')
    expect(getByTestId('git-commit').textContent).toContain('Commit')
    expect(getByTestId('git-push').className).toContain('btn-secondary')
    expect(getByTestId('git-push').textContent).toContain('Push')
  })

  it('puts the note last, under both actions', () => {
    const { getByTestId } = mount()
    const foot = getByTestId('git-foot')
    expect(foot.children[foot.children.length - 1]).toBe(getByTestId('git-foot-note'))
  })
})
