import { describe, expect, it } from 'vitest'
import { WORKFLOW_AUTHORING_ROUTES } from '../../src/workflow-authoring/routes'

describe('workflow/governance', () => {
  it('keeps the Visual and Governance authoring routes separate from executions', () => {
    expect(WORKFLOW_AUTHORING_ROUTES).toEqual([
      { id: 'workflows-visual', label: 'Workflows Visual' },
      { id: 'workflows-governance', label: 'Workflows Governance' },
    ])
    for (const route of WORKFLOW_AUTHORING_ROUTES) {
      expect(route).not.toHaveProperty('executionId')
      expect(route).not.toHaveProperty('runId')
    }
  })
})
