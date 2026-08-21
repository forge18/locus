import { Toast as KToast, toaster } from '@kobalte/core/toast'
import { Portal } from 'solid-js/web'

/**
 * Toasts are for something the reader is *not* looking at — a background run that
 * finished, a push that landed. Anything on the surface in front of them uses
 * `InlineError` instead, which is why this takes no "error on this pane" path.
 */
export function ToastRegion() {
  return (
    <Portal>
      <KToast.Region>
        <KToast.List class="toast-region" data-testid="toast-region" />
      </KToast.Region>
    </Portal>
  )
}

export interface NotifyOptions {
  title: string
  description?: string
  type?: 'default' | 'error'
}

export function notify(options: NotifyOptions): number {
  return toaster.show((props) => (
    <KToast class="toast" toastId={props.toastId} data-type={options.type ?? 'default'}>
      <KToast.Title class="em">{options.title}</KToast.Title>
      {options.description ? (
        <KToast.Description class="t-meta">{options.description}</KToast.Description>
      ) : null}
    </KToast>
  ))
}
