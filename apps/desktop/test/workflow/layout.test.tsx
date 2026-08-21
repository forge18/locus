import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <WorkflowView />)

describe('workflow/layout', () => {
  it('is three panes', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-palette')).toBeTruthy()
    expect(getByTestId('wf-canvas')).toBeTruthy()
    expect(getByTestId('wf-inspector')).toBeTruthy()
  })

  it('holds the palette near 180px and lets the canvas take the room', () => {
    expect(rule('.wf-palette').body).toContain('width: clamp(150px, 14%, 220px)')
    expect(rule('.wf-canvas').body).toContain('flex: 1 1 auto')
    expect(rule('.wf-canvas').body).toContain('min-width: 0')
  })

  it('holds the inspector near 340px rather than letting it swallow the canvas', () => {
    expect(rule('.wf-inspector').body).toContain('width: clamp(260px, 26%, 380px)')
    expect(rule('.wf-inspector').body).toContain('flex: none')
  })

  it('grounds the palette on --bg-deep', () => {
    expect(rule('.wf-palette').body).toContain('background: var(--bg-deep)')
  })

  it('hairlines both seams', () => {
    expect(rule('.wf-palette').body).toContain('border-right: 1px solid var(--line)')
    expect(rule('.wf-inspector').body).toContain('border-left: 1px solid var(--line)')
  })
})
