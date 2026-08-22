import { V2_FIXTURE_ROUTES } from '../fixtures/v2-screen-inventory'
import type { V2FixtureRoute } from '../fixtures/v2-screen-inventory'

/** Workflow authoring has two definition-only surfaces. */
export const WORKFLOW_AUTHORING_ROUTE_IDS = ['workflows-visual', 'workflows-governance'] as const

type WorkflowAuthoringRouteId = (typeof WORKFLOW_AUTHORING_ROUTE_IDS)[number]

function isWorkflowAuthoringRoute(
  route: V2FixtureRoute,
): route is V2FixtureRoute & { id: WorkflowAuthoringRouteId } {
  return (WORKFLOW_AUTHORING_ROUTE_IDS as readonly string[]).includes(route.id)
}

/**
 * Execution and result routes belong to the run viewer, never either authoring
 * surface. The v2 fixture inventory remains the source of route labels.
 */
export const WORKFLOW_AUTHORING_ROUTES = Object.freeze(
  V2_FIXTURE_ROUTES.filter(isWorkflowAuthoringRoute).map(({ id, label }) => ({ id, label })),
)

export type WorkflowAuthoringRoute = (typeof WORKFLOW_AUTHORING_ROUTES)[number]
