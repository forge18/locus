import { beforeEach, describe, expect, it } from 'vitest'
import { applyTheme, persistTheme, savedTheme, THEME_STORAGE_KEY } from '../../src/styles/theme'

describe('theme/preference-fallback', () => {
  beforeEach(() => window.localStorage.clear())

  it('defaults missing and unknown persisted values to Dark', () => {
    expect(savedTheme(window.localStorage)).toBe('dark')
    window.localStorage.setItem(THEME_STORAGE_KEY, 'midnight')
    expect(savedTheme(window.localStorage)).toBe('dark')
  })

  it('persists only the stable identifier and applies it to the root', () => {
    const root = document.documentElement
    expect(persistTheme(window.localStorage, root, 'light')).toBe('light')
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe('light')
    expect(root.dataset.theme).toBe('light')
  })

  it('falls back safely when an invalid value is applied', () => {
    expect(applyTheme(document.documentElement, 'custom')).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
  })
})
