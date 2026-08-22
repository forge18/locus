import { render } from '@solidjs/testing-library'
import { afterEach, describe, expect, it } from 'vitest'
import { createNavStore } from '../../src/nav'
import { Shell } from '../../src/shell/Shell'
import { applyTheme, INSTALLED_THEMES } from '../../src/styles/theme'

afterEach(() => applyTheme(document.documentElement, 'dark'))

describe('shell/themes', () => {
  for (const theme of INSTALLED_THEMES) {
    it(`renders the shared shell in ${theme}`, () => {
      applyTheme(document.documentElement, theme)
      const { getByTestId } = render(() => <Shell nav={createNavStore()}><div /></Shell>)
      expect(getByTestId('window')).toBeTruthy()
      expect(document.documentElement.dataset.theme).toBe(theme)
    })
  }
})
