import { createSignal, For } from 'solid-js'
import { INSTALLED_THEMES, persistTheme, savedTheme, type ThemeId } from '../../styles/theme'

const label: Record<ThemeId, string> = { dark: 'Dark', light: 'Light' }

/** Install-wide Appearance preference. It stores only the stable theme identifier. */
export function AppearanceSelector() {
  const [theme, setTheme] = createSignal(savedTheme(window.localStorage))

  const select = (next: ThemeId) => {
    setTheme(persistTheme(window.localStorage, document.documentElement, next))
  }

  return (
    <section class="settings-section" data-testid="appearance-theme">
      <h3>Appearance</h3>
      <div class="settings-row">
        <div class="settings-copy">
          <span>Theme</span>
          <p>Choose the value set used by this install.</p>
        </div>
        <div class="settings-control" role="group" aria-label="Theme">
          <For each={INSTALLED_THEMES}>
            {(id) => (
              <button
                type="button"
                class="settings-select"
                classList={{ 'settings-theme-selected': theme() === id }}
                aria-pressed={theme() === id}
                onClick={() => select(id)}
              >
                {label[id]}
              </button>
            )}
          </For>
        </div>
      </div>
    </section>
  )
}
