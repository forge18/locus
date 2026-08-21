import { For } from 'solid-js'
import { Tabs as KTabs } from '@kobalte/core/tabs'

export interface TabItem {
  value: string
  label: string
}

export interface TabsProps {
  items: TabItem[]
  value: string
  onChange: (value: string) => void
  label: string
}

/** The tab bar only. What a tab shows is a view, resolved by navigation. */
export function Tabs(props: TabsProps) {
  return (
    <KTabs value={props.value} onChange={props.onChange} data-testid="tabs">
      <KTabs.List class="tabs-list" aria-label={props.label}>
        <For each={props.items}>
          {(t) => (
            <KTabs.Trigger class="tab" value={t.value}>
              {t.label}
            </KTabs.Trigger>
          )}
        </For>
      </KTabs.List>
    </KTabs>
  )
}
