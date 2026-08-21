import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Tag } from '../../src/ui/Tag'
import { read, rules } from '../css'

const css = read('ui/ui.css')
const rule = (sel: string) => rules(css).find((r) => r.selector === sel)
const tag = (el: HTMLElement) => el.querySelector('span')!

describe('ui/tag', () => {
  it('defaults to the accent chip', () => {
    const { container } = render(() => <Tag>gate</Tag>)
    expect(tag(container).className).toBe('tag')
  })

  it('carries the outline and neutral variants', () => {
    const outline = render(() => <Tag variant="outline">+ pgvector</Tag>)
    expect(tag(outline.container).className).toContain('tag-outline')
    const neutral = render(() => <Tag variant="neutral">tapestry</Tag>)
    expect(tag(neutral.container).className).toContain('tag-neutral')
  })

  it('sets its content in mono — a tag names a tool, a repo or a flag', () => {
    expect(rule('.tag')!.body).toContain('font-family: var(--fm)')
  })

  it('gives neutral a floor width so a column of them lines up', () => {
    const neutral = rule('.tag-neutral')!.body
    expect(neutral).toMatch(/min-width:\s*\d+px/)
    expect(neutral).toContain('justify-content: center')
    // and the other variants do not, so they hug their text
    expect(rule('.tag-outline')!.body).not.toContain('min-width')
  })

  it('draws outline as a line and accent as a ground, both from --ac', () => {
    expect(rule('.tag-outline')!.body).toContain('border-color: var(--ac)')
    expect(rule('.tag')!.body).toContain('background: var(--ac-deep)')
  })
})
