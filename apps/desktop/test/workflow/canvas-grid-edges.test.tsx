import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { ARROW_MARKERS, useCanvas } from '../../src/data/workflow'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WorkflowView />)

describe('workflow/canvas-grid-edges', () => {
  it('draws a 24px dot grid', () => {
    const body = rule('.wf-canvas').body
    expect(body).toContain('background-size: 24px 24px')
    expect(body).toContain('radial-gradient(var(--line) 1px, transparent 1px)')
  })

  it('has an SVG edge layer over the grid', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-edges').tagName.toLowerCase()).toBe('svg')
    expect(rule('.wf-edges').body).toContain('position: absolute')
    expect(rule('.wf-edges').body).toContain('pointer-events: none')
  })

  it('defines four arrow markers', () => {
    const { getByTestId } = mount()
    expect(ARROW_MARKERS.length).toBe(4)
    for (const marker of ARROW_MARKERS) {
      expect(getByTestId(`wf-marker-${marker}`), marker).toBeTruthy()
    }
  })

  it('draws one line per edge', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-edges').querySelectorAll('line').length).toBe(useCanvas().edges.length)
  })

  it('points every edge at a marker', () => {
    const { getByTestId } = mount()
    for (const line of getByTestId('wf-edges').querySelectorAll('line')) {
      expect(line.getAttribute('marker-end')).toMatch(/^url\(#arrow-/)
    }
  })

  it('hairlines the edges from the shared token', () => {
    expect(rule('.graph-edge').body).toContain('stroke: var(--line2)')
  })
})
