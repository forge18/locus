import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Rail } from '../../src/shell/Rail'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('shell/shell.css')).find((r) => r.selector === sel)

describe('shell/rail-active', () => {
  it('marks exactly one item current', () => {
    const { getByTestId } = render(() => (
      <Rail view="telemetry" onNavigate={() => {}} inboxCount={0} />
    ))
    const current = getByTestId('rail').querySelectorAll('[aria-current="true"]')
    expect(current.length).toBe(1)
    expect(current[0].getAttribute('data-category')).toBe('review')
  })

  it('paints it --sf2 with the accent inset ring and accent text', () => {
    const body = rule(".rail-item[aria-current='true']")!.body
    expect(body).toContain('background: var(--sf2)')
    expect(body).toContain('box-shadow: var(--ring-sel-soft)')
    expect(body).toContain('color: var(--ac)')
  })

  it('resolves the ring from --ac, so retheming moves it', () => {
    expect(read('styles/tokens.css')).toContain('--ring-sel-soft: inset 0 0 0 1px var(--ac-ring)')
    expect(read('styles/tokens.css')).toContain('--ac-ring: color-mix(in srgb, var(--ac) 55%, transparent)')
  })

  it('leaves the inactive items in --mu', () => {
    expect(rule('.rail-item')!.body).toContain('color: var(--mu)')
  })
})
