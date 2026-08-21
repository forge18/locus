import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { REFERENCE_GROUP_LABEL, REFERENCE_KINDS } from '../../src/data/artifacts'
import { read, rules } from '../css'

const mount = () => render(() => <ArtifactsView />)

describe('artifacts/reference-group', () => {
  it('is labeled never in the inbox', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-group-reference').textContent).toBe(
      'Reference · never in the inbox',
    )
    expect(REFERENCE_GROUP_LABEL).toContain('never in the inbox')
  })

  it('is dimmed', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-group-reference').className).toContain('artifact-group-reference')
    expect(
      rules(read('screens/screens.css')).find(
        (r) => r.selector === '.artifact-group-reference',
      )!.body,
    ).toMatch(/opacity:\s*\.55/)
  })

  it('holds the two reference kinds', () => {
    expect(REFERENCE_KINDS.map((k) => k.label)).toEqual(['finding', 'payload'])
    const { getByTestId } = mount()
    for (const kind of REFERENCE_KINDS) {
      expect(getByTestId(`artifact-entry-${kind.label}`).getAttribute('data-group')).toBe(
        'reference',
      )
    }
  })

  it('dims the entries too', () => {
    const { getByTestId } = mount()
    for (const kind of REFERENCE_KINDS) {
      expect(getByTestId(`artifact-entry-${kind.label}`).className).toContain(
        'artifact-entry-reference',
      )
    }
  })

  it('says what a reference kind is: storage with a handle', () => {
    expect(REFERENCE_KINDS.find((k) => k.label === 'payload')!.note).toContain('handle')
  })

  it('sits below the review group', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('artifact-group-review').compareDocumentPosition(
        getByTestId('artifact-group-reference'),
      ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })
})
