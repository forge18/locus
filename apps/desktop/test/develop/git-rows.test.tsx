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

describe('develop/git-rows', () => {
  it('draws one row per file in each section', () => {
    const { getByTestId } = mount()
    for (const file of [...git.staged, ...git.unstaged]) {
      expect(getByTestId(`git-row-${file.path}`), file.path).toBeTruthy()
    }
  })

  it('colours the status letter by what it is', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-status-crates/locus-core/src/store/notify.rs').className).toContain(
      'dev-status-M',
    )
    expect(getByTestId('git-status-crates/locus-core/src/store/notify_test.rs').className).toContain(
      'dev-status-A',
    )
    expect(getByTestId('git-status-migrations/0042_notify.sql').className).toContain(
      'dev-status-unknown',
    )
  })

  it('gives the letter a 9px column', () => {
    expect(rule('.dev-status').body).toContain('width: 9px')
  })

  it('sets the path in mono and truncates it from the left', () => {
    const { getByTestId } = mount()
    const path = getByTestId('git-path-crates/locus-core/src/store/notify.rs')
    expect(path.textContent).toBe('crates/locus-core/src/store/notify.rs')
    const body = rule('.git-path').body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('text-overflow: ellipsis')
    expect(body).toContain('direction: rtl')
  })

  it('right-aligns the counts, added in --ok and removed in --bad', () => {
    const { getByTestId } = mount()
    const row = getByTestId('git-row-crates/locus-core/src/store/notify.rs')
    expect(row.querySelector('.git-added')!.textContent).toBe('+9')
    expect(row.querySelector('.git-removed')!.textContent).toBe('−2')
    expect(rule('.git-added').body).toContain('color: var(--ok)')
    expect(rule('.git-removed').body).toContain('color: var(--bad)')
  })

  it('shows no count where there is none — an untracked file has no diff yet', () => {
    const { getByTestId } = mount()
    const row = getByTestId('git-row-migrations/0042_notify.sql')
    expect(row.querySelector('.git-added')).toBe(null)
    expect(row.querySelector('.git-removed')).toBe(null)
  })
})
