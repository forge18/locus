import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { FixtureNotice } from '../../src/ui/FixtureNotice'

describe('fixtures/notice', () => {
  it('says the screen is fixture data, in the screen rather than in a changelog', () => {
    const { getByTestId } = render(() => (
      <FixtureNotice surface="Telemetry" command='invoke("telemetry_aggregates")' />
    ))
    expect(getByTestId('fixture-notice-surface').textContent).toContain('Telemetry')
    expect(getByTestId('fixture-notice-surface').textContent).toContain('no backend yet')
  })

  it('names the command that will replace it', () => {
    const { getByTestId } = render(() => (
      <FixtureNotice surface="Runs" command='invoke("runs_list")' />
    ))
    expect(getByTestId('fixture-notice-command').textContent).toBe('invoke("runs_list")')
    expect(getByTestId('fixture-notice-command').className).toContain('mono')
  })

  it('announces itself without stealing focus — it is a status, not an alert', () => {
    const { getByTestId } = render(() => <FixtureNotice surface="Wiki" command='invoke("wiki_pages")' />)
    expect(getByTestId('fixture-notice').getAttribute('role')).toBe('status')
  })

  it('requires both the surface and the command', () => {
    // @ts-expect-error — command is not optional
    const missing: Parameters<typeof FixtureNotice>[0] = { surface: 'Wiki' }
    void missing
  })
})
