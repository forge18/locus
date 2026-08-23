import { Desktop_FIXTURE_ROUTES } from '../fixtures/desktop-screen-inventory'
import type { DesktopFixtureRoute } from '../fixtures/desktop-screen-inventory'

/** Workflow authoring has two definition-only surfaces. */
export const WORKFLOW_AUTHORING_ROUTE_IDS = ['workflows-visual', 'workflows-governance'] as const

type WorkflowAuthoringRouteId = (typeof WORKFLOW_AUTHORING_ROUTE_IDS)[number]

function isWorkflowAuthoringRoute(
  route: DesktopFixtureRoute,
): route is DesktopFixtureRoute & { id: WorkflowAuthoringRouteId } {
  return (WORKFLOW_AUTHORING_ROUTE_IDS as readonly string[]).includes(route.id)
}

/**
 * Execution and result routes belong to the run viewer, never either authoring
 * surface. The desktop fixture inventory remains the source of route labels.
 */
export const WORKFLOW_AUTHORING_ROUTES = Object.freeze(
  Desktop_FIXTURE_ROUTES.filter(isWorkflowAuthoringRoute).map(({ id, label }) => ({ id, label })),
)

export type WorkflowAuthoringRoute = (typeof WORKFLOW_AUTHORING_ROUTES)[number]
