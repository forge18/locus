import { For } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { WikiGraph } from './WikiGraph'
import {
  LINT_CLEAN_LINE,
  MEMORY_DISTINCTION,
  useContradictions,
  useWikiLint,
} from '../../data/wiki'
import type { LintFinding } from '../../data/wiki'

const LINT_ICON: Record<LintFinding['kind'], string> = {
  orphan: 'warning',
  broken_link: 'link-break',
  unnamed_entity: 'cube',
  unsourced_assertion: 'seal-question',
}

const LINT_LABEL: Record<LintFinding['kind'], (f: LintFinding) => string> = {
  orphan: (f) => `${f.count} orphan pages — ${f.detail}`,
  broken_link: (f) => `${f.count} broken link — ${f.detail}`,
  unnamed_entity: (f) => `${f.count} ${f.detail}`,
  unsourced_assertion: (f) => `${f.count} ${f.detail}`,
}

export function WikiSidebar() {
  return (
    <aside class="wiki-side" data-testid="wiki-side">
      <span class="wiki-side-title">Graph</span>
      <WikiGraph />

      <span class="wiki-side-title">
        Contradictions
        <span class="wiki-side-note">flagged at ingest, not at query</span>
      </span>
      <For each={useContradictions()}>
        {(contradiction) => (
          <section
            class="output-card contradiction"
            data-testid={`contradiction-${contradiction.id}`}
          >
            <span class="contradiction-claim" data-testid="contradiction-claim">
              {contradiction.claim}
            </span>
            <For each={contradiction.values}>
              {(side, i) => (
                <span class="contradiction-side" data-testid={`contradiction-side-${i()}`}>
                  <span data-testid={`contradiction-value-${i()}`}>{side.value}</span>
                  <span class="contradiction-source" data-testid={`contradiction-source-${i()}`}>
                    — {side.source}, {side.age}
                  </span>
                </span>
              )}
            </For>
            <div class="contradiction-actions">
              <Button variant="primary" data-testid="contradiction-adjudicate">
                Adjudicate
              </Button>
              <Button variant="secondary" data-testid="contradiction-board-card">
                Board card
              </Button>
            </div>
          </section>
        )}
      </For>

      <span class="wiki-side-title">Locus wiki lint</span>
      <section class="output-card" data-testid="wiki-lint">
        <For each={useWikiLint()}>
          {(finding) => (
            <div class="lint-row" data-testid={`lint-${finding.kind}`}>
              <Icon name={LINT_ICON[finding.kind]} size={11} style={{ 'flex-shrink': 0 }} />
              <span>{LINT_LABEL[finding.kind](finding)}</span>
            </div>
          )}
        </For>
        <div class="lint-row lint-clean" data-testid="lint-clean">
          <Icon name="check-circle" size={11} style={{ 'flex-shrink': 0 }} />
          <span>{LINT_CLEAN_LINE}</span>
        </div>
      </section>

      <footer class="wiki-footer" data-testid="wiki-footer">
        {MEMORY_DISTINCTION}
      </footer>
    </aside>
  )
}
