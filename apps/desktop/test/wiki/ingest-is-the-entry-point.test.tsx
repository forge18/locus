import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiView } from '../../src/screens/wiki/WikiView'
import { createNavStore } from '../../src/nav'
import { read } from '../css'

const mount = () => render(() => <WikiView nav={createNavStore({ view: 'wiki' })} />)

describe('wiki/ingest-is-the-entry-point', () => {
  it('offers no "New page" action anywhere on the screen', () => {
    const { container } = mount()
    const labels = [...container.querySelectorAll('button')].map((b) =>
      b.textContent?.toLowerCase() ?? '',
    )
    for (const label of labels) {
      expect(label).not.toContain('new page')
      expect(label).not.toContain('blank')
    }
  })

  it('has exactly one block primary, and it is ingest', () => {
    const { container, getByTestId } = mount()
    const primaries = [...container.querySelectorAll('.btn-primary.btn-block')]
    expect(primaries.length).toBe(1)
    expect(primaries[0]).toBe(getByTestId('wiki-ingest'))
  })

  it('leaves every other action secondary or plainer', () => {
    const { container, getByTestId } = mount()
    for (const button of container.querySelectorAll('button')) {
      if (button === getByTestId('wiki-ingest')) continue
      expect(button.className, button.textContent ?? '').not.toContain('btn-block')
    }
  })

  it('names no authoring path in the source — a GUI editor exists, but not here', () => {
    const source = [
      read('screens/wiki/WikiTree.tsx'),
      read('screens/wiki/WikiView.tsx'),
    ].join('\n')
    expect(source).not.toMatch(/newPage|createPage|blank/i)
  })
})
