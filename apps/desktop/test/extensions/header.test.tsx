import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { HEADER_NOTE, HEADER_TITLE, NEW_LABEL } from '../../src/data/extensions'

const mount = () => render(() => <ExtensionsView onNavigate={() => {}} />)

describe('extensions/header', () => {
  it('is titled "The one surface"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('extensions-title').textContent).toBe(HEADER_TITLE)
    expect(HEADER_TITLE).toBe('The one surface')
  })

  it('says eight types, authored once, materialized fresh into every runtime', () => {
    const { getByTestId } = mount()
    const note = getByTestId('extensions-note').textContent!
    expect(note).toContain('Eight extension types')
    expect(note).toContain('authored once here')
    expect(note).toContain('materialized fresh into every runtime at run start')
    expect(HEADER_NOTE).toBe(note)
  })

  it('offers a search field', () => {
    const { getByTestId } = mount()
    expect((getByTestId('extensions-search') as HTMLInputElement).placeholder).toBe(
      'Search extensions',
    )
  })

  it('offers a primary New', () => {
    const { getByTestId } = mount()
    expect(getByTestId('extensions-new').textContent).toContain(NEW_LABEL)
    expect(getByTestId('extensions-new').className).toContain('btn-primary')
  })

  it('puts both actions on the right', () => {
    const { getByTestId } = mount()
    const actions = getByTestId('extensions-head').querySelector('.ws-actions')!
    expect(actions.contains(getByTestId('extensions-search'))).toBe(true)
    expect(actions.contains(getByTestId('extensions-new'))).toBe(true)
  })
})
