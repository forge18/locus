import { GUARDRAIL_SECTIONS, PRIORITY_METHODS, SETTINGS_NAVIGATION } from '../fixtures/settings-guardrails'
import type { GuardrailSection } from '../fixtures/settings-guardrails'
import { dataProvider } from './provider'
import type { Envelope } from './envelope'

export { PRIORITY_METHODS, SETTINGS_NAVIGATION }

export function fetchGuardrails(): Promise<Envelope<GuardrailSection[]>> {
  return dataProvider().query<GuardrailSection>('settings_guardrails')
}

export interface GuardrailSettingsPayload {
  maxIterations: number
  tokenBudget: number | null
  stuckIterations: number
  killAndReassign: boolean
  globalParallelism: number
  perProjectParallelism: number
  priorityMethod: string
  tieBreak: string
  changeLinesCeiling: number | null
  changeFilesCeiling: number | null
  networkTier: string
  blockSystemChanges: boolean
  autopilot: boolean
}

export function saveGuardrails(
  request: GuardrailSettingsPayload,
): Promise<Envelope<GuardrailSection[]>> {
  return dataProvider().queryOne<GuardrailSection[]>('settings_guardrails_set', {
    request,
  })
}

/** Becomes: invoke("settings_guardrails") — demo-only hook retained for tests. */
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
