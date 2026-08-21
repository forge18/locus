import { Dialog } from '@kobalte/core/dialog'
import type { JSX } from 'solid-js'
import { Button } from './Button'

export interface SheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  children: JSX.Element
  footer?: JSX.Element
}

/**
 * Detail opens in place, over the category it belongs to. It is never a second
 * window — a window is what a *detached pane* gets, and conflating the two loses
 * the reader's place in the category they were working in.
 *
 * The overlay and content mount into the app root rather than <body>, so the sheet
 * is bounded by the window chrome instead of covering it.
 */
export function Sheet(props: SheetProps) {
  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange} modal>
      <Dialog.Portal mount={document.getElementById('root') ?? undefined}>
        <Dialog.Overlay class="overlay" data-testid="sheet-overlay" />
        <Dialog.Content class="sheet" data-testid="sheet">
          <div class="sheet-head">
            <Dialog.Title class="t-title">{props.title}</Dialog.Title>
            <Dialog.CloseButton
              as={Button}
              variant="ghost"
              aria-label="Close"
              style={{ 'margin-left': 'auto' }}
            >
              Close
            </Dialog.CloseButton>
          </div>
          <div class="sheet-body">{props.children}</div>
          {props.footer}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  )
}
