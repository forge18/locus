import { describe, expect, it } from 'vitest'
import { useCanvas, useWorkflowEvents } from '../../src/data/workflow'

describe('canvas/one-event-source', () => {
  it('shares the normalized event collection with transcript projections', () => {
    expect(useCanvas().events).toBe(useWorkflowEvents())
  })
})
