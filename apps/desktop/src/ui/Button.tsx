import { splitProps } from 'solid-js'
import { Button as KButton } from '@kobalte/core/button'
import type { ComponentProps } from 'solid-js'

/** Primary is the accent as a line. Nothing in this app is filled with accent. */
export type ButtonVariant = 'primary' | 'secondary' | 'ghost'

export interface ButtonProps extends ComponentProps<'button'> {
  variant?: ButtonVariant
  /** Fill the width of the container it sits in. */
  block?: boolean
}

export function Button(props: ButtonProps) {
  const [own, rest] = splitProps(props, ['variant', 'block', 'class'])
  const classes = () =>
    ['btn', `btn-${own.variant ?? 'secondary'}`, own.block ? 'btn-block' : '', own.class ?? '']
      .filter(Boolean)
      .join(' ')

  return <KButton class={classes()} {...rest} />
}
