// schema: core.settings
// replaced by: invoke("settings_model_tiers")
//
// Mechanism lives in harnesses/*.toml; policy lives here. Which model `high`
// means is never the file's business.

import type { ModelTier, ModelTierSetting } from '../types/core'

export const TIERS: ModelTier[] = ['low', 'medium', 'high', 'xhigh']

/**
 * Per-harness tier mappings. A tier with no entry is unset, and an agent asking
 * for it **falls back up, never down** — falling down would answer a hard
 * question with a cheap model and look like a bad agent rather than a bad setting.
 */
const MAPPINGS: Record<string, Partial<Record<ModelTier, string>>> = {
  claude: { low: 'haiku-4.5', medium: 'sonnet-4.6', high: 'opus-4.6' },
  codex: { low: 'gpt-5.2-mini', medium: 'gpt-5.2', high: 'gpt-5.2-pro' },
  copilot: { low: 'gpt-5.2-mini', medium: 'gpt-5.2', high: 'opus-4.6' },
  pi: { low: 'pi-small', medium: 'pi-base', high: 'pi-large', xhigh: 'pi-large-thin' },
  omp: { low: 'omp-lite', medium: 'omp', high: 'omp-pro' },
  gemini: { low: 'gemini-3-flash', medium: 'gemini-3-pro', high: 'gemini-3-ultra' },
  hermes: { medium: 'hermes-3', high: 'hermes-3-xl' },
  cursor: { low: 'composer-2-mini', medium: 'composer-2', high: 'composer-2-max' },
  antigravity: { low: 'gemini-3-flash', medium: 'gemini-3-pro', high: 'gemini-3-ultra' },
  aider: { medium: 'sonnet-4.6', high: 'opus-4.6' },
  dsh: { low: 'haiku-4.5', medium: 'sonnet-4.6', high: 'opus-4.6' },
  opencode: { low: 'haiku-4.5', medium: 'sonnet-4.6', high: 'opus-4.6' },
}

export const MODEL_TIERS: ModelTierSetting[] = Object.entries(MAPPINGS).flatMap(
  ([harness, mapping]) =>
    TIERS.map((tier) => ({ harness, tier, model: mapping[tier] ?? null })),
)

/** Results of the registry's configured `list_argv` discovery, keyed by harness.
 * Missing entries deliberately mean free text; an empty array would mean discovery
 * ran and found no available models. */
export const DISCOVERED_MODEL_IDS: Record<string, string[]> = {
  aider: ['sonnet-4.6', 'opus-4.6'],
  opencode: ['haiku-4.5', 'sonnet-4.6', 'opus-4.6'],
}

/**
 * Which tier an unmapped one resolves to, **per harness**. This is a setting, not
 * a rule baked into the resolver: what a given harness should reach for when a
 * tier is unset depends on what that harness actually has, and no global constant
 * can know that.
 *
 * A harness with no entry here has no fallback at all — Locus passes no model
 * flag and the harness runs on whatever it would have chosen itself, which is
 * PLAN.md's "unset means the harness's default".
 */
export const TIER_FALLBACK: Record<string, ModelTier> = {
  claude: 'high',
  codex: 'high',
  copilot: 'high',
  pi: 'xhigh',
  omp: 'high',
  gemini: 'high',
  hermes: 'high',
  cursor: 'high',
  antigravity: 'high',
  aider: 'medium',
  dsh: 'high',
  opencode: 'high',
}
