import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ICON_MAX, ICON_MIN, Icon } from '../src/ui/Icon'
import { mountIconSprite } from '../src/ui/sprite'

describe('icon-renders', () => {
  it('points at the regular symbol by default', () => {
    const { container } = render(() => <Icon name="tray" />)
    expect(container.querySelector('use')?.getAttribute('href')).toBe('#ph-tray')
  })

  it('points at the fill symbol when asked', () => {
    const { container } = render(() => <Icon name="seal-check" weight="fill" />)
    expect(container.querySelector('use')?.getAttribute('href')).toBe('#ph-seal-check-fill')
  })

  it('renders at the size given', () => {
    const { container } = render(() => <Icon name="code" size={19} />)
    const svg = container.querySelector('svg')!
    expect(svg.getAttribute('width')).toBe('19')
    expect(svg.getAttribute('height')).toBe('19')
  })

  it('clamps to the 9-19px range the design uses', () => {
    const small = render(() => <Icon name="code" size={4} />)
    expect(small.container.querySelector('svg')!.getAttribute('width')).toBe(String(ICON_MIN))
    const big = render(() => <Icon name="code" size={64} />)
    expect(big.container.querySelector('svg')!.getAttribute('width')).toBe(String(ICON_MAX))
  })

  it('inherits color rather than carrying its own', () => {
    const { container } = render(() => <Icon name="code" />)
    expect(container.querySelector('svg')!.getAttribute('fill')).toBe('currentColor')
  })

  it('is hidden from readers unless it is given a label', () => {
    const mute = render(() => <Icon name="code" />)
    expect(mute.container.querySelector('svg')!.getAttribute('aria-hidden')).toBe('true')
    const spoken = render(() => <Icon name="code" label="Develop" />)
    const svg = spoken.container.querySelector('svg')!
    expect(svg.getAttribute('aria-hidden')).toBe(null)
    expect(svg.getAttribute('aria-label')).toBe('Develop')
  })

  it('resolves against a sprite mounted into the document once', () => {
    mountIconSprite()
    mountIconSprite()
    expect(document.querySelectorAll('#ph-sprite').length).toBe(1)
    expect(document.querySelector('#ph-sprite symbol#ph-tray')).not.toBe(null)
  })
})
