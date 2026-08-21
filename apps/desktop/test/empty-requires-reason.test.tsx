import { execFileSync } from 'node:child_process'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { EmptyPane, type EmptyPaneProps } from '../src/ui/EmptyPane'

// The real assertion is the line below: `reason` is required, so omitting it is a
// type error, and `@ts-expect-error` only holds while that stays true. Make `reason`
// optional and the directive goes unused, which is itself an error (TS2578).
// @ts-expect-error — reason is not optional
const withoutReason: EmptyPaneProps = { icon: 'tray' }
void withoutReason

describe('empty-requires-reason', () => {
  it('types reason as required, so a pane cannot render "No items"', () => {
    const root = resolve(__dirname, '..')
    // Throws on a non-zero exit, which is the failure we want reported.
    execFileSync('node_modules/.bin/tsc', ['--noEmit'], { cwd: root, stdio: 'pipe' })
  }, 60_000)

  it('states the reason it was given', () => {
    const { getByTestId } = render(() => <EmptyPane reason="No agent has run today" />)
    expect(getByTestId('empty-pane').textContent).toContain('No agent has run today')
  })

  it('keeps two different reasons distinct', () => {
    const a = render(() => <EmptyPane reason="No agent has run today" />)
    const b = render(() => <EmptyPane reason="Nothing needs you" />)
    expect(a.getByTestId('empty-pane').textContent).not.toBe(b.getByTestId('empty-pane').textContent)
  })

  it('renders an action when one is offered', () => {
    const { getByText } = render(() => (
      <EmptyPane reason="No plan is in progress" action={<button>New plan</button>} />
    ))
    expect(getByText('New plan')).toBeTruthy()
  })
})
