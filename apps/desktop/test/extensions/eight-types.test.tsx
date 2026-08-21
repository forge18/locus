import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { useTypeCards } from '../../src/data/extensions'
import { useExtensionCounts, useExtensionTypes } from '../../src/data/harnesses'

const mount = () => render(() => <ExtensionsView onNavigate={() => {}} />)

describe('extensions/eight-types', () => {
  it('shows exactly eight cards', () => {
    const { getByTestId } = mount()
    expect(getByTestId('type-grid').querySelectorAll('.type-card').length).toBe(8)
  })

  it('names the eight the registry declares', () => {
    expect(useTypeCards().map((c) => c.type).sort()).toEqual([...useExtensionTypes()].sort())
  })

  it('has a per-harness count for every one of them', () => {
    expect(useExtensionCounts().length).toBe(8)
    expect(useExtensionCounts().map((c) => c.type).sort()).toEqual([...useExtensionTypes()].sort())
  })

  it('names them the same on screen as in the registry', () => {
    const { getByTestId } = mount()
    for (const type of useExtensionTypes()) {
      expect(getByTestId(`type-card-${type}`), type).toBeTruthy()
    }
  })

  it('has no ninth — the card list is the registry list', () => {
    const { getByTestId } = mount()
    const rendered = [...getByTestId('type-grid').querySelectorAll('.type-card')].map((c) =>
      c.getAttribute('data-testid')?.replace('type-card-', ''),
    )
    for (const type of rendered) expect(useExtensionTypes(), type).toContain(type as string)
  })
})
