import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { Shell } from '../../src/shell/Shell'
import { InboxView } from '../../src/screens/inbox/InboxView'
import { StatusView } from '../../src/screens/status/StatusView'
import { createNavStore } from '../../src/nav'
import { SRC, read, rules } from '../css'

/**
 * A structural conformance check against screenshots/01-inbox.png and
 * 02-status.png — NOT a pixel diff. jsdom has no layout engine, so what is
 * asserted here is everything the screenshots encode that survives without one:
 * which elements exist, in what order, at which declared sizes, carrying which
 * copy. A real pixel comparison needs a browser and belongs with a packaged build.
 */
const SHOTS = resolve(SRC, '../../../docs/design_handoff_locus_desktop_ui/screenshots')
const rule = (file: string, sel: string) => rules(read(file)).find((r) => r.selector === sel)!

const mountInbox = () => {
  const nav = createNavStore({ view: 'inbox' })
  return render(() => (
    <Shell nav={nav}>
      <InboxView nav={nav} />
    </Shell>
  ))
}

const mountStatus = () => {
  const nav = createNavStore({ view: 'status' })
  return render(() => (
    <Shell nav={nav}>
      <StatusView />
    </Shell>
  ))
}

describe('visual: dashboard', () => {
  it('has both reference screenshots to conform to', () => {
    expect(existsSync(resolve(SHOTS, '01-inbox.png'))).toBe(true)
    expect(existsSync(resolve(SHOTS, '02-status.png'))).toBe(true)
  })

  it('inbox: four bands around a two-pane body, each holding its band height', () => {
    const { getByTestId } = mountInbox()
    for (const band of ['titlebar', 'rail', 'tabbar', 'strip']) {
      expect(getByTestId(band), band).toBeTruthy()
    }
    expect(rule('shell/shell.css', '.titlebar').body).toContain('height: 42px')
    expect(rule('shell/shell.css', '.rail').body).toContain('clamp(68px, 6vw, 92px)')
    expect(rule('shell/shell.css', '.tabbar').body).toContain('height: 40px')
    expect(rule('shell/shell.css', '.strip').body).toContain('height: 54px')
    expect(rule('screens/screens.css', '.inbox-list').body).toContain('clamp(300px, 30%, 440px)')
  })

  it('inbox: the Inbox tab is lit and the Inbox rail item is current', () => {
    const { getByTestId } = mountInbox()
    expect(getByTestId('tab-inbox').getAttribute('data-selected')).toBe('')
    expect(getByTestId('rail-dashboard').getAttribute('aria-current')).toBe('true')
    expect(getByTestId('tabbar-category').textContent).toBe('Inbox')
  })

  it('inbox: three cards over three resolved rows, in that order', () => {
    const { getByTestId } = mountInbox()
    expect(getByTestId('inbox-list').querySelectorAll('.inbox-card').length).toBe(3)
    expect(getByTestId('inbox-resolved').querySelectorAll('.inbox-resolved-row').length).toBe(3)
  })

  it('inbox: the copy the screenshot shows, verbatim', () => {
    const { getByTestId, container } = mountInbox()
    expect(getByTestId('needs-you-note').textContent).toContain('silence is the default')
    expect(getByTestId('inbox-comment-caption').textContent).toBe(
      'Comment steers the agent that made it',
    )
    expect(getByTestId('inbox-approve').textContent).toBe('Approve & release the loop')
    expect(getByTestId('inbox-send-back').textContent).toBe('Send back with comment')
    expect(getByTestId('inbox-footer-note').textContent).toBe(
      'Resolves here · the work opens where the work lives',
    )
    expect(container.textContent).toContain('sorted by needs-attention, then activity')
  })

  it('status: six metric cards, then the two panels, then the table', () => {
    const { getByTestId } = mountStatus()
    const screen = getByTestId('screen')
    const order = [...screen.querySelectorAll('[data-testid]')]
      .map((el) => el.getAttribute('data-testid'))
      .filter((id) => ['status-metrics', 'runs-by-hour', 'wants-attention', 'project-table'].includes(id!))
    expect(order).toEqual(['status-metrics', 'runs-by-hour', 'wants-attention', 'project-table'])
    expect(getByTestId('status-metrics').querySelectorAll('.metric-card').length).toBe(6)
  })

  it('status: twelve bars, three attention rows, four project rows', () => {
    const { getByTestId } = mountStatus()
    expect(getByTestId('hours').querySelectorAll('.hour-bar').length).toBe(12)
    expect(getByTestId('wants-attention').querySelectorAll('.attention-row').length).toBe(3)
    expect(getByTestId('project-table').querySelectorAll('tbody tr').length).toBe(4)
  })

  it('status: the legend the screenshot carries beside the chart title', () => {
    const { getByTestId } = mountStatus()
    expect(getByTestId('hours-legend').textContent).toContain('passed')
    expect(getByTestId('hours-legend').textContent).toContain('failed')
    expect(getByTestId('hours-legend').textContent).toContain('aborted')
  })

  it('status: the Status tab is lit while the rail still reads Inbox', () => {
    const { getByTestId } = mountStatus()
    expect(getByTestId('tab-status').getAttribute('data-selected')).toBe('')
    expect(getByTestId('rail-dashboard').getAttribute('aria-current')).toBe('true')
  })

  it('both: every color on screen resolves from a token', () => {
    // The screenshots are the palette; this is what keeps the app on it.
    for (const file of ['screens/screens.css', 'shell/shell.css', 'ui/ui.css']) {
      expect(read(file), file).not.toMatch(/#[0-9a-fA-F]{6}\b/)
    }
  })
})
