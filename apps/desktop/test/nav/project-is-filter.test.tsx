import { describe, expect, it } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { Shell } from '../../src/shell/Shell'
import { createNavStore } from '../../src/nav'

const mount = () => {
  document.body.innerHTML = ''
  const root = document.createElement('div')
  root.id = 'root'
  document.body.appendChild(root)
  const nav = createNavStore({ view: 'board' })
  const r = render(
    () => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ),
    { container: root },
  )
  return { nav, ...r }
}

describe('nav/project-is-filter', () => {
  it('defaults to all projects', () => {
    const { getByTestId } = mount()
    expect(getByTestId('project-filter-label').textContent).toBe('All projects')
  })

  it('does not change the view when the filter is opened', () => {
    const { nav, getByTestId } = mount()
    getByTestId('project-filter').click()
    expect(nav.view()).toBe('board')
  })

  it('does not change the view when a project is chosen', async () => {
    const { nav, getByTestId } = mount()
    const before = { view: nav.view(), locator: nav.locator(), history: nav.history().length }

    getByTestId('context-menu-trigger').dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true }),
    )
    await waitFor(() => expect(document.querySelector('.menu')).not.toBe(null))
    const weaver = [...document.querySelectorAll('.menu-item')].find(
      (el) => el.textContent === 'weaver',
    ) as HTMLElement
    weaver.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }))

    await waitFor(() => expect(getByTestId('project-filter-label').textContent).toBe('weaver'))
    expect(nav.view()).toBe(before.view)
    expect(nav.locator()).toBe(before.locator)
    expect(nav.history().length).toBe(before.history)
  })

  it('leaves the rail alone — filtering is not leaving', async () => {
    const { getByTestId } = mount()
    getByTestId('context-menu-trigger').dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true }),
    )
    await waitFor(() => expect(document.querySelector('.menu')).not.toBe(null))
    expect(getByTestId('rail-automate').getAttribute('aria-current')).toBe('true')
  })

  it('carries the project in the locator, so a filtered view is still addressable', () => {
    const nav = createNavStore({ project: 'weaver', view: 'board' })
    expect(nav.locator()).toBe('locus://weaver/board')
  })
})
