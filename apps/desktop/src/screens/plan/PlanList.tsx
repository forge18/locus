import { For, Show } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { NEW_PLAN_NOTE, ONE_APPROVAL_RULE } from '../../data/plan'
import type { PlanState, PlanSummary } from '../../data/plan'

export interface PlanListProps {
  plans: PlanSummary[]
  selectedId: string
  onSelect: (id: string) => void
  onNewPlan: () => void
}

/** Rejected drafts are kept, not discarded — the section says so in as many words. */
const SECTIONS: Array<{ state: PlanState; label: string }> = [
  { state: 'in_progress', label: 'In progress' },
  { state: 'draft_rejected', label: 'Drafts — rejected, kept here' },
  { state: 'approved', label: 'Approved · on the board' },
]

export function PlanList(props: PlanListProps) {
  const inState = (state: PlanState) => props.plans.filter((p) => p.state === state)

  return (
    <div class="plan-list" data-testid="plan-list">
      <div class="plan-list-body">
        <Button variant="primary" block data-testid="new-plan" onClick={props.onNewPlan}>
          <Icon name="plus" size={11} />
          New plan
        </Button>
        <span class="plan-list-note" data-testid="new-plan-note">
          {NEW_PLAN_NOTE}
        </span>

        <For each={SECTIONS}>
          {(section) => (
            <>
              <div class="plan-section" data-testid={`plan-section-${section.state}`}>
                {section.label}
                <span class="mono" style={{ 'margin-left': 'var(--g-3)', color: 'var(--mu2)' }}>
                  {inState(section.state).length}
                </span>
              </div>
              <For each={inState(section.state)}>
                {(plan) => (
                  <button
                    type="button"
                    class={['plan-card', plan.state === 'approved' ? 'plan-card-approved' : '']
                      .filter(Boolean)
                      .join(' ')}
                    data-testid={`plan-card-${plan.id}`}
                    data-state={plan.state}
                    aria-selected={props.selectedId === plan.id ? 'true' : 'false'}
                    onClick={() => props.onSelect(plan.id)}
                  >
                    <span class="plan-card-title">{plan.title}</span>
                    <span class="plan-card-meta">
                      <Show when={plan.state === 'in_progress'}>
                        <Icon name="circle-notch" size={10} style={{ color: 'var(--ac)' }} />
                      </Show>
                      <span
                        class={plan.landed ? 'plan-card-landed' : ''}
                        data-testid={`plan-card-step-${plan.id}`}
                      >
                        {plan.stepLine}
                      </span>
                      <span class="plan-card-project" data-testid={`plan-card-project-${plan.id}`}>
                        {plan.project}
                      </span>
                    </span>
                  </button>
                )}
              </For>
            </>
          )}
        </For>
      </div>

      <footer class="plan-list-footer" data-testid="plan-list-footer">
        {ONE_APPROVAL_RULE}
      </footer>
    </div>
  )
}
