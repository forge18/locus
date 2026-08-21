import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { VERB_CLASS } from '../../src/screens/automate/Transcript'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { EVENT_VERBS } from '../../src/types/event'
import { SESSION_DETAILS } from '../../src/fixtures/sessions'
import { read, rules } from '../css'

describe('agents/canonical-verbs-only', () => {
  it('has a class for each of the twelve, and only those twelve', () => {
    expect(Object.keys(VERB_CLASS).sort()).toEqual([...EVENT_VERBS].sort())
  })

  it('has a stylesheet rule for every one of them', () => {
    const selectors = rules(read('screens/screens.css')).map((r) => r.selector)
    for (const verb of EVENT_VERBS) {
      expect(selectors, verb).toContain(`.verb-${verb}`)
    }
  })

  it('has no rule for a verb that is not canonical', () => {
    const verbRules = rules(read('screens/screens.css'))
      .map((r) => r.selector)
      .filter((s) => s.startsWith('.verb-'))
      .map((s) => s.slice('.verb-'.length))
    expect(verbRules.sort()).toEqual([...EVENT_VERBS].sort())
  })

  it('uses only canonical verbs in every fixture transcript', () => {
    for (const session of SESSION_DETAILS) {
      for (const line of session.transcript) {
        expect(EVENT_VERBS, `${session.id}: ${line.verb}`).toContain(line.verb)
      }
    }
  })

  it('has nowhere to put a thirteenth', () => {
    expect((VERB_CLASS as Record<string, string>).tool_retry).toBe(undefined)
  })

  it('renders every verb it colours', () => {
    const { getByTestId } = render(() => <AgentsView />)
    const rendered = new Set(
      [...getByTestId('transcript').querySelectorAll('.transcript-line')].map((l) =>
        l.getAttribute('data-verb'),
      ),
    )
    for (const verb of rendered) expect(EVENT_VERBS).toContain(verb as never)
  })
})
