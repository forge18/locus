import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { FileTree } from '../../src/screens/develop/FileTree'
import { BRANCH } from '../../src/data/develop'
import { read, rules } from '../css'

const mount = () =>
  render(() => (
    <FileTree selectedPath="crates/locus-core/src/store/notify.rs" onSelect={() => {}} />
  ))

describe('develop/tree-header', () => {
  it('leads with the git-branch glyph', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree-head').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-git-branch',
    )
  })

  it('names the branch, in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree-branch').textContent).toBe(BRANCH)
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.dev-tree-head')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('sets the header in accent — the branch is the agent’s and that is worth saying', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.dev-tree-head')!.body,
    ).toContain('color: var(--ac)')
  })

  it('carries the caret on the right', () => {
    const { getByTestId } = mount()
    const icons = [...getByTestId('dev-tree-head').querySelectorAll('use')].map((u) =>
      u.getAttribute('href'),
    )
    expect(icons[icons.length - 1]).toBe('#ph-caret-down')
  })

  it('never names main — an agent never works there', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree-branch').textContent).toMatch(/^agent\//)
  })
})
