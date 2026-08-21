import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Strip } from '../../src/shell/Strip'
import type { StripCard } from '../../src/data/strip'

const cards: StripCard[] = [
  { id: 'run-a', kind: 'agent', project: 'alpha', agent: 'builder@1', role: 'builder', status: 'running', tool: 'edit', tokens: 1, idleMinutes: 0 },
  { id: 'run-b', kind: 'agent', project: 'beta', agent: 'builder@1', role: 'builder', status: 'running', tool: 'test', tokens: 1, idleMinutes: 0 },
  { id: 'run-c', kind: 'agent', project: 'gamma', agent: 'reviewer@1', role: 'reviewer', status: 'running', tool: 'read', tokens: 1, idleMinutes: 0 },
]

describe('strip/cross-project', () => {
  it('keeps a third run from another project in the same running strip', () => {
    const { getByTestId } = render(() => <Strip cards={cards} />)

    expect(getByTestId('strip-card-run-a').textContent).toContain('alpha')
    expect(getByTestId('strip-card-run-b').textContent).toContain('beta')
    expect(getByTestId('strip-card-run-c').textContent).toContain('gamma')
  })
})
