import { describe, expect, it } from 'vitest'
import { fireEvent, render } from '@solidjs/testing-library'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('screens/desktop-projects', () => {
  it('renders the project settings fixture with its five policy sections', () => {
    const { getByTestId } = render(() => <ProjectsView />)

    expect(getByTestId('project-settings')).toBeTruthy()
    expect(getByTestId('project-harnesses')).toBeTruthy()
    expect(getByTestId('project-repos')).toBeTruthy()
    expect(getByTestId('project-base-context').textContent).toContain('1,240 / 1,500 tokens')
    expect(getByTestId('project-extensions')).toBeTruthy()
    expect(getByTestId('project-cli-tools')).toBeTruthy()
  })

  it('keeps one enabled harness as the agent default', () => {
    const { getByTestId, getAllByTestId } = render(() => <ProjectsView />)

    fireEvent.click(getByTestId('harness-default-codex'))

    expect(getByTestId('harness-default-codex').getAttribute('aria-pressed')).toBe('true')
    expect(getByTestId('harness-default-claude').getAttribute('aria-pressed')).toBe('false')
    expect(getAllByTestId(/harness-default-/)).toHaveLength(4)
  })

  it('switches to analytics with project counters, agents, and per-model token data', () => {
    const { getByTestId, getAllByTestId } = render(() => <ProjectsView />)

    fireEvent.click(getByTestId('project-tab-analytics'))

    expect(getByTestId('project-analytics')).toBeTruthy()
    expect(getAllByTestId('project-tag-counter')).toHaveLength(6)
    expect(getAllByTestId('project-agent')).toHaveLength(6)
    expect(getAllByTestId('project-model-row')).toHaveLength(4)
    expect(getByTestId('project-statistics').textContent).toContain('Cache read is the number worth watching')
  })
})
