import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { useRecentlyEdited } from '../../src/data/extensions'
import { read, rules } from '../css'

const mount = () => render(() => <ExtensionsView onNavigate={() => {}} />)

describe('extensions/recently-edited', () => {
  it('is headed RECENTLY EDITED', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recently-edited').textContent).toContain('Recently edited')
  })

  it('lists one row per edit', () => {
    const { getByTestId } = mount()
    expect(getByTestId('recently-edited').querySelectorAll('.edited-row').length).toBe(
      useRecentlyEdited().length,
    )
  })

  it('chips the type with the neutral variant, which carries the min-width', () => {
    const { getByTestId } = mount()
    const chip = getByTestId('edited-builder.md').querySelector('.tag')!
    expect(chip.className).toContain('tag-neutral')
    expect(chip.textContent).toBe('agents')
    expect(
      rules(read('ui/ui.css')).find((r) => r.selector === '.tag-neutral')!.body,
    ).toMatch(/min-width:\s*\d+px/)
  })

  it('shows the file in mono, then what changed', () => {
    const { getByTestId } = mount()
    const row = getByTestId('edited-builder.md')
    expect(row.querySelector('.edited-file')!.textContent).toBe('builder.md')
    expect(row.querySelector('.edited-summary')!.textContent).toContain('read-only')
  })

  it('right-aligns the age', () => {
    const { getByTestId } = mount()
    expect(getByTestId('edited-builder.md').querySelector('.edited-age')!.textContent).toBe('2h')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.edited-age')!.body,
    ).toContain('margin-left: auto')
  })
})
