import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render } from '@solidjs/testing-library'
import { Desktop_FIXTURE_ROUTES } from '../../src/fixtures/desktop-screen-inventory'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { TelemetryView } from '../../src/screens/review/TelemetryView'

const projectWorkIds = ['develop', 'automate-kanban', 'automate-agents', 'review-telemetry']

describe('screens/desktop-project-work', () => {
  it('covers the four project-scoped Develop, Automate, and Review fixtures', () => {
    const routes = Desktop_FIXTURE_ROUTES.filter((route) => projectWorkIds.includes(route.id))

    expect(routes.map((route) => route.id)).toEqual(projectWorkIds)
    expect(routes.every((route) => route.scope === 'project')).toBe(true)
  })

  it('switches Automate between its Kanban and agent-list fixtures', () => {
    const showAgents = vi.fn()
    const kanban = render(() => <KanbanView onShowAgents={showAgents} />)
    const switcher = kanban.getByTestId('automate-view-switcher')

    expect(switcher.textContent).toBe('KanbanList')
    expect(kanban.getByTestId('automate-kanban-tab').getAttribute('aria-pressed')).toBe('true')
    fireEvent.click(kanban.getByTestId('automate-list-tab'))
    expect(showAgents).toHaveBeenCalledOnce()
    kanban.unmount()

    const showKanban = vi.fn()
    const agents = render(() => <AgentsView onShowKanban={showKanban} />)
    expect(agents.getByTestId('automate-list-tab').getAttribute('aria-pressed')).toBe('true')
    fireEvent.click(agents.getByTestId('automate-kanban-tab'))
    expect(showKanban).toHaveBeenCalledOnce()
  })

  it('keeps the Develop diff and Review telemetry fixtures mounted', () => {
    const develop = render(() => <DevelopView />)
    expect(develop.getByTestId('develop')).toBeTruthy()
    expect(develop.getByTestId('diff')).toBeTruthy()
    develop.unmount()

    const review = render(() => <TelemetryView />)
    expect(review.getByTestId('telemetry')).toBeTruthy()
    expect(review.getByTestId('tm-filters')).toBeTruthy()
    expect(review.getByTestId('tm-sessions')).toBeTruthy()
  })
})
