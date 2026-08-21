// Helpers for asserting on the stylesheets themselves. The token system is a
// static contract — reading the CSS is the honest way to check it, and it does
// not depend on jsdom implementing the cascade.
import { readFileSync, readdirSync } from 'node:fs'
import { join, resolve } from 'node:path'

export const SRC = resolve(__dirname, '../src')

export function read(rel: string): string {
  return readFileSync(resolve(SRC, rel), 'utf8')
}

/** Every .css file under src/, as [relative path, contents]. */
export function allCss(): Array<[string, string]> {
  const out: Array<[string, string]> = []
  const walk = (dir: string, prefix: string) => {
    for (const e of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const p = join(dir, e.name)
      const rel = prefix ? `${prefix}/${e.name}` : e.name
      if (e.isDirectory()) walk(p, rel)
      else if (e.name.endsWith('.css')) out.push([rel, readFileSync(p, 'utf8')])
    }
  }
  walk(SRC, '')
  return out
}

/** Every source file that can carry a style: .css, .ts, .tsx. */
export function allSource(): Array<[string, string]> {
  const out: Array<[string, string]> = []
  const walk = (dir: string, prefix: string) => {
    for (const e of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const p = join(dir, e.name)
      const rel = prefix ? `${prefix}/${e.name}` : e.name
      if (e.isDirectory()) walk(p, rel)
      else if (/\.(css|ts|tsx)$/.test(e.name)) out.push([rel, readFileSync(p, 'utf8')])
    }
  }
  walk(SRC, '')
  return out
}

/** Strip /* … *\/ comments so an example in prose is not read as a rule. */
export function stripComments(css: string): string {
  return css.replace(/\/\*[\s\S]*?\*\//g, '')
}

export interface Rule {
  selector: string
  body: string
}

/** Flat list of `selector { body }` pairs. Good enough for a hand-written stylesheet. */
export function rules(css: string): Rule[] {
  const out: Rule[] = []
  const re = /([^{}]+)\{([^{}]*)\}/g
  let m: RegExpExecArray | null
  while ((m = re.exec(stripComments(css)))) {
    const selector = m[1].trim()
    // @font-face carries declarations; other at-rule preludes do not.
    if (selector.startsWith('@') && !selector.endsWith('@font-face')) continue
    out.push({ selector, body: m[2].trim() })
  }
  return out
}

/** All values assigned to a property, across every rule in the CSS. */
export function declarations(css: string, prop: string): string[] {
  const out: string[] = []
  const re = new RegExp(`(?:^|[;{\\s])${prop}\\s*:\\s*([^;}]+)`, 'g')
  let m: RegExpExecArray | null
  while ((m = re.exec(stripComments(css)))) out.push(m[1].trim())
  return out
}
