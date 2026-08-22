import { GUARDRAIL_SECTIONS, PRIORITY_METHODS, SETTINGS_NAVIGATION } from '../fixtures/settings-guardrails'
import type { GuardrailSection } from '../fixtures/settings-guardrails'

export { PRIORITY_METHODS, SETTINGS_NAVIGATION }

/** Becomes: invoke("settings_guardrails") */
export function useGuardrails(): readonly GuardrailSection[] {
  return GUARDRAIL_SECTIONS
}
