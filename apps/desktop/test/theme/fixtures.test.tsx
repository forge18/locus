import { cleanup, render } from '@solidjs/testing-library'
import { afterEach, describe, expect, it } from 'vitest'
import { V2DashboardView, V2InboxView } from '../../src/screens/v2-dashboard'
import { INSTALLED_THEMES } from '../../src/styles/theme'

afterEach(cleanup)

describe('theme/fixtures', () => {
  for (const theme of INSTALLED_THEMES) {
    it(`renders the v2 fixture inventory in ${theme}`, () => {
      const inbox = render(() => <div data-theme={theme}><V2InboxView /></div>)
      expect(inbox.getByTestId('v2-inbox').closest(`[data-theme="${theme}"]`)).toBeTruthy()
      inbox.unmount()

      const dashboard = render(() => <div data-theme={theme}><V2DashboardView /></div>)
      expect(dashboard.getByTestId('v2-dashboard').closest(`[data-theme="${theme}"]`)).toBeTruthy()
    })
  }
})
