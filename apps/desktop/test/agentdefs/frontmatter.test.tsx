import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { useFrontmatter } from '../../src/data/agent-defs'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/frontmatter', () => {
  it('grounds the block on --sf with a 2px accent left border', () => {
    const body = rule('.frontmatter').body
    expect(body).toContain('background: var(--surface-raised)')
    expect(body).toContain('border-left: 2px solid var(--action-attention)')
  })

  it('is mono at 15px on a 1.72 line', () => {
    const body = rule('.frontmatter').body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('font-size: var(--t-row)')
    expect(body).toContain('line-height: 1.72')
  })

  it('colours the keys #8fb8d6, through the token', () => {
    expect(rule('.frontmatter-key').body).toContain('color: var(--code-keyword)')
    expect(read('styles/tokens.css')).toContain('--code-keyword: #8fb8d6')
  })

  it('carries the six keys the design draws', () => {
    expect(useFrontmatter().map((f) => f.key)).toEqual([
      'harness',
      'model_tier',
      'tools',
      'skills',
      'rules',
      'memory_scope',
    ])
    const { getByTestId } = mount()
    for (const line of useFrontmatter()) {
      expect(getByTestId(`frontmatter-${line.key}`).textContent, line.key).toContain(line.value)
    }
  })

  it('marks each key, and only the key, as a key', () => {
    const { getByTestId } = mount()
    const row = getByTestId('frontmatter-tools')
    expect(row.querySelector('.frontmatter-key')!.textContent).toBe('tools')
    expect(row.textContent).toContain('[read_file, edit_file, run_command, rg]')
  })

  it('is fenced by --- on both sides, as frontmatter is', () => {
    const { getByTestId } = mount()
    const block = getByTestId('agentdefs-frontmatter')
    expect(block.children[0].textContent).toBe('---')
    expect(block.children[block.children.length - 1].textContent).toBe('---')
  })
})
