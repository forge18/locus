import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiArticle } from '../../src/screens/wiki/WikiArticle'
import { useWikiPage } from '../../src/data/wiki'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const page = useWikiPage('w-clone')!
const mount = () => render(() => <WikiArticle page={page} onFollow={() => {}} />)

describe('wiki/prose', () => {
  it('sets prose at 16px on a 1.68 line', () => {
    const body = rule('.wiki-prose').body
    expect(body).toContain('font-size: var(--t-lead)')
    expect(body).toContain('line-height: 1.68')
  })

  it('holds it at 88% opacity and 720px', () => {
    const body = rule('.wiki-prose').body
    expect(body).toMatch(/opacity:\s*\.88/)
    expect(body).toContain('max-width: 720px')
  })

  it('renders one paragraph per body entry', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-prose').querySelectorAll('p').length).toBe(page.body.length)
  })

  it('sets an inline path in mono', () => {
    const { getByTestId } = mount()
    const codes = [...getByTestId('wiki-prose').querySelectorAll('code')].map((c) => c.textContent)
    expect(codes).toContain('/var/lib/locus/repos/<project>.git')
    expect(codes).toContain('git clone --reference')
  })

  it('sets an inline wikilink in mono too', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-prose').textContent).toContain('[[locus]]')
  })

  it('escapes rather than interpreting angle brackets in a path', () => {
    const { getByTestId } = mount()
    // `<project>` survives as text and does not become an element.
    expect(getByTestId('wiki-prose').querySelector('project')).toBe(null)
    expect(getByTestId('wiki-prose').textContent).toContain('<project>')
  })
})
