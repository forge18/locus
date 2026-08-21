#!/usr/bin/env node
// `pnpm test -- <filter>` forwards a literal `--` to the runner, which vitest reads
// as a filter of its own and quietly matches everything. Drop it, then hand the rest
// to vitest so the documented verify commands narrow to the test they name.
import { spawn } from 'node:child_process'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const args = process.argv.slice(2).filter((a) => a !== '--')
const bin = resolve(dirname(fileURLToPath(import.meta.url)), '../node_modules/.bin/vitest')

spawn(bin, ['run', ...args], { stdio: 'inherit' }).on('exit', (code) => process.exit(code ?? 1))
