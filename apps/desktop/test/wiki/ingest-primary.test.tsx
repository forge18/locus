import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiTree } from '../../src/screens/wiki/WikiTree'
import { INGEST_NOTE } from '../../src/data/wiki'

const mount = (onIngest = () => {}) =>
  render(() => <WikiTree selectedId="w-clone" onSelect={() => {}} onIngest={onIngest} />)

describe('wiki/ingest-primary', () => {
  it('leads the tree with "Ingest a document"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-ingest').textContent).toContain('Ingest a document')
  })

  it('is the block primary', () => {
    const { getByTestId } = mount()
    const button = getByTestId('wiki-ingest')
    expect(button.className).toContain('btn-primary')
    expect(button.className).toContain('btn-block')
  })

  it('says a path or a URL, not a blank page', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-ingest-note').textContent).toBe(INGEST_NOTE)
    expect(INGEST_NOTE).toContain('Derived, then curated')
    expect(INGEST_NOTE).toContain('not a blank page')
  })

  it('sits above every group', () => {
    const { getByTestId } = mount()
    const tree = getByTestId('wiki-tree')
    const first = tree.querySelector('.wiki-group')!
    expect(
      getByTestId('wiki-ingest').compareDocumentPosition(first) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('reports the ingest', () => {
    let ingested = 0
    const { getByTestId } = mount(() => ingested++)
    getByTestId('wiki-ingest').click()
    expect(ingested).toBe(1)
  })
})
