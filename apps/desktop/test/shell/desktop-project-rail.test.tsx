import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { createNavStore } from '../../src/nav'
import { Shell } from '../../src/shell/Shell'

describe('shell/desktop-project-rail', () => {
  it('replaces the v1 chrome with the desktop title bar and selected-project rail', () => {
    const { getByRole, getByTestId, queryByTestId } = render(() => (
      <Shell nav={createNavStore()}>
        <p>screen body</p>
      </Shell>
    ))

    expect(getByTestId('app-titlebar')).toBeTruthy()
    expect(getByTestId('project-rail')).toBeTruthy()
    expect(getByTestId('selected-project-card').textContent).toContain('tapestry')
    expect(queryByTestId('project-filter')).toBeNull()
    expect(queryByTestId('tabbar')).toBeNull()

    getByTestId('running-pill').click()
    expect(getByRole('dialog', { name: 'Active sessions' })).toBeTruthy()
    expect(getByTestId('active-session-list').querySelectorAll('li')).not.toHaveLength(0)
  })
})
