import { For, Show } from 'solid-js'
import { ContextMenu as KContextMenu } from '@kobalte/core/context-menu'
import type { JSX } from 'solid-js'

export interface MenuAction {
  label: string
  onSelect: () => void
  disabled?: boolean
}

export interface ContextMenuProps {
  actions: MenuAction[]
  /** Optional heading above the actions, e.g. the object the menu acts on. */
  heading?: string
  children: JSX.Element
}

export function ContextMenu(props: ContextMenuProps) {
  return (
    <KContextMenu>
      <KContextMenu.Trigger as="div" data-testid="context-menu-trigger">
        {props.children}
      </KContextMenu.Trigger>
      <KContextMenu.Portal>
        <KContextMenu.Content class="menu" data-testid="context-menu">
          <Show when={props.heading}>
            <div class="menu-section">{props.heading}</div>
          </Show>
          <For each={props.actions}>
            {(a) => (
              <KContextMenu.Item
                class="menu-item"
                disabled={a.disabled}
                onSelect={a.onSelect}
              >
                {a.label}
              </KContextMenu.Item>
            )}
          </For>
        </KContextMenu.Content>
      </KContextMenu.Portal>
    </KContextMenu>
  )
}
