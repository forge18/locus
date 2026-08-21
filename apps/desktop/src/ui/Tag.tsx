import { splitProps } from 'solid-js'
import type { ComponentProps } from 'solid-js'

/**
 * `accent` is the amber-ground chip, `outline` the accent hairline, `neutral` the
 * project chip — neutral carries a min-width so a column of them lines up.
 */
export type TagVariant = 'accent' | 'outline' | 'neutral'

export interface TagProps extends ComponentProps<'span'> {
  variant?: TagVariant
}

export function Tag(props: TagProps) {
  const [own, rest] = splitProps(props, ['variant', 'class'])
  const classes = () =>
    ['tag', own.variant && own.variant !== 'accent' ? `tag-${own.variant}` : '', own.class ?? '']
      .filter(Boolean)
      .join(' ')

  return <span class={classes()} {...rest} />
}
