import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ShellState, type ShellStateKind } from '../../src/shell/ShellState'

for (const kind of ['loading', 'empty', 'error'] as const satisfies readonly ShellStateKind[]) {
  describe('shell/state-families', () => {
    it(`renders the ${kind} shell state`, () => {
      const { getByTestId } = render(() => <ShellState kind={kind}>{kind}</ShellState>)
      expect(getByTestId(`shell-state-${kind}`).getAttribute('data-state')).toBe(kind)
    })
  })
}
