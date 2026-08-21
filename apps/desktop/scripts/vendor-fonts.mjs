// Copies the woff2 faces we actually use out of the @fontsource packages and into
// src/assets/fonts/, so the app carries its own type and never asks a CDN for it.
// Re-run after bumping the @fontsource devDependencies.
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const out = resolve(root, 'src/assets/fonts')
mkdirSync(out, { recursive: true })

const faces = [
  ['@fontsource/inter', 'inter-latin-400-normal.woff2'],
  ['@fontsource/inter', 'inter-latin-500-normal.woff2'],
  ['@fontsource/jetbrains-mono', 'jetbrains-mono-latin-400-normal.woff2'],
  ['@fontsource/jetbrains-mono', 'jetbrains-mono-latin-500-normal.woff2'],
]

for (const [pkg, file] of faces) {
  copyFileSync(resolve(root, 'node_modules', pkg, 'files', file), resolve(out, file))
  console.log('vendored', file)
}

for (const pkg of ['@fontsource/inter', '@fontsource/jetbrains-mono']) {
  const name = pkg.split('/')[1]
  writeFileSync(
    resolve(out, `LICENSE-${name}.txt`),
    readFileSync(resolve(root, 'node_modules', pkg, 'LICENSE'), 'utf8'),
  )
  console.log('vendored', `LICENSE-${name}.txt`)
}
