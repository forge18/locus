import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { useProse } from '../../src/data/agent-defs'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/prose', () => {
  it('is 13px on a 1.65 line', () => {
    const body = rule('.agentdefs-prose').body
    expect(body).toContain('font-size: var(--t-lead)')
    expect(body).toContain('line-height: 1.65')
  })

  it('is capped at 660px', () => {
    expect(rule('.agentdefs-prose').body).toContain('max-width: 660px')
  })

  it('is Inter, not mono — this half is prose', () => {
    expect(rule('.agentdefs-prose').body).not.toContain('font-family')
    expect(read('styles/type.css')).toContain('font-family: var(--fs)')
  })

  it('renders one paragraph per entry', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-prose').querySelectorAll('p').length).toBe(useProse().length)
  })

  it('sits under the frontmatter', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId('agentdefs-frontmatter').compareDocumentPosition(
        getByTestId('agentdefs-prose'),
      ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })
})
