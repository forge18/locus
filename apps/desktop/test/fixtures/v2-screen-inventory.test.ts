import { describe, expect, it } from 'vitest'
import { V2_FIXTURE_ROUTES } from '../../src/fixtures/v2-screen-inventory'

const EXPECTED_SCREENS = [
  '01-inbox',
  '02-dashboard',
  '03-project-settings',
  '04-project-analytics',
  '05-plan-conversation',
  '06-plan-spec',
  '07-plan-tasks-decompose',
  '08-develop',
  '09-automate-kanban',
  '10-automate-agents',
  '11-dispatch-autorun',
  '12-dispatch-schedules',
  '13-dispatch-runs',
  '14-memory-short-term',
  '15-memory-long-term',
  '16-memory-artifacts',
  '17-memory-wiki',
  '18-review-telemetry',
  '19-settings-guardrails',
  '20-workshop-agents',
  '21-workshop-cli',
  '22-workshop-commands',
  '23-workshop-harnesses',
  '24-workshop-hooks',
  '25-workshop-linters',
  '26-workshop-output-styles',
  '27-workshop-providers',
  '28-workshop-rules',
  '29-workshop-skills',
  '30-workflows-visual',
  '31-workflows-governance',
] as const

describe('fixtures/v2-screen-inventory', () => {
  it('registers every delivered v2 screen exactly once', () => {
    expect(V2_FIXTURE_ROUTES.map((route) => route.screen)).toEqual(EXPECTED_SCREENS)
    expect(new Set(V2_FIXTURE_ROUTES.map((route) => route.id)).size).toBe(31)
  })

  it('gives every route a stable fixture id, label, scope, and screenshot', () => {
    for (const route of V2_FIXTURE_ROUTES) {
      expect(route.id).toMatch(/^[a-z][a-z0-9-]*$/)
      expect(route.label).not.toBe('')
      expect(route.screenshot).toBe(`${route.screen}.png`)
      expect(['global', 'project']).toContain(route.scope)
    }
  })
})
