import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { PRIMARY_SURFACE_NOTE, useGitPanel } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <DevelopView />)

describe('develop/footer', () => {
  it('is 52px under a top hairline', () => {
    const body = rule('.dev-footer').body
    expect(body).toContain('height: 52px')
    expect(body).toContain('border-top: 1px solid var(--line)')
  })

  it('offers Revert chunk as the secondary', () => {
    const { getByTestId } = mount()
    const button = getByTestId('dev-revert')
    expect(button.textContent).toContain('Revert chunk')
    expect(button.className).toContain('btn-secondary')
  })

  it('offers Open PR from this branch as the primary', () => {
    const { getByTestId } = mount()
    const button = getByTestId('dev-open-pr')
    expect(button.textContent).toContain('Open PR from this branch')
    expect(button.className).toContain('btn-primary')
  })

  it('shows the LSP line in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-lsp').textContent).toBe(useGitPanel().lsp)
    expect(getByTestId('dev-lsp').textContent).toBe('rust-analyzer · 0 errors · 2 hints')
    expect(rule('.dev-lsp').body).toContain('font-family: var(--fm)')
  })

  it('carries the note about what this surface is for, on the right', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-footer-note').textContent).toBe(PRIMARY_SURFACE_NOTE)
    expect(PRIMARY_SURFACE_NOTE).toBe(
      'Reviewing what an agent changed is the primary editor surface',
    )
    expect(rule('.dev-footer-note').body).toContain('margin-left: auto')
  })
})
