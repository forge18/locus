import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Transcript } from '../../src/screens/automate/Transcript'
import { useSessionDetail } from '../../src/data/sessions'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const session = useSessionDetail('sd-weaver')!
const mount = () => render(() => <Transcript session={session} />)

describe('agents/cursor', () => {
  it('ends the transcript on a prompt line', () => {
    const { getByTestId } = mount()
    const body = getByTestId('transcript')
    expect(body.children[body.children.length - 1]).toBe(getByTestId('transcript-prompt'))
    expect(getByTestId('transcript-prompt').textContent).toContain('weaver ❯')
  })

  it('draws a 7x14px block cursor', () => {
    const body = rule('.transcript-cursor').body
    expect(body).toContain('width: 7px')
    expect(body).toContain('height: 14px')
    expect(body).toContain('display: inline-block')
  })

  it('paints it accent', () => {
    expect(rule('.transcript-cursor').body).toContain('background: var(--action-attention)')
  })

  it('blinks it with the shared keyframe', () => {
    const { getByTestId } = mount()
    expect(getByTestId('transcript-cursor').className).toContain('blink')
    expect(read('styles/motion.css')).toContain('.blink { animation: blink 1.1s')
  })

  it('takes the prompt from the session, not the markup', () => {
    const other = useSessionDetail('sd-texere')!
    const { getByTestId } = render(() => <Transcript session={other} />)
    expect(getByTestId('transcript-prompt').textContent).toContain('texere ❯')
  })
})
