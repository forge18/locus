import { splitProps } from 'solid-js'
import type { ComponentProps } from 'solid-js'

export interface InputProps extends ComponentProps<'input'> {
  /** Set for locators, branches, paths, ids — anything a machine named. */
  mono?: boolean
}

export function Input(props: InputProps) {
  const [own, rest] = splitProps(props, ['mono', 'class'])
  return (
    <input
      class={['input', own.mono ? 'mono' : '', own.class ?? ''].filter(Boolean).join(' ')}
      {...rest}
    />
  )
}

export interface TextareaProps extends ComponentProps<'textarea'> {
  mono?: boolean
}

export function Textarea(props: TextareaProps) {
  const [own, rest] = splitProps(props, ['mono', 'class'])
  return (
    <textarea
      class={['input', own.mono ? 'mono' : '', own.class ?? ''].filter(Boolean).join(' ')}
      {...rest}
    />
  )
}
