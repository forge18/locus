import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { FileTree } from '../../src/screens/develop/FileTree'
import { useFileTree } from '../../src/data/develop'
import { read, rules } from '../css'

const mount = (onSelect: (path: string) => void = () => {}) =>
  render(() => (
    <FileTree selectedPath="crates/locus-core/src/store/notify.rs" onSelect={onSelect} />
  ))

describe('develop/tree', () => {
  it('renders one row per node', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree').querySelectorAll('.dev-tree-row').length).toBe(
      useFileTree().length,
    )
  })

  it('indents 20px at the first level and 34px at the second', () => {
    const { getByTestId } = mount()
    const at = (path: string) =>
      (getByTestId(`dev-tree-row-${path}`) as HTMLElement).style.paddingLeft
    expect(at('crates/locus-core/src')).toBe('0px')
    expect(at('crates/locus-core/src/store')).toBe('20px')
    expect(at('crates/locus-core/src/store/notify.rs')).toBe('34px')
  })

  it('sets rows at 14px', () => {
    expect(rules(read('screens/screens.css')).find((r) => r.selector === '.dev-tree-row')!.body).toContain(
      'font-size: var(--t-body)',
    )
  })

  it('selects a file when it is clicked', () => {
    let selected: string | null = null
    const { getByTestId } = mount((p) => (selected = p))
    getByTestId('dev-tree-row-crates/locus-core/src/store/mod.rs').click()
    expect(selected).toBe('crates/locus-core/src/store/mod.rs')
  })

  it('does not select a directory — there is nothing to open', () => {
    let selected: string | null = null
    const { getByTestId } = mount((p) => (selected = p))
    getByTestId('dev-tree-row-crates/locus-core/src/store').click()
    expect(selected).toBe(null)
  })

  it('carries a status letter only where there is a change', () => {
    const { getByTestId, queryByTestId } = mount()
    expect(getByTestId('dev-tree-status-crates/locus-core/src/store/notify.rs').textContent).toBe('M')
    expect(queryByTestId('dev-tree-status-crates/locus-core/src/store/pool.rs')).toBe(null)
  })
})
