import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import {
  BUDGET_NOTE,
  SAVE_LABEL,
  VALIDATE_LABEL,
  WAITING_NOTE,
  useGuardrails,
} from '../../src/data/workflow'
import { read, rules } from '../css'

const mount = () => render(() => <WorkflowView />)

describe('workflow/guardrails', () => {
  it('shows the six guardrails', () => {
    const { getByTestId } = mount()
    for (const guardrail of useGuardrails()) {
      expect(getByTestId(`guardrail-${guardrail.key}`), guardrail.key).toBeTruthy()
    }
    expect(useGuardrails().length).toBe(6)
  })

  it('gives max_iterations a stepper at 8', () => {
    const { getByTestId } = mount()
    expect(getByTestId('guardrail-stepper-max_iterations')).toBeTruthy()
    expect(getByTestId('guardrail-value-max_iterations').textContent).toBe('8')
  })

  it('gives kill & reassign a stepper at 3, matching the guardrail rule', () => {
    const { getByTestId } = mount()
    expect(getByTestId('guardrail-stepper-kill_and_reassign')).toBeTruthy()
    expect(getByTestId('guardrail-value-kill_and_reassign').textContent).toBe('3')
  })

  it('gives reflection-before-retry a toggle, on', () => {
    const { getByTestId } = mount()
    expect(getByTestId('guardrail-toggle-reflection_before_retry').getAttribute('data-on')).toBe(
      'true',
    )
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === ".toggle[data-on='true']")!.body,
    ).toContain('inset 0 0 0 1px var(--action-attention)')
  })

  it('shows idle at 60s, and wall-clock and budget as none', () => {
    const { getByTestId } = mount()
    expect(getByTestId('guardrail-value-idle_detection').textContent).toBe('60s')
    expect(getByTestId('guardrail-value-wall_clock').textContent).toBe('none')
    expect(getByTestId('guardrail-value-token_budget').textContent).toBe('none')
  })

  it('says why an unset budget is unbounded on purpose', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-budget-note').textContent).toBe(BUDGET_NOTE)
    expect(BUDGET_NOTE).toContain('stops good runs before it stops bad ones')
  })

  it('says waiting is not idle', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-waiting-note').textContent).toBe(WAITING_NOTE)
    expect(WAITING_NOTE).toBe('Waiting ≠ idle')
  })

  it('closes with Validate graph and Save workflow', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-validate').textContent).toBe(VALIDATE_LABEL)
    expect(getByTestId('wf-save').textContent).toBe(SAVE_LABEL)
    expect(getByTestId('wf-save').className).toContain('btn-primary')
  })
})
