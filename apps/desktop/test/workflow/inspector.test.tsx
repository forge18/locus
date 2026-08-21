import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { COMPILED, COMPILED_NOTE, useExpression, useOperands } from '../../src/data/workflow'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WorkflowView />)

describe('workflow/inspector', () => {
  it('heads with the node it is inspecting', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-inspector-title').textContent).toContain('Condition')
  })

  it('builds the expression from 26px mono token fields', () => {
    const { getByTestId } = mount()
    const clause = getByTestId('clause-0')
    expect(clause.querySelectorAll('.clause-field').length).toBe(3)
    expect(clause.textContent).toContain('verify.passed')
    const body = rule('.clause-field').body
    expect(body).toContain('height: 26px')
    expect(body).toContain('font-family: var(--fm)')
  })

  it('joins clauses with an accent and', () => {
    const { getByTestId } = mount()
    expect(getByTestId('clause-joiner').textContent).toBe('and')
    expect(rule('.clause-joiner').body).toContain('color: var(--ac)')
    expect(useExpression().length).toBe(2)
  })

  it('offers a ghost add-clause', () => {
    const { getByTestId } = mount()
    expect(getByTestId('clause-add').textContent).toBe('+ add clause')
    expect(rule('.clause-add').body).toContain('color: var(--mu2)')
  })

  it('shows the compiled expression in a card with an --ok hairline', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-compiled-expr').textContent).toBe(COMPILED)
    expect(rule('.compiled').body).toContain('inset 0 0 0 1px var(--ok)')
  })

  it('says the expression is total, evaluable in the core, and reproducible', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-compiled-note').textContent).toBe(COMPILED_NOTE)
    expect(COMPILED_NOTE).toContain('total')
    expect(COMPILED_NOTE).toContain('evaluable in the core')
    expect(COMPILED_NOTE).toContain('reproducible from stored events')
  })

  it('shows the operand chips', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-operands').querySelectorAll('.operand').length).toBe(
      useOperands().length,
    )
  })
})
