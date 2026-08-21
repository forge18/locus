import { For, Show } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { EXTENSION_LABELS, useExtensionTypes, useHarnessSummary, useHarnesses } from '../../data/harnesses'
import { TIERS, fallbackMarker, resolveTier } from '../../data/settings'
import type { ModelTier } from '../../types/core'

/** At four or more, the downgrades are the story rather than a footnote. */
const HEAVY = 4

export const HEADER_NOTE =
  'Mechanism lives in the file; policy lives here. Every harness has every capability — only the mechanism differs.'

/** The count comes from the registry too — a literal here goes stale on the next one. */
export const tuiNote = (count: number) =>
  `tui = false is required on all ${count}; a harness claiming true is refused at registration.`

/**
 * Every figure on this screen is computed from harnesses/*.toml. Registering a
 * thirteenth harness moves all of them without an edit here — which is the whole
 * argument for the registry being a file rather than a table in the source.
 */
export function HarnessesView() {
  const harnesses = useHarnesses()
  const summary = useHarnessSummary()
  const types = useExtensionTypes()

  return (
    <div class="harnesses" data-testid="harnesses">
      <header class="ws-head" data-testid="harnesses-head">
        <span class="ws-title" data-testid="harnesses-title">
          Registered harnesses{' '}
          <span class="mono" data-testid="harnesses-count">
            {summary.harnesses}
          </span>
        </span>
        <span class="ws-note" data-testid="harnesses-note">
          {HEADER_NOTE}
        </span>
        <div class="ws-actions">
          <div class="hn-legend" data-testid="harnesses-legend">
            <span class="hn-legend-native">
              <i />
              native
            </span>
            <span class="hn-legend-downgraded">
              <i />
              downgraded — each names its loss
            </span>
          </div>
          <Button variant="primary" data-testid="harnesses-register">
            <Icon name="plus" size={11} />
            Register a harness
          </Button>
        </div>
      </header>

      <div class="hn-grid" data-testid="harnesses-grid">
        <For each={harnesses}>
          {(harness) => {
            const downgrades = harness.extensions.filter((e) => e.weakerThanNative).length
            const heavy = downgrades >= HEAVY
            return (
              <article
                class={['hn-card', heavy ? 'hn-card-heavy' : ''].filter(Boolean).join(' ')}
                data-testid={`hn-card-${harness.name}`}
                data-heavy={heavy ? 'true' : undefined}
              >
                <div class="hn-card-head">
                  <span class="hn-name">{harness.name}</span>
                  <span class="hn-id">{harness.binary}</span>
                  <span
                    class={`hn-badge hn-badge-${harness.badge.variant}`}
                    data-testid={`hn-badge-${harness.name}`}
                    data-variant={harness.badge.variant}
                  >
                    {harness.badge.label}
                  </span>
                </div>

                <span class="hn-injection" data-testid={`hn-injection-${harness.name}`}>
                  injection: {harness.injection}
                </span>

                <div class="hn-tiers" data-testid={`hn-tiers-${harness.name}`}>
                  <For each={TIERS}>
                    {(tier: ModelTier) => {
                      const resolved = resolveTier(harness.name, tier)
                      const own = resolved.fellBackTo === null
                      return (
                        <div
                          class={['hn-tier', tier === 'high' ? 'hn-tier-high' : '']
                            .filter(Boolean)
                            .join(' ')}
                          data-testid={`hn-tier-${harness.name}-${tier}`}
                          data-fallback={own ? undefined : resolved.fellBackTo!}
                        >
                          {tier === 'medium' ? 'med' : tier}
                          <Show
                            when={own}
                            fallback={
                              <span
                                class="hn-tier-fallback"
                                data-testid={`hn-fallback-${harness.name}-${tier}`}
                              >
                                {fallbackMarker(resolved.fellBackTo!)}
                              </span>
                            }
                          >
                            <span class="hn-tier-value">{resolved.model}</span>
                          </Show>
                        </div>
                      )
                    }}
                  </For>
                </div>

                <div class="hn-bar" data-testid={`hn-bar-${harness.name}`}>
                  <For each={types}>
                    {(type) => {
                      const entry = harness.extensions.find((e) => e.type === type)!
                      const native = entry.weakerThanNative === null
                      return (
                        <span
                          class={`hn-seg hn-seg-${native ? 'native' : 'downgraded'}`}
                          data-testid={`hn-seg-${harness.name}-${type}`}
                          data-native={native ? 'true' : 'false'}
                          title={
                            native
                              ? `${EXTENSION_LABELS[type as never]}: native`
                              : `${EXTENSION_LABELS[type as never]}: ${entry.weakerThanNative}`
                          }
                        />
                      )
                    }}
                  </For>
                </div>

                <div class="hn-foot">
                  <span data-testid={`hn-extension-count-${harness.name}`}>
                    {types.length} extensions
                  </span>
                  <span
                    class={['hn-downgrades', heavy ? 'hn-downgrades-bad' : '']
                      .filter(Boolean)
                      .join(' ')}
                    data-testid={`hn-downgrades-${harness.name}`}
                  >
                    {downgrades === 0 ? 'all native' : `${downgrades} downgraded`}
                  </span>
                </div>
              </article>
            )
          }}
        </For>
      </div>

      <footer class="harnesses-foot" data-testid="harnesses-foot">
        <span data-testid="harnesses-downgrade-line">
          {summary.downgrades} of {summary.entries} entries are downgrades — the honest measure of how
          uneven the field is.
        </span>{' '}
        <span class="mono" data-testid="harnesses-tui-note">
          {tuiNote(summary.harnesses)}
        </span>
      </footer>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default HarnessesView
