import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { OPERAND_NOTE, useExpression, useOperands } from '../../src/data/workflow'

const mount = () => render(() => <WorkflowView />)

describe('workflow/operands-are-columns', () => {
  it('draws every chip from the Condition operand list', () => {
    const { getByTestId } = mount()
    const rendered = [...getByTestId('wf-operands').querySelectorAll('.operand')].map(
      (o) => o.textContent,
    )
    expect(rendered).toEqual([...useOperands()])
  })

  it('names each one as a column path, never a function call', () => {
    for (const operand of useOperands()) {
      expect(operand, operand).toMatch(/^[a-z_]+\.[a-z_]+$/)
      expect(operand, operand).not.toContain('(')
    }
  })

  it('heads the group with "every one is a column"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-operands-title').textContent).toContain('every one is a column')
  })

  it('says anything unexpressible is a Gate', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-operand-note').textContent).toBe(OPERAND_NOTE)
    expect(OPERAND_NOTE).toBe('No code, no model, no I/O — anything this cannot express is a Gate.')
  })

  it('builds the expression only from operands on the list', () => {
    for (const clause of useExpression()) {
      expect(useOperands(), clause.operand).toContain(clause.operand)
    }
  })

  it('offers a Gate node for what the list cannot reach', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-chip-gate')).toBeTruthy()
  })
})
