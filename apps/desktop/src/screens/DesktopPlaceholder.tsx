import type { View } from '../nav'

export function DesktopPlaceholder(props: { view: View }) {
  return <section class="screen-placeholder" data-testid={`screen-${props.view}`}><h1>{props.view}</h1></section>
}
