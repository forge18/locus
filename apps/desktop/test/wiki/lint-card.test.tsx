import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiSidebar } from '../../src/screens/wiki/WikiSidebar'
import { LINT_CLEAN_LINE, useWikiLint } from '../../src/data/wiki'
import { read, rules } from '../css'

const mount = () => render(() => <WikiSidebar />)

describe('wiki/lint-card', () => {
  it('is headed LOCUS WIKI LINT', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-side').textContent).toContain('Locus wiki lint')
  })

  it('shows the four categories', () => {
    const { getByTestId } = mount()
    for (const kind of ['orphan', 'broken_link', 'unnamed_entity', 'unsourced_assertion']) {
      expect(getByTestId(`lint-${kind}`), kind).toBeTruthy()
    }
    expect(useWikiLint().length).toBe(4)
  })

  it('names what each finding actually is, not just a count', () => {
    const { getByTestId } = mount()
    expect(getByTestId('lint-orphan').textContent).toContain('credential broker')
    expect(getByTestId('lint-broken_link').textContent).toContain('[[egress tiers]]')
    expect(getByTestId('lint-unnamed_entity').textContent).toContain('never given a page')
  })

  it('gives each category its own glyph', () => {
    const { getByTestId } = mount()
    const icons = ['orphan', 'broken_link', 'unnamed_entity', 'unsourced_assertion'].map((k) =>
      getByTestId(`lint-${k}`).querySelector('use')!.getAttribute('href'),
    )
    expect(new Set(icons).size).toBe(4)
  })

  it('closes with the --ok clean line', () => {
    const { getByTestId } = mount()
    const clean = getByTestId('lint-clean')
    expect(clean.textContent).toContain(LINT_CLEAN_LINE)
    expect(clean.className).toContain('lint-clean')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.lint-clean')!.body,
    ).toContain('color: var(--status-success)')
  })
})
