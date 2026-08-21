import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { REVIEW_KINDS } from '../../src/data/artifacts'

const mount = () => render(() => <ArtifactsView />)

describe('artifacts/review-kinds', () => {
  it('is headed REVIEW ARTIFACTS', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-group-review').textContent).toBe('Review artifacts')
  })

  it('has one entry per review kind', () => {
    const { getByTestId } = mount()
    const entries = [...getByTestId('artifact-list').querySelectorAll('[data-group="review"]')]
    expect(entries.length).toBe(REVIEW_KINDS.length)
  })

  it('names the five the design draws', () => {
    expect(REVIEW_KINDS.map((k) => k.label)).toEqual([
      'diff',
      'walkthrough',
      'image',
      'recording',
      'diagram',
    ])
  })

  it('says what each one carries', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-entry-image').textContent).toContain('OCR text derived')
    expect(getByTestId('artifact-entry-recording').textContent).toContain('9 keyframes derived')
  })

  it('gives each its own glyph', () => {
    const { getByTestId } = mount()
    const icons = REVIEW_KINDS.map((k) =>
      getByTestId(`artifact-entry-${k.label}`).querySelector('use')!.getAttribute('href'),
    )
    expect(new Set(icons).size).toBe(icons.length)
  })

  it('derives text for every blob kind, because text is cheaper than pixels', () => {
    // Both the image and the recording carry a derived representation, not a blob.
    expect(REVIEW_KINDS.find((k) => k.label === 'image')!.note).toContain('derived')
    expect(REVIEW_KINDS.find((k) => k.label === 'recording')!.note).toContain('derived')
  })
})
