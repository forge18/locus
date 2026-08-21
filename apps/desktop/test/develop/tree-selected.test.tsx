import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { FileTree } from '../../src/screens/develop/FileTree'
import { read, rules } from '../css'

const SELECTED = 'crates/locus-core/src/store/notify.rs'
const mount = () => render(() => <FileTree selectedPath={SELECTED} onSelect={() => {}} />)

describe('develop/tree-selected', () => {
  it('marks exactly one row selected', () => {
    const { getByTestId } = mount()
    const marked = getByTestId('dev-tree').querySelectorAll('[aria-selected="true"]')
    expect(marked.length).toBe(1)
    expect(marked[0].getAttribute('data-testid')).toBe(`dev-tree-row-${SELECTED}`)
  })

  it('paints it --sf2 at radius 5', () => {
    const css = rules(read('screens/screens.css'))
    expect(css.find((r) => r.selector === ".dev-tree-row[aria-selected='true']")!.body).toContain(
      'background: var(--sf2)',
    )
    expect(css.find((r) => r.selector === '.dev-tree-row')!.body).toContain(
      'border-radius: var(--r-sm)',
    )
    expect(read('styles/tokens.css')).toContain('--r-sm: 5px')
  })

  it('carries the M badge in accent', () => {
    const { getByTestId } = mount()
    const badge = getByTestId(`dev-tree-status-${SELECTED}`)
    expect(badge.textContent).toBe('M')
    expect(badge.className).toContain('dev-status-M')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.dev-status-M')!.body,
    ).toContain('color: var(--ac)')
  })

  it('gives A the --ok colour and ? the muted one', () => {
    const css = rules(read('screens/screens.css'))
    expect(css.find((r) => r.selector === '.dev-status-A')!.body).toContain('color: var(--ok)')
    expect(css.find((r) => r.selector === '.dev-status-unknown')!.body).toContain('color: var(--mu2)')
  })
})
