import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Transcript } from '../../src/screens/automate/Transcript'
import { useSessionDetail } from '../../src/data/sessions'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const session = useSessionDetail('sd-weaver')!
const mount = () => render(() => <Transcript session={session} />)

describe('agents/transcript', () => {
  it('is mono at 14px on a 1.68 line', () => {
    const body = rule('.transcript-body').body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('font-size: var(--t-body)')
    expect(body).toContain('line-height: 1.68')
  })

  it('renders one line per event', () => {
    const { getByTestId } = mount()
    expect(getByTestId('transcript').querySelectorAll('.transcript-line').length).toBe(
      session.transcript.length,
    )
  })

  it('colours tool calls in accent', () => {
    expect(rule('.verb-tool_call').body).toContain('color: var(--action-attention)')
  })

  it('colours thinking #8fb8d6, through the token', () => {
    expect(rule('.verb-thinking').body).toContain('color: var(--code-keyword)')
    expect(read('styles/tokens.css')).toContain('--code-keyword: #8fb8d6')
  })

  it('colours a pass --ok and an error --bad', () => {
    expect(rule('.verb-tool_result').body).toContain('color: var(--status-success)')
    expect(rule('.verb-tool_error').body).toContain('color: var(--status-danger)')
  })

  it('tags every line with the verb that coloured it', () => {
    const { getByTestId } = mount()
    for (const line of getByTestId('transcript').querySelectorAll('.transcript-line')) {
      const verb = line.getAttribute('data-verb')!
      expect(line.className, verb).toContain(`verb-${verb}`)
    }
  })

  it('wraps a long line rather than clipping it', () => {
    expect(rule('.transcript-line').body).toContain('white-space: pre-wrap')
  })
})
