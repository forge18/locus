import { fireEvent, render } from '@solidjs/testing-library'
import { afterEach, describe, expect, it } from 'vitest'
import { ProjectRail, RAIL_EXPANSION_STORAGE_KEY } from '../../src/shell/ProjectRail'

afterEach(() => localStorage.removeItem(RAIL_EXPANSION_STORAGE_KEY))

describe('shell/rail-expansion-persists', () => {
  it('restores Memory and Workshop expansion after remount', () => {
    const first = render(() => <ProjectRail selectedProject="locus" />)
    fireEvent.click(first.getByRole('button', { name: 'Memory' }))
    fireEvent.click(first.getByRole('button', { name: 'Workshop' }))
    first.unmount()

    const restored = render(() => <ProjectRail selectedProject="locus" />)
    expect(restored.getByTestId('memory-rail-links').hidden).toBe(false)
    expect(restored.getByTestId('workshop-rail-links').hidden).toBe(false)
  })
})
