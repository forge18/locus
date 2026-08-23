import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { ENTRY_TYPE } from '../../src/data/extensions'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = (onNavigate: (v: string) => void = () => {}) =>
  render(() => <ExtensionsView onNavigate={onNavigate as never} />)

describe('extensions/agents-entry-card', () => {
  it('marks the agents card as the entry, and only it', () => {
    const { getByTestId } = mount()
    expect(getByTestId('type-card-agents').getAttribute('data-entry')).toBe('true')
    expect(getByTestId('type-grid').querySelectorAll('[data-entry="true"]').length).toBe(1)
    expect(ENTRY_TYPE).toBe('agents')
  })

  it('rings it in accent and gives it a pointer', () => {
    const body = rule('.type-card-entry').body
    expect(body).toContain('box-shadow: var(--ring-sel-soft)')
    expect(body).toContain('cursor: pointer')
  })

  it('carries the accent arrow-right', () => {
    const { getByTestId } = mount()
    const arrow = getByTestId('type-card-arrow')
    expect(arrow.querySelector('use')!.getAttribute('href')).toBe('#ph-arrow-right')
    expect(rule('.type-card-arrow').body).toContain('color: var(--action-attention)')
  })

  it('navigates to the drill-down when clicked', () => {
    const landed: string[] = []
    const { getByTestId } = mount((v) => landed.push(v))
    getByTestId('type-card-agents').click()
    expect(landed).toEqual(['agents'])
  })

  it('navigates from no other card', () => {
    const landed: string[] = []
    const { getByTestId } = mount((v) => landed.push(v))
    getByTestId('type-card-skills').click()
    getByTestId('type-card-linters').click()
    expect(landed).toEqual([])
  })
})
