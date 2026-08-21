import { MODEL_TIERS, TIER_FALLBACK } from '../fixtures/settings'
import type { ModelTier, ModelTierSetting } from '../types/core'

export { TIERS, TIER_FALLBACK } from '../fixtures/settings'

/** Becomes: invoke("settings_model_tiers") */
export function useModelTiers(harness?: string): ModelTierSetting[] {
  return harness ? MODEL_TIERS.filter((m) => m.harness === harness) : MODEL_TIERS
}

/**
 * Becomes: invoke("resolve_model_tier", { harness, tier })
 *
 * An unmapped tier resolves to the tier that harness's settings nominate — see
 * `TIER_FALLBACK`. The fallback is configuration, not a rule in the resolver:
 * which tier is the sensible landing place depends on what the harness has, and
 * a global constant would be wrong for some harness the day it is registered.
 *
 * With no fallback configured, nothing resolves: Locus passes no model flag and
 * the harness runs on its own default. A new harness is usable the moment it is
 * registered, unconfigured.
 */
export function resolveTier(
  harness: string,
  tier: ModelTier,
): { model: string | null; fellBackTo: ModelTier | null } {
  const mapped = (t: ModelTier) =>
    MODEL_TIERS.find((m) => m.harness === harness && m.tier === t)?.model ?? null

  const own = mapped(tier)
  if (own) return { model: own, fellBackTo: null }

  const configured = TIER_FALLBACK[harness]
  if (!configured || configured === tier) return { model: null, fellBackTo: null }

  const fallback = mapped(configured)
  return fallback ? { model: fallback, fellBackTo: configured } : { model: null, fellBackTo: null }
}

/**
 * Becomes: invoke("settings_tier_fallback", { harness })
 *
 * The tier this harness's settings nominate, or null where none is configured.
 */
export function fallbackTierFor(harness: string): ModelTier | null {
  return TIER_FALLBACK[harness] ?? null
}

/**
 * Becomes: nothing — this is presentation, not a call.
 *
 * The marker an unmapped tier shows: where it resolved to, and that it went up.
 */
export function fallbackMarker(fellBackTo: ModelTier): string {
  return `↑ ${fellBackTo}`
}
