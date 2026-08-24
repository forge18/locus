import { GUARDRAIL_SECTIONS, PRIORITY_METHODS, SETTINGS_NAVIGATION } from '../fixtures/settings-guardrails'
import type { GuardrailSection } from '../fixtures/settings-guardrails'

export { PRIORITY_METHODS, SETTINGS_NAVIGATION }

/** Becomes: invoke("settings_guardrails") */
export function useGuardrails(): readonly GuardrailSection[] {
  return GUARDRAIL_SECTIONS
}

/** Guardrail defaults are installation policy; updates apply only to future runs. */
export interface GuardrailChange {
  id: string
  current: number
  next: number
  overrideRecorded?: boolean
}

export function validateGuardrailChanges(changes: readonly GuardrailChange[]) {
  const rejected = changes.filter((change) => change.next > change.current && !change.overrideRecorded)
  return { valid: rejected.length === 0, rejected }
}

export function saveGuardrailDefaults(changes: readonly GuardrailChange[]) {
  const result = validateGuardrailChanges(changes)
  if (!result.valid) return { ...result, saved: false }
  return { ...result, saved: true, appliesTo: 'runs started after saving' as const }
}
