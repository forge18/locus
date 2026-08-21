import { For, Show } from 'solid-js'
import { Icon } from '../../ui/Icon'
import { PLAN_STEPS } from '../../data/plan'
import type { PlanStep } from '../../data/plan'

export interface BreadcrumbProps {
  /** The step the plan is actually on. Everything before is done, after is ahead. */
  current: PlanStep
}

export type StepState = 'done' | 'current' | 'ahead'

export function stepState(step: PlanStep, current: PlanStep): StepState {
  const at = PLAN_STEPS.indexOf(current)
  const here = PLAN_STEPS.indexOf(step)
  return here < at ? 'done' : here === at ? 'current' : 'ahead'
}

/** Eight steps, three states, and which is which comes from the plan, not the markup. */
export function Breadcrumb(props: BreadcrumbProps) {
  return (
    <div class="crumbs" data-testid="breadcrumb">
      <For each={PLAN_STEPS}>
        {(step, i) => {
          const state = () => stepState(step, props.current)
          return (
            <span
              class={`crumb crumb-${state()}`}
              data-testid={`crumb-${step.toLowerCase()}`}
              data-state={state()}
              aria-current={state() === 'current' ? 'step' : undefined}
            >
              <Show when={state() === 'done'} fallback={<span>{i() + 1}</span>}>
                <Icon name="check" size={9} />
              </Show>
              {step}
            </span>
          )
        }}
      </For>
    </div>
  )
}
