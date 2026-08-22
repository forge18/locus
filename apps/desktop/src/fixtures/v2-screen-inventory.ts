export type FixtureScope = 'global' | 'project'

export interface V2FixtureRoute {
  /** Stable route identity; the v2 navigation resolver will consume it in task 4. */
  id: string
  label: string
  scope: FixtureScope
  /** Screenshot stem in `docs/design_handoff_locus_v2/screenshots/`. */
  screen: string
  screenshot: string
}

function fixtureRoute(
  id: string,
  label: string,
  scope: FixtureScope,
  screen: string,
): V2FixtureRoute {
  return Object.freeze({ id, label, scope, screen, screenshot: `${screen}.png` })
}

/**
 * The delivered v2 fixture inventory, in the handoff's navigation order.
 *
 * This deliberately describes routes before the v2 shell replaces the v1
 * resolver. Subsequent M0.6 tasks consume this list rather than maintaining a
 * second, partial screen inventory in a component or test.
 */
export const V2_FIXTURE_ROUTES = Object.freeze([
  fixtureRoute('inbox', 'Inbox', 'global', '01-inbox'),
  fixtureRoute('dashboard', 'Dashboard', 'global', '02-dashboard'),
  fixtureRoute('project-settings', 'Project Settings', 'global', '03-project-settings'),
  fixtureRoute('project-analytics', 'Project Analytics', 'global', '04-project-analytics'),
  fixtureRoute('plan-conversation', 'Plan Conversation', 'project', '05-plan-conversation'),
  fixtureRoute('plan-spec', 'Plan Spec', 'project', '06-plan-spec'),
  fixtureRoute('plan-tasks', 'Plan Tasks & Cards', 'project', '07-plan-tasks-decompose'),
  fixtureRoute('develop', 'Develop', 'project', '08-develop'),
  fixtureRoute('automate-kanban', 'Automate Kanban', 'project', '09-automate-kanban'),
  fixtureRoute('automate-agents', 'Automate Agents', 'project', '10-automate-agents'),
  fixtureRoute('dispatch-autorun', 'Dispatch Autorun', 'global', '11-dispatch-autorun'),
  fixtureRoute('dispatch-schedules', 'Dispatch Schedules', 'global', '12-dispatch-schedules'),
  fixtureRoute('dispatch-runs', 'Dispatch Runs', 'global', '13-dispatch-runs'),
  fixtureRoute('memory-short-term', 'Memory Short-term', 'global', '14-memory-short-term'),
  fixtureRoute('memory-long-term', 'Memory Long-term', 'global', '15-memory-long-term'),
  fixtureRoute('memory-artifacts', 'Memory Artifacts', 'global', '16-memory-artifacts'),
  fixtureRoute('memory-wiki', 'Memory Wiki', 'global', '17-memory-wiki'),
  fixtureRoute('review-telemetry', 'Review Telemetry', 'project', '18-review-telemetry'),
  fixtureRoute('settings-guardrails', 'Settings Guardrails', 'global', '19-settings-guardrails'),
  fixtureRoute('workshop-agents', 'Workshop Agents', 'global', '20-workshop-agents'),
  fixtureRoute('workshop-cli', 'Workshop CLI', 'global', '21-workshop-cli'),
  fixtureRoute('workshop-commands', 'Workshop Commands', 'global', '22-workshop-commands'),
  fixtureRoute('workshop-harnesses', 'Workshop Harnesses', 'global', '23-workshop-harnesses'),
  fixtureRoute('workshop-hooks', 'Workshop Hooks', 'global', '24-workshop-hooks'),
  fixtureRoute('workshop-linters', 'Workshop Linters', 'global', '25-workshop-linters'),
  fixtureRoute('workshop-output-styles', 'Workshop Output Styles', 'global', '26-workshop-output-styles'),
  fixtureRoute('workshop-providers', 'Workshop Providers', 'global', '27-workshop-providers'),
  fixtureRoute('workshop-rules', 'Workshop Rules', 'global', '28-workshop-rules'),
  fixtureRoute('workshop-skills', 'Workshop Skills', 'global', '29-workshop-skills'),
  fixtureRoute('workflows-visual', 'Workflows Visual', 'global', '30-workflows-visual'),
  fixtureRoute('workflows-governance', 'Workflows Governance', 'global', '31-workflows-governance'),
] as const)
