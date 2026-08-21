import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiTree } from '../../src/screens/wiki/WikiTree'
import { useWikiPages } from '../../src/data/wiki'
import { read, rules } from '../css'

const mount = () =>
  render(() => <WikiTree selectedId="w-clone" onSelect={() => {}} onIngest={() => {}} />)
const orphans = useWikiPages().filter((p) => p.orphan)

describe('wiki/orphan-flag', () => {
  it('flags every orphan page in the tree', () => {
    const { getByTestId } = mount()
    for (const page of orphans) {
      expect(getByTestId(`wiki-orphan-${page.id}`).textContent, page.id).toBe('orphan')
    }
  })

  it('sets the flag in --bad', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.wiki-orphan')!.body,
    ).toContain('color: var(--bad)')
  })

  it('flags nothing that is linked to', () => {
    const { queryByTestId } = mount()
    for (const page of useWikiPages().filter((p) => !p.orphan)) {
      expect(queryByTestId(`wiki-orphan-${page.id}`), page.id).toBe(null)
    }
  })

  it('flags entities, which is where an orphan actually happens', () => {
    for (const page of orphans) expect(page.kind, page.id).toBe('entity')
  })

  it('has orphans to flag at all, so the path is exercised', () => {
    expect(orphans.length).toBeGreaterThan(0)
  })
})
