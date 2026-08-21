import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiTree } from '../../src/screens/wiki/WikiTree'
import { PAGE_KINDS, useWikiKindCounts } from '../../src/data/wiki'

const mount = () =>
  render(() => <WikiTree selectedId="w-clone" onSelect={() => {}} onIngest={() => {}} />)

describe('wiki/typed-groups', () => {
  it('groups by the six kinds, in order', () => {
    const { getByTestId } = mount()
    expect(
      [...getByTestId('wiki-tree').querySelectorAll('.wiki-group')].map((g) =>
        g.getAttribute('data-testid'),
      ),
    ).toEqual([
      'wiki-group-overview',
      'wiki-group-decision',
      'wiki-group-concept',
      'wiki-group-entity',
      'wiki-group-synthesis',
      'wiki-group-source',
    ])
  })

  it('shows a count on every group', () => {
    const { getByTestId } = mount()
    const counts = useWikiKindCounts()
    for (const { kind } of PAGE_KINDS) {
      expect(getByTestId(`wiki-count-${kind}`).textContent, kind).toBe(String(counts[kind]))
    }
  })

  it('gives each kind its own glyph', () => {
    const { getByTestId } = mount()
    const icons = PAGE_KINDS.map(
      ({ kind }) =>
        getByTestId('wiki-tree')
          .querySelector(`.wiki-page[data-kind="${kind}"] use`)
          ?.getAttribute('href') ?? null,
    ).filter(Boolean)
    expect(new Set(icons).size).toBe(icons.length)
    expect(icons).toContain('#ph-gavel')
    expect(icons).toContain('#ph-lightbulb')
    expect(icons).toContain('#ph-cube')
    expect(icons).toContain('#ph-file-text')
  })

  it('carries the counts the design draws', () => {
    const counts = useWikiKindCounts()
    expect(counts).toEqual({
      overview: 1,
      decision: 14,
      concept: 31,
      entity: 42,
      synthesis: 8,
      source: 57,
    })
  })

  it('lists pages under the group they belong to', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-page-w-clone').getAttribute('data-kind')).toBe('decision')
    expect(getByTestId('wiki-page-w-locusd').getAttribute('data-kind')).toBe('entity')
  })
})
