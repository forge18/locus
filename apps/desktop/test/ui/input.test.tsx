import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Input, Textarea } from '../../src/ui/Input'
import { read, rules } from '../css'

const css = read('ui/ui.css')
const rule = (sel: string) => rules(css).find((r) => r.selector === sel)

describe('ui/input', () => {
  it('grounds both controls on --sf', () => {
    expect(rule('.input')!.body).toContain('background: var(--surface-raised)')
  })

  it('shares one class, so an input and a textarea cannot drift apart', () => {
    const input = render(() => <Input placeholder="Filter runs" />)
    const textarea = render(() => <Textarea placeholder="Comment steers the agent" />)
    expect(input.container.querySelector('input')!.className).toBe('input')
    expect(textarea.container.querySelector('textarea')!.className).toBe('input')
  })

  it('sets a mono control in --fm for locators, paths and ids', () => {
    const { container } = render(() => <Input mono value="locus://tapestry/session/" />)
    expect(container.querySelector('input')!.className).toContain('mono')
  })

  it('gives the caret the accent, and focus the accent border', () => {
    expect(rule('.input')!.body).toContain('caret-color: var(--action-attention)')
    expect(rule('.input:focus-visible')!.body).toContain('border-color: var(--action-attention)')
  })

  it('gives the textarea a real starting height and lets it grow', () => {
    const body = rule('textarea.input')!.body
    expect(body).toMatch(/min-height:\s*\d+px/)
    expect(body).toContain('resize: vertical')
  })

  it('passes value and input events through', () => {
    let seen = ''
    const { container } = render(() => (
      <Input onInput={(e) => (seen = e.currentTarget.value)} />
    ))
    const el = container.querySelector('input')!
    el.value = 'weaver'
    el.dispatchEvent(new Event('input', { bubbles: true }))
    expect(seen).toBe('weaver')
  })
})
