import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { FileTree } from '../../src/screens/develop/FileTree'
import { LINKED_REPO_NOTE } from '../../src/data/develop'
import { read, rules } from '../css'

const mount = () =>
  render(() => (
    <FileTree selectedPath="crates/locus-core/src/store/notify.rs" onSelect={() => {}} />
  ))

describe('develop/tree-footer', () => {
  it('names the linked repo and the checkout path', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree-foot').textContent).toBe(
      'Linked repo · your own checkout at ~/Repos/tapestry',
    )
  })

  it('states it from one constant', () => {
    expect(LINKED_REPO_NOTE).toContain('your own checkout at')
  })

  it('sits under a top hairline at the foot of the column', () => {
    const { getByTestId } = mount()
    const tree = getByTestId('dev-tree')
    const foot = getByTestId('dev-tree-foot')
    expect([...tree.children].indexOf(foot)).toBe(2)
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.dev-tree-foot')!.body,
    ).toContain('border-top: 1px solid var(--line)')
  })

  it('says "your own", which is the distinction that matters', () => {
    expect(LINKED_REPO_NOTE).toContain('your own')
  })
})
