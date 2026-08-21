import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiSidebar } from '../../src/screens/wiki/WikiSidebar'
import { MEMORY_DISTINCTION } from '../../src/data/wiki'
import { read, rules } from '../css'

const mount = () => render(() => <WikiSidebar />)

describe('wiki/memory-distinction-note', () => {
  it('renders the note verbatim', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-footer').textContent).toBe(
      'The wiki is curated prose a human reads. Memory is what an agent recalls — they share pgvector and nothing else.',
    )
  })

  it('states it from one constant, so the screen and the spec cannot drift', () => {
    expect(MEMORY_DISTINCTION).toContain('share pgvector and nothing else')
  })

  it('sits at the foot of the sidebar, pushed down', () => {
    const { getByTestId } = mount()
    const side = getByTestId('wiki-side')
    expect(side.children[side.children.length - 1]).toBe(getByTestId('wiki-footer'))
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.wiki-footer')!.body,
    ).toContain('margin-top: auto')
  })

  it('names both things it is telling apart', () => {
    expect(MEMORY_DISTINCTION).toContain('wiki')
    expect(MEMORY_DISTINCTION).toContain('Memory')
    expect(MEMORY_DISTINCTION).toContain('a human reads')
    expect(MEMORY_DISTINCTION).toContain('an agent recalls')
  })
})
