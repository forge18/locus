import { describe, expect, it } from 'vitest'
import { readdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { HARNESSES, HARNESS_COUNT } from '../../src/fixtures/generated/harnesses'
import { SRC } from '../css'

const harnessDir = resolve(SRC, '../../../harnesses')
const tomlFiles = readdirSync(harnessDir).filter((f) => f.endsWith('.toml'))

describe('fixtures/harness-count', () => {
  it('reports 12 harnesses', () => {
    expect(HARNESS_COUNT).toBe(12)
  })

  it('counts one per harnesses/*.toml, so adding a file moves the number', () => {
    expect(HARNESS_COUNT).toBe(tomlFiles.length)
    expect(HARNESSES.length).toBe(tomlFiles.length)
  })

  it('names each harness by the name inside its file, not its filename', () => {
    expect(HARNESSES.map((h) => h.name).sort()).toEqual([
      'aider', 'antigravity', 'claude', 'codex', 'copilot', 'cursor',
      'dsh', 'gemini', 'hermes', 'omp', 'opencode', 'pi',
    ])
  })

  it('is sorted, so regenerating never churns the diff', () => {
    const names = HARNESSES.map((h) => h.name)
    expect(names).toEqual([...names].sort())
  })
})
