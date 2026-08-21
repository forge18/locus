#!/usr/bin/env bash
# Every text colour clears WCAG AA (4.5:1) against every ground it can sit on.
#
# The palette is a handful of colours over five surfaces, so this is a complete
# enumeration rather than a sample. It is a script and not a test because it is
# arithmetic over tokens.css, and it should fail the moment a token moves.
set -euo pipefail
cd "$(dirname "$0")/.."

node - <<'JS'
const fs = require('node:fs')
const css = fs.readFileSync('src/styles/tokens.css', 'utf8')

const token = (name) => css.match(new RegExp(`${name}:\\s*([^;]+);`))[1].trim()
const hex = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16))
const rgba = (v) => {
  const m = v.match(/rgba\((\d+),(\d+),(\d+),([\d.]+)\)/)
  return m ? { rgb: [+m[1], +m[2], +m[3]], a: +m[4] } : { rgb: hex(v), a: 1 }
}

const srgb = (c) => (c /= 255) <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
const L = ([r, g, b]) => 0.2126 * srgb(r) + 0.7152 * srgb(g) + 0.0722 * srgb(b)
const ratio = (a, b) => {
  const [l1, l2] = [L(a), L(b)]
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05)
}
const over = (fg, a, bg) => fg.map((c, i) => c * a + bg[i] * (1 - a))

// Every ground text is ever set on.
const GROUNDS = ['--bg', '--bg-deep', '--sf', '--sf2', '--sf3'].map((n) => [n, hex(token(n))])

// Text colours, and which grounds each actually appears on. --ok and --bad are
// text on the app grounds and on cards; neither is ever text on a --sf3 chip.
const TEXT = [
  ['--tx', GROUNDS],
  ['--mu', GROUNDS],
  ['--mu2', GROUNDS],
  ['--ac', GROUNDS],
  ['--code-keyword', GROUNDS],
  ['--ok', GROUNDS.filter(([n]) => n !== '--sf3')],
  ['--bad', GROUNDS.filter(([n]) => n !== '--sf3')],
]

const AA = 4.5
let fail = 0
for (const [name, grounds] of TEXT) {
  const { rgb, a } = rgba(token(name))
  for (const [gname, g] of grounds) {
    const r = ratio(over(rgb, a, g), g)
    if (r < AA) {
      console.log(`${name} on ${gname}: ${r.toFixed(2)}:1 — below AA (${AA})`)
      fail = 1
    }
  }
}

// Hairlines are decoration; 3:1 is the bar for a non-text boundary.
for (const name of ['--line', '--line2']) {
  const { rgb, a } = rgba(token(name))
  const worst = Math.min(...GROUNDS.map(([, g]) => ratio(over(rgb, a, g), g)))
  if (worst < 1.2) {
    console.log(`${name}: ${worst.toFixed(2)}:1 — invisible against its own ground`)
    fail = 1
  }
}

if (!fail) {
  const worst = Math.min(
    ...TEXT.flatMap(([name, grounds]) => {
      const { rgb, a } = rgba(token(name))
      return grounds.map(([, g]) => ratio(over(rgb, a, g), g))
    }),
  )
  console.log(`check-contrast: every text token clears AA; worst pairing is ${worst.toFixed(2)}:1`)
}
process.exit(fail)
JS
