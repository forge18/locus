import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectRail } from '../../src/shell/ProjectRail'

describe('desktop rail navigation', () => {
  it('emits a canonical desktop locator when a route is clicked', () => {
    const locators: string[] = []
    const { getByText } = render(() => <ProjectRail selectedProject="demo" onNavigate={(locator) => locators.push(locator)} />)
    fireEvent.click(getByText('Inbox'))
    expect(locators).toEqual(['locus://global/inbox'])
  })

  it('routes project controls and project selection with explicit project scope', () => {
    const locators: string[] = []
    const { getByTestId, getByText } = render(() => (
      <ProjectRail
        selectedProject="demo"
        projects={['demo', 'other']}
        onNavigate={(locator) => locators.push(locator)}
      />
    ))
    fireEvent.click(getByText('Plan'))
    fireEvent.click(getByText('Develop'))
    fireEvent.click(getByText('Automate'))
    fireEvent.click(getByText('Review'))
    fireEvent.click(getByTestId('project-switcher-option-other'))
    expect(locators).toEqual([
      'locus://project/demo/plan-conversation',
      'locus://project/demo/develop',
      'locus://project/demo/automate-kanban',
      'locus://project/demo/review-telemetry',
      'locus://project/other/plan-conversation',
    ])
  })

  it('routes expandable section controls and their route buttons', () => {
    const locators: string[] = []
    const { getByText } = render(() => (
      <ProjectRail selectedProject="demo" onNavigate={(locator) => locators.push(locator)} />
    ))
    fireEvent.click(getByText('Memory'))
    fireEvent.click(getByText('Short-term'))
    fireEvent.click(getByText('Workshop'))
    fireEvent.click(getByText('Agents'))
    expect(locators).toEqual([
      'locus://global/memory-short-term',
      'locus://global/memory-short-term',
      'locus://global/workshop-agents',
      'locus://global/workshop-agents',
    ])
  })
})
