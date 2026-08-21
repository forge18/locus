import { splitProps } from 'solid-js'
import type { ComponentProps } from 'solid-js'

export interface CardProps extends ComponentProps<'div'> {
  /** Selected is the raised surface plus the accent inset ring. Never an outer glow. */
  selected?: boolean
  /** Responds to the pointer. Set it only when the card actually does something. */
  interactive?: boolean
}

export function Card(props: CardProps) {
  const [own, rest] = splitProps(props, ['selected', 'interactive', 'class'])
  const classes = () =>
    [
      'card',
      own.interactive ? 'card-interactive' : '',
      own.selected ? 'card-selected' : '',
      own.class ?? '',
    ]
      .filter(Boolean)
      .join(' ')

  return <div class={classes()} aria-selected={own.selected ? 'true' : undefined} {...rest} />
}
