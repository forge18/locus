import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { SRC } from './css'

const sprite = readFileSync(resolve(SRC, 'assets/icons/phosphor.svg'), 'utf8')
const names = readFileSync(resolve(SRC, '../scripts/icons.txt'), 'utf8')
  .split('\n')
  .map((s) => s.trim())
  .filter(Boolean)

describe('icons-local', () => {
  it('carries a regular and a fill symbol for every named icon', () => {
    for (const n of names) {
      expect(sprite, `missing ph-${n}`).toContain(`id="ph-${n}"`)
      expect(sprite, `missing ph-${n}-fill`).toContain(`id="ph-${n}-fill"`)
    }
  })

  it('has exactly two symbols per icon and no strays', () => {
    const ids = [...sprite.matchAll(/id="([^"]+)"/g)].map((m) => m[1])
    expect(ids.length).toBe(names.length * 2)
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('references no external host', () => {
    expect(sprite).not.toMatch(/https?:\/\/(?!www\.w3\.org)/)
  })

  it('is sorted, so the build is byte-deterministic', () => {
    const ids = [...sprite.matchAll(/id="ph-([^"]+)"/g)]
      .map((m) => m[1].replace(/-fill$/, ''))
      .filter((_, i) => i % 2 === 0)
    expect(ids).toEqual([...names].sort())
  })
})
