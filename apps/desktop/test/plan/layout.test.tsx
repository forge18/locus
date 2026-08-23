import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <PlanView />)

describe('plan/layout', () => {
  it('is three panes', () => {
    const { getByTestId } = mount()
    expect(getByTestId('plan-list')).toBeTruthy()
    expect(getByTestId('plan-convo')).toBeTruthy()
    expect(getByTestId('plan-outputs')).toBeTruthy()
  })

  it('holds the list near 216px and the outputs near 296px, both flexing', () => {
    expect(rule('.plan-list').body).toContain('width: clamp(180px, 17%, 260px)')
    expect(rule('.plan-list').body).toContain('flex: none')
    expect(rule('.plan-outputs').body).toContain('width: clamp(240px, 23%, 340px)')
    expect(rule('.plan-outputs').body).toContain('flex: none')
  })

  it('lets the conversation take the rest', () => {
    expect(rule('.plan-convo').body).toContain('flex: 1')
    expect(rule('.plan-convo').body).toContain('min-width: 0')
  })

  it('hairlines both seams', () => {
    expect(rule('.plan-list').body).toContain('border-right: 1px solid var(--border-subtle)')
    expect(rule('.plan-outputs').body).toContain('border-left: 1px solid var(--border-subtle)')
  })

  it('scrolls each pane on its own', () => {
    expect(rule('.plan-list-body').body).toContain('overflow: auto')
    expect(rule('.plan-messages').body).toContain('overflow: auto')
    expect(rule('.plan-outputs').body).toContain('overflow: auto')
  })
})
