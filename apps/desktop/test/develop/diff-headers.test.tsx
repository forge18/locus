import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { SideBySideDiff } from '../../src/screens/develop/SideBySideDiff'
import { BRANCH, DIFF_LEFT_HEADER, DIFF_RIGHT_HEADER, useHunks } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <SideBySideDiff hunks={useHunks()} onToggleHunk={() => {}} />)

describe('develop/diff-headers', () => {
  it('names the left side HEAD · main', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff-header-left').textContent).toBe('HEAD · main')
    expect(DIFF_LEFT_HEADER).toBe('HEAD · main')
  })

  it('names the right side by the agent branch and who pushed it', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff-header-right').textContent).toBe(`${BRANCH} · builder@4`)
    expect(DIFF_RIGHT_HEADER).toContain('builder@4')
  })

  it('dims the base and accents the agent side', () => {
    expect(rule('.diff-header-left').body).toContain('color: var(--text-muted)')
    expect(rule('.diff-header-right').body).toContain('color: var(--action-attention)')
  })

  it('sets both in mono', () => {
    expect(rule('.diff-header').body).toContain('font-family: var(--fm)')
  })

  it('tells them apart without reading — two different colours, two different sides', () => {
    const { getByTestId } = mount()
    expect(getByTestId('diff-header-left').className).not.toBe(
      getByTestId('diff-header-right').className,
    )
  })
})
