import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'
import { PRESET_NOTE, usePresets } from '../../src/data/workflow'
import { read, rules } from '../css'

const mount = () => render(() => <WorkflowView />)

describe('workflow/presets', () => {
  it('is headed PRESETS', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-presets-title').textContent).toBe('Presets')
  })

  it('offers the Ralph loop and the Review pass', () => {
    expect(usePresets().map((p) => p.name)).toEqual(['Ralph loop', 'Review pass'])
    const { getByTestId } = mount()
    expect(getByTestId('wf-preset-Ralph-loop').textContent).toContain('pick · act · validate')
  })

  it('grounds them on --sf2, a step above the node chips', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.wf-preset')!.body,
    ).toContain('background: var(--sf2)')
  })

  it('says a preset expands into ordinary nodes', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wf-preset-note').textContent).toBe(PRESET_NOTE)
    expect(PRESET_NOTE).toContain('expands into ordinary nodes')
    expect(PRESET_NOTE).toContain('edited rather than configured')
  })

  it('sits below the node chips', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('wf-chip-verify').compareDocumentPosition(getByTestId('wf-presets-title')) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })
})
