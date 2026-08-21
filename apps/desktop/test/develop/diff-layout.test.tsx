import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { SideBySideDiff } from '../../src/screens/develop/SideBySideDiff'
import { useHunks } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <SideBySideDiff hunks={useHunks()} onToggleHunk={() => {}} />)

describe('develop/diff-layout', () => {
  it('is side by side, two equal columns', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff-side-left')).toBeTruthy()
    expect(getByTestId('diff-side-right')).toBeTruthy()
    expect(rule('.diff-body').body).toContain('grid-template-columns: 1fr 1fr')
  })

  it('sets the body in mono at 14px on a 1.65 line', () => {
    const body = rule('.diff-body').body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('font-size: var(--t-body)')
    expect(body).toContain('line-height: 1.65')
  })

  it('gives the gutter 34px, right-aligned, in --mu2', () => {
    const body = rule('.diff-gutter').body
    expect(body).toContain('width: 34px')
    expect(body).toContain('text-align: right')
    expect(body).toContain('color: var(--mu2)')
  })

  it('gives the sign column 12px', () => {
    expect(rule('.diff-sign').body).toContain('width: 12px')
  })

  it('numbers each side from its own file', () => {
    const { getByTestId } = mount()
    const gutters = (side: string) =>
      [...getByTestId(`diff-side-${side}`).querySelectorAll('.diff-gutter')].map(
        (g) => g.textContent,
      )
    expect(gutters('left')).toContain('20')
    expect(gutters('right')).toContain('22')
  })

  it('signs added rows + and removed rows −', () => {
    const { getByTestId } = mount()
    const right = [...getByTestId('diff-side-right').querySelectorAll('.diff-row-added .diff-sign')]
    const left = [...getByTestId('diff-side-left').querySelectorAll('.diff-row-removed .diff-sign')]
    expect(right.every((s) => s.textContent === '+')).toBe(true)
    expect(left.every((s) => s.textContent === '−')).toBe(true)
  })

  it('does not wrap code — a diff line is a line', () => {
    expect(rule('.diff-text').body).toContain('white-space: pre')
  })
})
