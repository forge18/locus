import { createSignal } from 'solid-js'
import { describe, expect, it } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { Combobox } from '../../src/ui/Combobox'
import { ContextMenu } from '../../src/ui/ContextMenu'
import { Tabs } from '../../src/ui/Tabs'
import { ToastRegion, notify } from '../../src/ui/Toast'
import { allSource, read, rules } from '../css'

const ui = read('ui/ui.css')
const rule = (sel: string) => rules(ui).find((r) => r.selector === sel)

describe('ui/kobalte-styled', () => {
  it('renders tabs with the token classes and reports the tab picked', () => {
    const [value, setValue] = createSignal('telemetry')
    const { getByText, container } = render(() => (
      <Tabs
        items={[
          { value: 'telemetry', label: 'Telemetry' },
          { value: 'runs', label: 'Runs' },
        ]}
        value={value()}
        onChange={setValue}
        label="Review"
      />
    ))
    expect(container.querySelector('.tabs-list')).not.toBe(null)
    expect(container.querySelector('.tab[data-selected]')!.textContent).toBe('Telemetry')
    getByText('Runs').click()
    expect(value()).toBe('runs')
  })

  it('opens a context menu with the token classes', async () => {
    const { getByTestId } = render(() => (
      <ContextMenu heading="Session" actions={[{ label: 'Reassign', onSelect: () => {} }]}>
        <div>weaver · builder@4</div>
      </ContextMenu>
    ))
    getByTestId('context-menu-trigger').dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true }),
    )
    await waitFor(() => expect(document.querySelector('.menu')).not.toBe(null))
    expect(document.querySelector('.menu-section')!.textContent).toBe('Session')
    expect(document.querySelector('.menu-item')!.textContent).toBe('Reassign')
  })

  it('renders a combobox control on the surface ground', () => {
    const { getByTestId } = render(() => (
      <Combobox
        options={[{ value: 'tapestry', label: 'tapestry' }]}
        value={null}
        onChange={() => {}}
        label="Project"
      />
    ))
    expect(getByTestId('combobox-input').className).toBe('combobox-input')
    expect(rule('.combobox-control')!.body).toContain('background: var(--surface-raised)')
  })

  it('shows a toast in the region, for work the reader is not watching', async () => {
    render(() => <ToastRegion />)
    notify({ title: 'Run finished', description: 'tapestry · builder@4' })
    await waitFor(() => expect(document.querySelector('.toast')).not.toBe(null))
    expect(document.querySelector('.toast')!.textContent).toContain('Run finished')
    expect(document.querySelector('.toast-region')).not.toBe(null)
  })

  it('styles every Kobalte surface from tokens rather than its own defaults', () => {
    for (const sel of ['.tab', '.menu', '.menu-item', '.combobox-control', '.toast', '.tooltip']) {
      const r = rule(sel)
      expect(r, `missing ${sel}`).toBeDefined()
      expect(r!.body, `${sel} paints a raw color`).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    }
  })

  it('imports Kobalte from source and never a published shadcn package', () => {
    for (const [file, contents] of allSource()) {
      if (!file.startsWith('ui/')) continue
      expect(contents, `${file}`).not.toMatch(/from '[^']*shadcn/)
    }
  })
})
