import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { LOOP_GROUP, useCanvas } from '../../src/data/workflow'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WorkflowView />)

describe('workflow/loop-visuals', () => {
  it('labels every edge that carries one, as a pill on --bg', () => {
    const { getByTestId } = mount()
    const labelled = useCanvas().edges.filter((e) => e.label)
    for (const edge of labelled) {
      expect(
        getByTestId(`wf-edge-label-${edge.from}-${edge.to}`).textContent,
        `${edge.from}->${edge.to}`,
      ).toBe(edge.label)
    }
    const body = rule('.wf-edge-label').body
    expect(body).toContain('background: var(--surface-ground)')
    expect(body).toContain('border: 1px solid var(--border-strong)')
  })

  it('dashes the loop-back edge, and only it', () => {
    const { getByTestId } = mount()
    const dashed = [...getByTestId('wf-edges').querySelectorAll('[data-dashed="true"]')]
    expect(dashed.length).toBe(1)
    expect(dashed[0].getAttribute('data-testid')).toBe('wf-edge-n-cond-n-build')
    expect(dashed[0].getAttribute('stroke-dasharray')).toBe('4 3')
  })

  it('points the loop-back edge at the loop marker', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-edge-n-cond-n-build').getAttribute('marker-end')).toBe(
      'url(#arrow-loop)',
    )
  })

  it('groups the loop in a dashed rounded rect', () => {
    const { getByTestId } = mount()
    const group = getByTestId('wf-loop-group') as HTMLElement
    expect(group.style.left).toBe(`${LOOP_GROUP.x}px`)
    expect(group.style.width).toBe(`${LOOP_GROUP.width}px`)
    const body = rule('.wf-loop-group').body
    expect(body).toContain('border: 1px dashed var(--border-strong)')
    expect(body).toContain('border-radius: var(--r-lg)')
  })

  it('labels the group with its bound', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-loop-group').textContent).toBe('loop · max 3')
  })

  it('leaves the group non-interactive, so it does not eat clicks on the nodes', () => {
    expect(rule('.wf-loop-group').body).toContain('pointer-events: none')
  })
})
