import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiView } from '../../src/screens/wiki/WikiView'
import { createNavStore } from '../../src/nav'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WikiView nav={createNavStore({ view: 'wiki' })} />)

describe('wiki/layout', () => {
  it('is three panes', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-tree')).toBeTruthy()
    expect(getByTestId('wiki-article')).toBeTruthy()
    expect(getByTestId('wiki-side')).toBeTruthy()
  })

  it('holds the tree near 246px and the sidebar near 284px, both flexing', () => {
    expect(rule('.wiki-tree').body).toContain('width: clamp(200px, 19%, 300px)')
    expect(rule('.wiki-tree').body).toContain('flex: none')
    expect(rule('.wiki-side').body).toContain('width: clamp(230px, 22%, 340px)')
    expect(rule('.wiki-side').body).toContain('flex: none')
  })

  it('lets the article take the rest', () => {
    expect(rule('.wiki-article').body).toContain('flex: 1')
    expect(rule('.wiki-article').body).toContain('min-width: 0')
  })

  it('hairlines both seams', () => {
    expect(rule('.wiki-tree').body).toContain('border-right: 1px solid var(--border-subtle)')
    expect(rule('.wiki-side').body).toContain('border-left: 1px solid var(--border-subtle)')
  })

  it('has no tabs to draw — Wiki is a category with one view', () => {
    const { container } = mount()
    expect(container.querySelectorAll('.tab').length).toBe(0)
  })
})
