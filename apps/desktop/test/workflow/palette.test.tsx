import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { usePalette } from '../../src/data/workflow'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WorkflowView />)

describe('workflow/palette', () => {
  it('offers seven node kinds', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-palette').querySelectorAll('.wf-chip').length).toBe(7)
    expect(usePalette().length).toBe(7)
  })

  it('names them Goal, Agent, Task, Loop, Condition, Gate, Verify', () => {
    expect(usePalette().map((n) => n.label)).toEqual([
      'Goal',
      'Agent',
      'Task',
      'Loop',
      'Condition',
      'Gate',
      'Verify',
    ])
  })

  it('marks Verify as required, and only Verify', () => {
    const { getByTestId, queryByTestId } = mount()
    expect(getByTestId('wf-chip-req-verify').textContent).toBe('req')
    expect(queryByTestId('wf-chip-req-agent')).toBe(null)
    expect(usePalette().filter((n) => n.required).length).toBe(1)
  })

  it('makes every chip draggable with a grab cursor and a grip', () => {
    const { getByTestId } = mount()
    for (const node of usePalette()) {
      const chip = getByTestId(`wf-chip-${node.kind}`)
      expect(chip.getAttribute('draggable'), node.kind).toBe('true')
      expect(chip.querySelector('use')!.getAttribute('href'), node.kind).toBe(
        '#ph-dots-six-vertical',
      )
    }
    expect(rule('.wf-chip').body).toContain('cursor: grab')
  })

  it('gives Goal an amber hairline and Condition the blue', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-chip-goal').getAttribute('data-tone')).toBe('goal')
    expect(getByTestId('wf-chip-condition').getAttribute('data-tone')).toBe('condition')
    expect(rule('.wf-chip-goal').body).toContain('border-color: var(--ac-ring)')
    expect(rule('.wf-chip-condition').body).toContain('color: var(--code-keyword)')
  })
})
