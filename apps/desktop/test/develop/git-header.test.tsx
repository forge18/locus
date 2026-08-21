import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GitPanel } from '../../src/screens/develop/GitPanel'
import { useGitPanel } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const git = useGitPanel()
const mount = () =>
  render(() => (
    <GitPanel
      git={git}
      currentFile="crates/locus-core/src/store/notify.rs"
      onToggleFile={() => {}}
      onStageAll={() => {}}
      onUnstageAll={() => {}}
    />
  ))

describe('develop/git-header', () => {
  it('sits on the deep ground behind a left hairline', () => {
    const body = rule('.git-panel').body
    expect(body).toContain('background: var(--bg-deep)')
    expect(body).toContain('border-left: 1px solid var(--line)')
  })

  it('is headed GIT', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-head').textContent).toContain('Git')
    expect(rule('.git-head').body).toContain('text-transform: uppercase')
  })

  it('shows ahead in accent and behind in --mu2, both mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('git-ahead').textContent).toBe('2↑')
    expect(getByTestId('git-behind').textContent).toBe('0↓')
    expect(rule('.git-ahead').body).toContain('color: var(--ac)')
    expect(rule('.git-behind').body).toContain('color: var(--mu2)')
    expect(rule('.git-ahead').body).toContain('font-family: var(--fm)')
  })

  it('takes both counts from the data', () => {
    const { getByTestId } = render(() => (
      <GitPanel
        git={{ ...git, ahead: 7, behind: 3 }}
        currentFile=""
        onToggleFile={() => {}}
        onStageAll={() => {}}
        onUnstageAll={() => {}}
      />
    ))
    expect(getByTestId('git-ahead').textContent).toBe('7↑')
    expect(getByTestId('git-behind').textContent).toBe('3↓')
  })

  it('offers a refresh on the right', () => {
    const { getByLabelText } = mount()
    expect(getByLabelText('Refresh').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-arrows-clockwise',
    )
  })
})
