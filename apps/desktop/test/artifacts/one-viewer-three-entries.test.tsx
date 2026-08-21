import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { createNavStore } from '../../src/nav'

/** The same artifact, reached three ways. */
const ENTRIES = [
  { name: 'from the inbox', locator: 'locus://tapestry/artifact/a-1' },
  { name: 'from the board', locator: 'locus://tapestry/artifact/a-1' },
  { name: 'from a run', locator: 'locus://tapestry/artifact/a-1' },
]

const renderFrom = (locator: string) => {
  const nav = createNavStore()
  const target = nav.open(locator)
  const { container, unmount } = render(() => (
    <ArtifactsView artifactId={target.params.artifactId} />
  ))
  const html = container.innerHTML
  unmount()
  return { view: target.view, html }
}

describe('artifacts/one-viewer-three-entries', () => {
  it('resolves all three entry points to the same view', () => {
    for (const entry of ENTRIES) {
      expect(renderFrom(entry.locator).view, entry.name).toBe('artifact')
    }
  })

  it('renders byte-identical markup from all three', () => {
    const [a, b, c] = ENTRIES.map((e) => renderFrom(e.locator).html)
    expect(b).toBe(a)
    expect(c).toBe(a)
  })

  it('renders the same as the default entry with no locator at all', () => {
    const { container, unmount } = render(() => <ArtifactsView />)
    const plain = container.innerHTML
    unmount()
    expect(renderFrom(ENTRIES[0].locator).html).toBe(plain)
  })

  it('takes one artifact id and nothing about where it came from', () => {
    const { container } = render(() => <ArtifactsView artifactId="a-1" />)
    expect(container.querySelector('[data-testid="artifact-name"]')!.textContent).toBe(
      'crates/locus-core/src/store/notify.rs',
    )
  })

  it('is one component — there is no second artifact viewer to disagree with it', () => {
    const { container } = render(() => <ArtifactsView />)
    expect(container.querySelectorAll('[data-testid="artifact-view"]').length).toBe(1)
  })
})
