import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Rail } from '../../src/shell/Rail'
import { VIEWS, categoryOf } from '../../src/nav'
import type { View } from '../../src/nav'

const lit = (view: View) => {
  const { getByTestId, unmount } = render(() => (
    <Rail view={view} onNavigate={() => {}} inboxCount={0} />
  ))
  const el = getByTestId('rail').querySelector('[aria-current="true"]')!
  const category = el.getAttribute('data-category')
  unmount()
  return category
}

describe('shell/rail-lights-by-category', () => {
  it('lights the owning category for every one of the fourteen views', () => {
    for (const view of VIEWS) {
      expect(lit(view), view).toBe(categoryOf(view))
    }
  })

  it('keeps Workshop lit on the agent-definitions drill-down', () => {
    expect(lit('agents')).toBe('workshop')
    expect(lit('extensions')).toBe('workshop')
  })

  it('never adds an eighth item for a drill-down', () => {
    const { getByTestId } = render(() => (
      <Rail view="agents" onNavigate={() => {}} inboxCount={0} />
    ))
    expect(getByTestId('rail').querySelectorAll('.rail-item').length).toBe(7)
  })

  it('groups the two dashboard views under one entry', () => {
    expect(lit('inbox')).toBe('dashboard')
    expect(lit('status')).toBe('dashboard')
  })
})
