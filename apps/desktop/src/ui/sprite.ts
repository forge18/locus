// The Phosphor sprite is inlined into the document once, rather than fetched by
// `<use href="file.svg#id">`. A cross-document `use` reference is a network fetch
// in every engine we ship on, and this app has to draw with the network off.
import sprite from '../assets/icons/phosphor.svg?raw'

const MOUNT_ID = 'ph-sprite'

export function mountIconSprite(doc: Document = document): void {
  if (doc.getElementById(MOUNT_ID)) return
  const host = doc.createElement('div')
  host.id = MOUNT_ID
  host.setAttribute('aria-hidden', 'true')
  host.style.display = 'none'
  host.innerHTML = sprite
  doc.body.appendChild(host)
}
