import { describe, expect, it } from 'vitest'
import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { SRC, allCss, read, rules, stripComments } from './css'

const FACES = [
  'inter-latin-400-normal.woff2',
  'inter-latin-500-normal.woff2',
  'jetbrains-mono-latin-400-normal.woff2',
  'jetbrains-mono-latin-500-normal.woff2',
]

describe('fonts-local', () => {
  const css = read('assets/fonts/fonts.css')

  it('vendors every woff2 face into src/assets/fonts', () => {
    for (const f of FACES) {
      expect(existsSync(resolve(SRC, 'assets/fonts', f)), `missing ${f}`).toBe(true)
    }
  })

  it('declares Inter at 400 and 500, and nothing heavier', () => {
    const inter = rules(css).filter((r) => r.selector === '@font-face' && r.body.includes("'Inter'"))
    expect(inter.map((r) => r.body.match(/font-weight:\s*(\d+)/)?.[1]).sort()).toEqual(['400', '500'])
  })

  it('declares JetBrains Mono at 400 and 500', () => {
    const mono = rules(css).filter(
      (r) => r.selector === '@font-face' && r.body.includes("'JetBrains Mono'"),
    )
    expect(mono.map((r) => r.body.match(/font-weight:\s*(\d+)/)?.[1]).sort()).toEqual(['400', '500'])
  })

  it('sources every face from a relative path, never a host', () => {
    const srcs = stripComments(css).match(/url\(([^)]+)\)/g) ?? []
    expect(srcs.length).toBe(FACES.length)
    for (const s of srcs) expect(s).toMatch(/url\('\.\//)
  })

  it('names the vendored families in the font tokens', () => {
    const tokens = read('styles/tokens.css')
    expect(tokens).toContain('--fm: "JetBrains Mono"')
    expect(tokens).toContain('--fs: "Inter"')
  })

  it('leaves no @import of a remote stylesheet anywhere', () => {
    for (const [file, contents] of allCss()) {
      const imports = stripComments(contents).match(/@import\s+[^;]+;/g) ?? []
      for (const imp of imports) {
        expect(imp, `${file}: ${imp}`).not.toMatch(/https?:|\/\//)
      }
    }
  })
})
