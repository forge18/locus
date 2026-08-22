import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectRail } from '../../src/shell/ProjectRail'

describe('shell/memory-expander', () => {
  it('reveals Memory links only after the Memory control expands', () => {
    const { getByRole, getByTestId } = render(() => <ProjectRail selectedProject="locus" />)
    const memory = getByRole('button', { name: 'Memory' })

    expect(getByTestId('memory-rail-links').hidden).toBe(true)
    fireEvent.click(memory)
    expect(getByTestId('memory-rail-links').hidden).toBe(false)
    expect(getByTestId('memory-rail-links').textContent).toContain('Short-term')
  })
})
