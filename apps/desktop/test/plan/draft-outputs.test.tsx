import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { usePlanOutputs } from '../../src/data/plan'
import { read, rules } from '../css'

const mount = () => render(() => <PlanView />)
const outputs = usePlanOutputs()

describe('plan/draft-outputs', () => {
  it('is headed DRAFT OUTPUTS', () => {
    const { getByTestId } = mount()
    expect(getByTestId('plan-outputs').textContent).toContain('Draft outputs')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.plan-outputs-title')!.body,
    ).toContain('text-transform: uppercase')
  })

  it('shows spec.md in mono with what it contains', () => {
    const { getByTestId } = mount()
    const spec = getByTestId('output-spec')
    expect(spec.querySelector('.mono')!.textContent).toBe('spec.md')
    expect(spec.textContent).toContain('14 requirements')
    expect(spec.textContent).toContain('3 trust boundaries')
  })

  it('numbers the tasks', () => {
    const { getByTestId } = mount()
    const list = getByTestId('output-task-list')
    expect(list.tagName).toBe('OL')
    expect(list.querySelectorAll('li').length).toBe(outputs.tasks.length)
    expect(list.querySelectorAll('li')[0].textContent).toBe(outputs.tasks[0])
  })

  it('chips the tool list in mono', () => {
    const { getByTestId } = mount()
    const tools = getByTestId('output-tools')
    for (const tool of outputs.tools) expect(tools.textContent, tool).toContain(tool)
    expect(
      rules(read('ui/ui.css')).find((r) => r.selector === '.tag')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('outlines a tool the plan would add, rather than filling it like one already there', () => {
    const { getByTestId } = mount()
    const chips = [...getByTestId('output-tools').querySelectorAll('.tag')]
    const pgvector = chips.find((c) => c.textContent === '+ pgvector')!
    expect(pgvector.className).toContain('tag-outline')
    for (const existing of outputs.tools) {
      expect(chips.find((c) => c.textContent === existing)!.className, existing).not.toContain(
        'tag-outline',
      )
    }
  })

  it('shows the four cards in order: spec, tasks, tools, recommendation', () => {
    const { getByTestId } = mount()
    const order = [...getByTestId('plan-outputs').querySelectorAll('.output-card')].map((c) =>
      c.getAttribute('data-testid'),
    )
    expect(order).toEqual(['output-spec', 'output-tasks', 'output-tools', 'recommendation'])
  })
})
