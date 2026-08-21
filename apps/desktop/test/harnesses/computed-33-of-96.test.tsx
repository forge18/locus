import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { useExtensionTypes, useHarnessSummary, useHarnesses } from '../../src/data/harnesses'
import { read } from '../css'

const mount = () => render(() => <HarnessesView />)

describe('harnesses/computed-33-of-96', () => {
  it('reports 33 of 96', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-downgrade-line').textContent).toContain('33 of 96')
  })

  it('computes 96 as harnesses times extension types', () => {
    expect(useHarnessSummary().entries).toBe(useHarnesses().length * useExtensionTypes().length)
    expect(useHarnessSummary().entries).toBe(96)
  })

  it('computes 33 by counting entries that name what was lost', () => {
    const counted = useHarnesses()
      .flatMap((h) => h.extensions)
      .filter((e) => e.weakerThanNative).length
    expect(counted).toBe(33)
    expect(useHarnessSummary().downgrades).toBe(33)
  })

  it('reports neither of the handoff copy’s stale figures', () => {
    const { getByTestId } = mount()
    const text = getByTestId('harnesses-foot').textContent!
    expect(text).not.toContain('27 of 88')
    expect(text).not.toContain('88')
  })

  it('holds both numbers only in the generated file', () => {
    expect(read('fixtures/generated/harnesses.ts')).toContain('DOWNGRADE_COUNT = 33')
    expect(read('screens/workshop/HarnessesView.tsx')).not.toMatch(/\b(33|96)\b/)
  })
})
