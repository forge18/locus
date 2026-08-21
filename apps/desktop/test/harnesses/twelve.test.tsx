import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { useHarnessSummary, useHarnesses } from '../../src/data/harnesses'
import { read } from '../css'

const mount = () => render(() => <HarnessesView />)

describe('harnesses/twelve', () => {
  it('renders twelve cards', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-grid').querySelectorAll('.hn-card').length).toBe(12)
  })

  it('takes the count from the registry, not from a literal', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-count').textContent).toBe('12')
    expect(useHarnessSummary().harnesses).toBe(12)
    expect(useHarnesses().length).toBe(12)
  })

  it('names the twelve the repo actually registers', () => {
    expect(useHarnesses().map((h) => h.name)).toEqual([
      'aider', 'antigravity', 'claude', 'codex', 'copilot', 'cursor',
      'dsh', 'gemini', 'hermes', 'omp', 'opencode', 'pi',
    ])
  })

  it('holds no numeric literal in the screen source', () => {
    const source = read('screens/workshop/HarnessesView.tsx')
    expect(source).not.toMatch(/\b(12|33|96|88|27)\b/)
  })

  it('reports every one of them with tui = false, which is why they are here', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-tui-note').textContent).toContain('tui = false is required on all 12')
  })
})
