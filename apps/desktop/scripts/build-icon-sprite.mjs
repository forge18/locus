// Builds src/assets/icons/phosphor.svg — one <symbol> per icon, regular and fill,
// for the set named in scripts/icons.txt (the icons the design actually uses).
// Vendoring the whole 1500-icon library would cost ~4MB for 79 glyphs of value.
//
// Symbol ids: `ph-<name>` for regular, `ph-<name>-fill` for fill.
// Output is byte-deterministic: sorted names, no timestamps.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const core = resolve(root, 'node_modules/@phosphor-icons/core/assets')
const names = readFileSync(resolve(root, 'scripts/icons.txt'), 'utf8')
  .split('\n')
  .map((s) => s.trim())
  .filter(Boolean)
  .sort()

// Phosphor ships each icon as a standalone <svg viewBox="0 0 256 256">…</svg>.
// Keep the body, drop the wrapper, and re-wrap as a <symbol>.
const body = (file) => {
  const svg = readFileSync(file, 'utf8')
  const m = svg.match(/<svg[^>]*>([\s\S]*)<\/svg>/)
  if (!m) throw new Error(`unparseable phosphor svg: ${file}`)
  return m[1].trim()
}

const symbols = []
for (const name of names) {
  symbols.push(
    `<symbol id="ph-${name}" viewBox="0 0 256 256">${body(`${core}/regular/${name}.svg`)}</symbol>`,
  )
  symbols.push(
    `<symbol id="ph-${name}-fill" viewBox="0 0 256 256">${body(`${core}/fill/${name}-fill.svg`)}</symbol>`,
  )
}

const out = resolve(root, 'src/assets/icons')
mkdirSync(out, { recursive: true })
writeFileSync(
  resolve(out, 'phosphor.svg'),
  `<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" style="display:none">\n${symbols.join('\n')}\n</svg>\n`,
)
writeFileSync(
  resolve(out, 'LICENSE-phosphor.txt'),
  readFileSync(resolve(root, 'node_modules/@phosphor-icons/core/LICENSE'), 'utf8'),
)
console.log(`sprite: ${names.length} icons, ${symbols.length} symbols`)
