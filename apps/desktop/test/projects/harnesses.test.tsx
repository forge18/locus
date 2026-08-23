import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project harnesses', () => {
  it('renders the allow-list and router fallback summary', () => {
    const { getByTestId, getByText } = render(() => <ProjectsView />)

    expect(getByTestId('project-harnesses').textContent).toContain('claude')
    expect(getByTestId('project-router-summary').textContent).toContain('agent default')
    expect(getByText('codex').textContent).toBe('codex')
  })
})
