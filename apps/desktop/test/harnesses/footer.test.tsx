import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView, tuiNote } from '../../src/screens/workshop/HarnessesView'
import { useHarnessSummary } from '../../src/data/harnesses'

const mount = () => render(() => <HarnessesView />)
const summary = useHarnessSummary()

describe('harnesses/footer', () => {
  it('states the downgrade line', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-downgrade-line').textContent).toContain(
      `${summary.downgrades} of ${summary.entries} entries are downgrades`,
    )
  })

  it('calls it the honest measure of how uneven the field is', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-downgrade-line').textContent).toContain(
      'the honest measure of how uneven the field is',
    )
  })

  it('states the tui rule, in mono', () => {
    const { getByTestId } = mount()
    const note = getByTestId('harnesses-tui-note')
    expect(note.textContent).toBe(tuiNote(summary.harnesses))
    expect(note.className).toContain('mono')
  })

  it('says a harness claiming true is refused at registration', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-tui-note').textContent).toContain(
      'a harness claiming true is refused at registration',
    )
  })

  it('builds the tui count from the registry', () => {
    expect(tuiNote(13)).toContain('all 13')
    expect(tuiNote(summary.harnesses)).toContain(`all ${summary.harnesses}`)
  })
})
