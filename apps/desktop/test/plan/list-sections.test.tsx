import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { usePlans } from '../../src/data/plan'

const mount = () => render(() => <PlanView />)
const count = (state: string) => usePlans().filter((p) => p.state === state).length

describe('plan/list-sections', () => {
  it('has the three sections, in order', () => {
    const { getByTestId } = mount()
    const sections = [...getByTestId('plan-list').querySelectorAll('.plan-section')].map(
      (s) => s.textContent,
    )
    expect(sections[0]).toContain('In progress')
    expect(sections[1]).toContain('Drafts — rejected, kept here')
    expect(sections[2]).toContain('Approved · on the board')
  })

  it('counts each section from the data', () => {
    const { getByTestId } = mount()
    for (const state of ['in_progress', 'draft_rejected', 'approved']) {
      expect(getByTestId(`plan-section-${state}`).textContent, state).toContain(
        String(count(state)),
      )
    }
  })

  it('keeps rejected drafts visible and reachable, rather than discarding them', () => {
    const { getByTestId } = mount()
    for (const plan of usePlans().filter((p) => p.state === 'draft_rejected')) {
      const card = getByTestId(`plan-card-${plan.id}`)
      expect(card.textContent).toContain(plan.title)
      card.click()
      expect(card.getAttribute('aria-selected')).toBe('true')
    }
  })

  it('leads with the block primary New plan', () => {
    const { getByTestId } = mount()
    const button = getByTestId('new-plan')
    expect(button.className).toContain('btn-primary')
    expect(button.className).toContain('btn-block')
    expect(button.textContent).toContain('New plan')
  })

  it('says what a plan starts from', () => {
    const { getByTestId } = mount()
    const note = getByTestId('new-plan-note').textContent!
    expect(note).toContain('goal')
    expect(note).toContain('target repo')
    expect(note).toContain('repos involved')
  })
})
