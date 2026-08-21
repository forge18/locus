import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { NO_MODEL_NOTE, ZOOM } from '../../src/data/workflow'
import { read, rules } from '../css'

const mount = () => render(() => <WorkflowView />)

describe('workflow/canvas-footer', () => {
  it('shows a zoom pill in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-zoom').textContent).toBe(ZOOM)
    const body = rules(read('screens/screens.css')).find((r) => r.selector === '.wf-zoom')!.body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('border-radius: 10px')
  })

  it('says there is no model in the orchestration path', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-no-model-note').textContent).toBe(NO_MODEL_NOTE)
    expect(NO_MODEL_NOTE).toBe('No model in the orchestration path — the graph decides')
  })

  it('sits bottom-left of the canvas', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.wf-canvas-foot',
    )!.body
    expect(body).toContain('left: 12px')
    expect(body).toContain('bottom: 12px')
    expect(body).toContain('position: absolute')
  })

  it('is a claim the screen keeps — nothing here calls a model', () => {
    const source = read('screens/workshop/WorkflowView.tsx')
    expect(source).not.toMatch(/invoke\(['"]\w*model|prompt|completion/i)
  })
})
