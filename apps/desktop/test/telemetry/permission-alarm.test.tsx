import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { useActionRows } from '../../src/data/telemetry'
import { read, rules } from '../css'

const mount = () => render(() => <TelemetryView />)

describe('telemetry/permission-alarm', () => {
  it('renders an alarm callout beside the permission_request row', () => {
    const { getByTestId } = mount()
    expect(getByTestId('permission-alarm')).toBeTruthy()
    expect(getByTestId('action-permission_request').getAttribute('data-alarm')).toBe('true')
  })

  it('says it is a misconfiguration alarm, not a metric', () => {
    const { getByTestId } = mount()
    expect(getByTestId('permission-alarm').textContent).toContain(
      'is a misconfiguration alarm, not a metric',
    )
  })

  it('says what actually happened — a gate was left on and the runs hung', () => {
    const { getByTestId } = mount()
    const text = getByTestId('permission-alarm').textContent!
    expect(text).toContain('its own gate on')
    expect(text).toContain('hung')
  })

  it('is visually distinct from a count row: red ring, red tint, its own glyph', () => {
    const { getByTestId } = mount()
    expect(getByTestId('permission-alarm').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-warning-octagon-fill',
    )
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.alarm-callout',
    )!.body
    expect(body).toContain('inset 0 0 0 1px var(--status-danger)')
    expect(body).toContain('color-mix(in srgb, var(--status-danger) 8%, var(--surface-raised))')
  })

  it('is the only verb carrying one', () => {
    const alarms = useActionRows().filter((a) => a.alarm !== null)
    expect(alarms.length).toBe(1)
    expect(alarms[0].verb).toBe('permission_request')
  })

  it('stays in the vocabulary precisely so it can fire', () => {
    expect(useActionRows().find((a) => a.verb === 'permission_request')!.count).toBeGreaterThan(0)
  })
})
