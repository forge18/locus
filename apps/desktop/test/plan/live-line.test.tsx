import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { usePlanLiveLine } from '../../src/data/plan'
import { read, rules } from '../css'

const mount = () => render(() => <PlanView />)

describe('plan/live-line', () => {
  it('says what is happening right now', () => {
    const { getByTestId } = mount()
    expect(getByTestId('plan-live').textContent).toContain(
      'interviewer is re-opening question 14 of 14',
    )
    expect(usePlanLiveLine()).toContain('re-opening')
  })

  it('pulses a working-state dot beside it', () => {
    const { getByTestId } = mount()
    const dot = getByTestId('plan-live-dot')
    expect(dot.className).toContain('pulse')
    expect(
      rules(read('shell/shell.css')).find((r) => r.selector === '.live-dot')!.body,
    ).toContain('background: var(--ac2)')
  })

  it('reuses the one pulse keyframe rather than adding another', () => {
    expect(read('styles/motion.css').match(/@keyframes/g)!.length).toBe(2)
  })

  it('sits at the foot of the conversation, after the last message', () => {
    const { getByTestId } = mount()
    const children = [...getByTestId('plan-messages').children]
    expect(children[children.length - 1]).toBe(getByTestId('plan-live'))
  })
})
