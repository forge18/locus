import { For, createSignal } from 'solid-js'
import { Button } from '../../ui/Button'
import { Textarea } from '../../ui/Input'
import { SPEC_REQUIREMENTS } from '../../data/plan'
import type { SpecRequirement } from '../../data/plan'

export function PlanSpecView() {
  const [requirements, setRequirements] = createSignal(SPEC_REQUIREMENTS)
  const [resolved, setResolved] = createSignal<string[]>([])

  const updateRequirement = (id: string, body: string) => {
    setRequirements((current) => current.map((item) => (item.id === id ? { ...item, body } : item)))
  }

  const findingText = (requirement: SpecRequirement) =>
    resolved().includes(requirement.id) ? 'Finding resolved' : requirement.finding

  return (
    <section class="plan-spec" data-testid="plan-spec">
      <header class="plan-spec-toolbar">
        <span class="mono plan-spec-file">spec.md</span>
        <span class="plan-spec-meta">desktop · 14 requirements · you edited R-07 at 09:44</span>
        <span class="plan-unsaved" data-testid="spec-unsaved">unsaved</span>
        <div class="plan-spec-actions">
          <Button variant="ghost">Revert</Button>
          <Button variant="secondary">History</Button>
          <Button variant="primary">Save &amp; re-audit</Button>
        </div>
      </header>

      <div class="plan-spec-body">
        <article class="plan-spec-document">
          <h2 class="plan-spec-title"># Provenance beats recency in memory conflicts</h2>
          <p class="plan-spec-intro">
            Edited here, the spec is the artefact the board is built from. Requirement ids are stable: a task already on the board keeps pointing at the requirement it came from, even after a rewrite.
          </p>
          <h3 class="plan-spec-section">## 3 · Conflict resolution</h3>
          <For each={requirements()}>
            {(requirement) => (
              <div class="plan-requirement" data-testid={`requirement-${requirement.id}`}>
                <label class="plan-requirement-id" for={`requirement-input-${requirement.id}`}>{requirement.id}</label>
                <div class="plan-requirement-content">
                  <Textarea
                    id={`requirement-input-${requirement.id}`}
                    value={requirement.body}
                    aria-label={`Requirement ${requirement.id}`}
                    onInput={(event) => updateRequirement(requirement.id, event.currentTarget.value)}
                  />
                  <div class="plan-requirement-finding" data-testid={`requirement-finding-${requirement.id}`}>
                    {findingText(requirement)}
                    {requirement.finding && !resolved().includes(requirement.id) && (
                      <Button variant="ghost" data-testid={`resolve-finding-${requirement.id}`} onClick={() => setResolved((ids) => [...ids, requirement.id])}>
                        Mark finding resolved
                      </Button>
                    )}
                  </div>
                </div>
              </div>
            )}
          </For>
          <Button variant="secondary" class="plan-add-requirement">+ Add requirement to §3</Button>
          <p class="plan-spec-note">
            Saving sends stage 5 back over the changed requirements only. Requirements already carried by a card on the board are marked, so you can see what a rewrite is about to contradict.
          </p>
        </article>
        <aside class="plan-outline" aria-label="Spec outline" data-testid="spec-outline">
          <span class="plan-outline-title">Outline</span>
          <span>1 · Scope</span>
          <span>2 · Trust boundaries</span>
          <span class="plan-outline-current">3 · Conflict resolution</span>
          <span>4 · Error conditions</span>
          <span>5 · Out of scope</span>
          <p>1 open finding · 1 requirement with no card</p>
        </aside>
      </div>
    </section>
  )
}
