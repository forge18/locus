import { createSignal } from 'solid-js'
import { beforeEach, describe, expect, it } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { Sheet } from '../../src/ui/Sheet'
import { read, rules } from '../css'

// The app renders into #root. A sheet has to land inside it, not over the window.
beforeEach(() => {
  document.body.innerHTML = ''
  const root = document.createElement('div')
  root.id = 'root'
  document.body.appendChild(root)
})

function Harness() {
  const [open, setOpen] = createSignal(true)
  return (
    <>
      <Sheet open={open()} onOpenChange={setOpen} title="Run 8f21">
        <p>evidence: 2 runs, 41 events</p>
      </Sheet>
      <span data-testid="open">{String(open())}</span>
    </>
  )
}

describe('ui/sheet-in-place', () => {
  it('opens inside the app root, not as a sibling of it', async () => {
    render(() => <Harness />, { container: document.getElementById('root')! })
    await waitFor(() => expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null))
    const sheet = document.querySelector('[data-testid="sheet"]')!
    expect(document.getElementById('root')!.contains(sheet)).toBe(true)
  })

  it('never reaches for a second window — that is what a detached pane gets', () => {
    const source = read('ui/Sheet.tsx')
    expect(source).not.toMatch(/WebviewWindow|window\.open|new Window/)
  })

  it('shows the title and the body it was given', async () => {
    render(() => <Harness />, { container: document.getElementById('root')! })
    await waitFor(() => expect(document.body.textContent).toContain('Run 8f21'))
    expect(document.body.textContent).toContain('evidence: 2 runs, 41 events')
  })

  it('closes back through the caller, so the caller owns the state', async () => {
    const { getByTestId } = render(() => <Harness />, {
      container: document.getElementById('root')!,
    })
    await waitFor(() => expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null))
    ;(document.querySelector('.sheet-head button') as HTMLElement).click()
    await waitFor(() => expect(getByTestId('open').textContent).toBe('false'))
  })

  it('is anchored to its container, so the window chrome stays visible', () => {
    const rule = rules(read('ui/ui.css')).find((r) => r.selector === '.sheet')!
    expect(rule.body).toContain('position: absolute')
    expect(rules(read('ui/ui.css')).find((r) => r.selector === '.overlay')!.body).toContain(
      'position: absolute',
    )
  })
})
