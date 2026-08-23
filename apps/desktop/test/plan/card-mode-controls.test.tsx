import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { PlanTasksView } from '../../src/screens/plan/PlanTasksView'

describe('card mode controls', () => {
  it('offers carve-out mode and counts approval cards', () => {
    const { getByTestId } = render(() => <PlanTasksView />)
    fireEvent.click(getByTestId('granularity-spec-carve-outs'))
    expect(getByTestId('tasks-cards-summary').textContent).toContain('card')
  })
})
