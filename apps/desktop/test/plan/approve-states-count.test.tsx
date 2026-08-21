import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { Recommendation } from '../../src/screens/plan/Recommendation'
import { usePlanOutputs, usePlanRecommendation } from '../../src/data/plan'

const recommendation = usePlanRecommendation()

describe('plan/approve-states-count', () => {
  it('names the number of tasks it would land', () => {
    const { getByTestId } = render(() => (
      <Recommendation recommendation={recommendation} onApprove={() => {}} />
    ))
    expect(getByTestId('recommendation-approve').textContent).toBe(
      'Approve — 4 tasks to the board',
    )
  })

  it('takes the count from the data, not from the label', () => {
    const { getByTestId } = render(() => (
      <Recommendation recommendation={{ ...recommendation, taskCount: 9 }} onApprove={() => {}} />
    ))
    expect(getByTestId('recommendation-approve').textContent).toBe(
      'Approve — 9 tasks to the board',
    )
  })

  it('matches the tasks actually drafted', () => {
    expect(recommendation.taskCount).toBe(usePlanOutputs().tasks.length)
  })

  it('is the block primary at the foot of the outputs rail', () => {
    const { getByTestId } = render(() => <PlanView />)
    const button = getByTestId('recommendation-approve')
    expect(button.className).toContain('btn-primary')
    expect(button.className).toContain('btn-block')
    const cards = [...getByTestId('plan-outputs').querySelectorAll('.output-card')]
    expect(cards[cards.length - 1].contains(button)).toBe(true)
  })
})
