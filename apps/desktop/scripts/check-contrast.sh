#!/usr/bin/env bash
# Every text colour clears WCAG AA against the ground it is actually painted on.
#
# The first version of this only compared tokens against the five grounds, which
# missed the two ways contrast actually broke: a <button> inheriting the UA's
# black because no rule set `color`, and text dimmed with `opacity`, which
# multiplies the contrast of whatever is under it and leaves nothing to measure.
# This reads the stylesheets and checks what they paint.
set -euo pipefail
cd "$(dirname "$0")/.."

node - <<'JS'
const fs = require('node:fs')

const FILES = [
  'src/styles/tokens.css',
  'src/styles/type.css',
  'src/styles/interaction.css',
  'src/ui/ui.css',
  'src/shell/shell.css',
  'src/screens/screens.css',
]
const css = Object.fromEntries(FILES.map((f) => [f, fs.readFileSync(f, 'utf8')]))
const all = Object.values(css).join('\n')
const strip = (s) => s.replace(/\/\*[\s\S]*?\*\//g, '')

// ---------- colour resolution ----------
const TOKENS = Object.fromEntries(
  [...strip(css['src/styles/tokens.css']).matchAll(/(--[a-z0-9-]+):\s*([^;]+);/g)].map((m) => [
    m[1],
    m[2].trim(),
  ]),
)
const hex = (h) =>
  h.length === 4
    ? [1, 2, 3].map((i) => parseInt(h[i] + h[i], 16)).concat(1)
    : [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)).concat(1)

/** -> [r,g,b,a], or null when the value is not a colour. */
function resolve(value, seen = 0) {
  if (!value || seen > 8) return null
  const v = value.trim()
  if (v === 'transparent') return [0, 0, 0, 0]
  if (v === 'inherit' || v === 'currentColor' || v === 'none') return null
  if (v.startsWith('#')) return hex(v)

  let m = v.match(/^var\((--[a-z0-9-]+)/)
  if (m) return resolve(TOKENS[m[1]], seen + 1)

  m = v.match(/^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*([\d.]+))?\s*\)/)
  if (m) return [+m[1], +m[2], +m[3], m[4] === undefined ? 1 : +m[4]]

  m = v.match(/^color-mix\(in srgb,\s*(.+?)\s+([\d.]+)%,\s*(.+?)\s*\)$/)
  if (m) {
    const a = resolve(m[1], seen + 1)
    const b = resolve(m[3], seen + 1)
    if (!a || !b) return null
    const p = +m[2] / 100
    // color-mix with transparent yields a translucent colour, not a blend to black.
    if (b[3] === 0) return [a[0], a[1], a[2], a[3] * p]
    return [0, 1, 2].map((i) => a[i] * p + b[i] * (1 - p)).concat(a[3] * p + b[3] * (1 - p))
  }
  return null
}

const srgb = (c) => ((c /= 255) <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4))
const L = ([r, g, b]) => 0.2126 * srgb(r) + 0.7152 * srgb(g) + 0.0722 * srgb(b)
const ratio = (a, b) => {
  const [l1, l2] = [L(a), L(b)]
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05)
}
/** Composite a possibly-translucent colour over an opaque ground. */
const flatten = (c, ground) =>
  c[3] >= 1 ? c.slice(0, 3) : [0, 1, 2].map((i) => c[i] * c[3] + ground[i] * (1 - c[3]))

const GROUNDS = ['--bg', '--bg-deep', '--sf', '--sf2', '--sf3'].map((n) => [
  n,
  resolve(`var(${n})`).slice(0, 3),
])

// ---------- rules ----------
const rules = []
for (const [file, text] of Object.entries(css)) {
  const re = /([^{}]+)\{([^{}]*)\}/g
  let m
  while ((m = re.exec(strip(text)))) {
    const selector = m[1].trim()
    if (selector.startsWith('@')) continue
    const body = m[2]
    rules.push({
      file,
      selector,
      body,
      color: (body.match(/(?:^|[;\s])color:\s*([^;]+)/) || [])[1],
      background: (body.match(/(?:^|[;\s])background(?:-color)?:\s*([^;]+)/) || [])[1],
      opacity: (body.match(/(?:^|[;\s])opacity:\s*([\d.]+)/) || [])[1],
    })
  }
}

let fail = 0
const problem = (msg) => {
  console.log(msg)
  fail = 1
}

// ---------- 1. the UA-colour reset ----------
if (!/button,[\s\S]{0,120}color: inherit/.test(strip(css['src/styles/interaction.css']))) {
  problem(
    'no rule makes <button> inherit its colour — the UA paints it black on a dark app',
  )
}

// ---------- 2. opacity is not a dimmer for text ----------
const OPACITY_ALLOWED = /disabled|\.pulse|\.blink|@keyframes/
for (const r of rules) {
  if (!r.opacity || +r.opacity >= 1) continue
  if (OPACITY_ALLOWED.test(r.selector) || OPACITY_ALLOWED.test(r.body)) continue
  problem(
    `${r.file}: ${r.selector} dims with opacity ${r.opacity} — dim with a colour token instead`,
  )
}

// ---------- 3. every colour, against the ground it is painted on ----------
const AA = 4.5
const worstSeen = []
for (const r of rules) {
  const fg = resolve(r.color)
  if (!fg) continue

  // The ground: this rule's own background, else the nearest ancestor selector
  // that sets one, else every app ground.
  let grounds = GROUNDS
  const own = resolve(r.background)
  if (own && own[3] > 0) {
    grounds = GROUNDS.map(([n, g]) => [`${n} + ${r.selector}`, flatten(own, g)])
  } else {
    const parentSel = r.selector.includes(' ') ? r.selector.split(' ')[0] : null
    const parent = parentSel && rules.find((x) => x.selector === parentSel && x.background)
    const pbg = parent && resolve(parent.background)
    if (pbg && pbg[3] > 0) grounds = GROUNDS.map(([n, g]) => [`${n} + ${parentSel}`, flatten(pbg, g)])
  }

  for (const [gname, g] of grounds) {
    const rr = ratio(flatten(fg, g), g)
    worstSeen.push([rr, `${r.selector} on ${gname}`])
    if (rr < AA) problem(`${r.file}: ${r.selector} on ${gname} — ${rr.toFixed(2)}:1, below AA`)
  }
}

if (!fail) {
  worstSeen.sort((a, b) => a[0] - b[0])
  const [rr, where] = worstSeen[0]
  console.log(
    `check-contrast: ${worstSeen.length} colour/ground pairings, all >= ${AA}; ` +
      `worst is ${rr.toFixed(2)}:1 (${where})`,
  )
}
process.exit(fail)
JS
