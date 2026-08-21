import { splitProps } from 'solid-js'
import type { JSX } from 'solid-js'

/** Phosphor ships two weights for this app: outline, and solid for emphasis. */
export type IconWeight = 'regular' | 'fill'

/** The design draws icons between 9px (inline glyphs) and 19px (the rail). */
export const ICON_MIN = 9
export const ICON_MAX = 19

export interface IconProps extends JSX.SvgSVGAttributes<SVGSVGElement> {
  /** Phosphor icon name, e.g. `tray`, `git-branch`. */
  name: string
  weight?: IconWeight
  /** Pixels. Clamped to the 9-19px range the design uses. */
  size?: number
  /** Reader-facing name. Omit for an icon that only repeats adjacent text. */
  label?: string
}

const clamp = (n: number) => Math.min(ICON_MAX, Math.max(ICON_MIN, n))

export function Icon(props: IconProps) {
  const [own, rest] = splitProps(props, ['name', 'weight', 'size', 'label'])
  const size = () => clamp(own.size ?? 14)
  const id = () => `ph-${own.name}${own.weight === 'fill' ? '-fill' : ''}`

  return (
    <svg
      width={size()}
      height={size()}
      viewBox="0 0 256 256"
      fill="currentColor"
      role={own.label ? 'img' : 'presentation'}
      aria-hidden={own.label ? undefined : 'true'}
      aria-label={own.label}
      {...rest}
    >
      <use href={`#${id()}`} />
    </svg>
  )
}
