import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { SIDEBAR_NOTE } from '../../src/data/agent-defs'
import { read, rules } from '../css'

const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/footer-note', () => {
  it('says Markdown plus a tool list, verbatim', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-side-foot').textContent).toBe(
      'Markdown plus a tool list. No canvas, no compile.',
    )
    expect(SIDEBAR_NOTE).toBe('Markdown plus a tool list. No canvas, no compile.')
  })

  it('sits at the foot under a top hairline', () => {
    const { getByTestId } = mount()
    const side = getByTestId('agentdefs-side')
    expect(side.children[1]).toBe(getByTestId('agentdefs-side-foot'))
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.agentdefs-side-foot')!.body,
    ).toContain('border-top: 1px solid var(--border-subtle)')
  })

  it('rules out a canvas, which is a claim the screen has to keep', () => {
    // The words appear in the comment stating the rule; what must be absent is
    // anything that would actually draw one.
    const source = read('screens/workshop/AgentDefsView.tsx')
    expect(source).not.toMatch(/GraphRenderer|<svg|WorkflowView|solid-flow/)
  })

  it('rules out a compile step too — the editor is prose, not a build', () => {
    const source = read('screens/workshop/AgentDefsView.tsx')
    expect(source).not.toMatch(/\bcompile\(|onCompile|Validate|validateGraph/)
  })
})
