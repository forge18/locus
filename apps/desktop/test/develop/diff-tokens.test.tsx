import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { SideBySideDiff, tokenize } from '../../src/screens/develop/SideBySideDiff'
import { useHunks } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <SideBySideDiff hunks={useHunks()} onToggleHunk={() => {}} />)

describe('develop/diff-tokens', () => {
  it('colours keywords #8fb8d6, through the token', () => {
    expect(rule('.tok-keyword').body).toContain('color: var(--code-keyword)')
    expect(read('styles/tokens.css')).toContain('--code-keyword: #8fb8d6')
  })

  it('colours comments --mu', () => {
    expect(rule('.tok-comment').body).toContain('color: var(--text-secondary)')
  })

  it('marks the keywords in a real diff line', () => {
    const { container } = render(() => <div>{tokenize('    pub async fn notify() {')}</div>)
    expect([...container.querySelectorAll('.tok-keyword')].map((k) => k.textContent)).toEqual([
      'pub',
      'async',
      'fn',
    ])
  })

  it('marks a comment from // to the end of the line', () => {
    const { container } = render(() => (
      <div>{tokenize('        // NOTIFY carries an id only')}</div>
    ))
    expect(container.querySelector('.tok-comment')!.textContent).toBe(
      '// NOTIFY carries an id only',
    )
  })

  it('leaves an identifier alone — notify is not a keyword', () => {
    const { container } = render(() => <div>{tokenize('let notify = 1;')}</div>)
    const keywords = [...container.querySelectorAll('.tok-keyword')].map((k) => k.textContent)
    expect(keywords).toContain('let')
    expect(keywords).not.toContain('notify')
  })

  it('colours the diff body it actually renders', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff').querySelectorAll('.tok-keyword').length).toBeGreaterThan(0)
    expect(getByTestId('diff').querySelectorAll('.tok-comment').length).toBeGreaterThan(0)
  })
})
