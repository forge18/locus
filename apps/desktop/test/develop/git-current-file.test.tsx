import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { GitPanel } from '../../src/screens/develop/GitPanel'
import { useGitPanel } from '../../src/data/develop'
import { read, rules } from '../css'

const git = useGitPanel()
const CURRENT = 'crates/locus-core/src/store/notify.rs'

describe('develop/git-current-file', () => {
  it('highlights the row for the file being looked at', () => {
    const { getByTestId } = render(() => (
      <GitPanel
        git={git}
        currentFile={CURRENT}
        onToggleFile={() => {}}
        onStageAll={() => {}}
        onUnstageAll={() => {}}
      />
    ))
    expect(getByTestId(`git-row-${CURRENT}`).getAttribute('data-current')).toBe('true')
    expect(getByTestId(`git-row-${CURRENT}`).className).toContain('git-row-current')
  })

  it('paints it --sf2', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.git-row-current')!.body,
    ).toContain('background: var(--surface-selected)')
  })

  it('highlights exactly one row', () => {
    const { getByTestId } = render(() => (
      <GitPanel
        git={git}
        currentFile={CURRENT}
        onToggleFile={() => {}}
        onStageAll={() => {}}
        onUnstageAll={() => {}}
      />
    ))
    expect(getByTestId('git-panel').querySelectorAll('[data-current="true"]').length).toBe(1)
  })

  it('follows the tree selection', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId(`git-row-${CURRENT}`).getAttribute('data-current')).toBe('true')
    getByTestId('dev-tree-row-crates/locus-core/src/store/mod.rs').click()
    expect(getByTestId(`git-row-${CURRENT}`).getAttribute('data-current')).toBe(null)
    expect(
      getByTestId('git-row-crates/locus-core/src/store/mod.rs').getAttribute('data-current'),
    ).toBe('true')
  })

  it('highlights nothing when the open file has no change', () => {
    const { getByTestId } = render(() => (
      <GitPanel
        git={git}
        currentFile="crates/locus-core/src/store/pool.rs"
        onToggleFile={() => {}}
        onStageAll={() => {}}
        onUnstageAll={() => {}}
      />
    ))
    expect(getByTestId('git-panel').querySelectorAll('[data-current="true"]').length).toBe(0)
  })
})
