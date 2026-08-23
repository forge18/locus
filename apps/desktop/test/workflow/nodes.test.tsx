import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { useCanvas } from '../../src/data/workflow'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WorkflowView />)

describe('workflow/nodes', () => {
  it('draws one card per node', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-canvas').querySelectorAll('.wf-node').length).toBe(
      useCanvas().nodes.length,
    )
  })

  it('positions them absolutely at their stored coordinates', () => {
    const { getByTestId } = mount()
    for (const node of useCanvas().nodes) {
      const el = getByTestId(`wf-node-${node.id}`) as HTMLElement
      expect(el.style.left, node.id).toBe(`${node.x}px`)
      expect(el.style.top, node.id).toBe(`${node.y}px`)
    }
    expect(rule('.wf-node').body).toContain('position: absolute')
  })

  it('draws each at radius 9 with the node shadow', () => {
    const body = rule('.wf-node').body
    expect(body).toContain('border-radius: var(--r-lg)')
    expect(body).toContain('box-shadow: var(--sh-node)')
  })

  it('gives each a tinted header strip with the kind in uppercase', () => {
    const { getByTestId } = mount()
    const strip = getByTestId('wf-node-strip-n-plan')
    expect(strip.textContent).toContain('agent')
    expect(rule('.wf-node-strip').body).toContain('height: 9px')
    expect(rule('.wf-node-kind').body).toContain('text-transform: uppercase')
  })

  it('tints the strip by tone — goal amber, condition blue', () => {
    expect(rule('.wf-node-goal .wf-node-strip').body).toContain('background: var(--action-attention-wash)')
    expect(rule('.wf-node-condition .wf-node-strip').body).toContain('rgba(143,184,214,.18)')
    const { getByTestId } = mount()
    expect(getByTestId('wf-node-n-goal').getAttribute('data-tone')).toBe('goal')
    expect(getByTestId('wf-node-n-cond').getAttribute('data-tone')).toBe('condition')
  })

  it('puts the state on the right of the strip', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-node-state-n-build').textContent).toBe('running')
    expect(getByTestId('wf-node-state-n-verify').textContent).toBe('failed 1/3')
    expect(rule('.wf-node-state').body).toContain('margin-left: auto')
  })
})
