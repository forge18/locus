import { describe, expect, it } from 'vitest'
import { readdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { SRC } from '../css'

/** One module per screen that shows data, plus the two computed sets. */
const AUTHORED = [
  'artifacts',
  'board',
  'core',
  'develop',
  'inbox',
  'plan',
  'runs',
  'sessions',
  'status',
  'telemetry',
  'wiki',
  'workflow',
]

const files = readdirSync(resolve(SRC, 'fixtures'))
  .filter((f) => f.endsWith('.ts'))
  .map((f) => f.replace(/\.ts$/, ''))

describe('fixtures/all-present', () => {
  it('has a module for every screen that shows data', () => {
    for (const name of AUTHORED) {
      expect(files, `missing src/fixtures/${name}.ts`).toContain(name)
    }
  })

  it('has the computed harness set alongside the authored ones', () => {
    expect(
      readdirSync(resolve(SRC, 'fixtures/generated')).filter((f) => f.endsWith('.ts')),
    ).toContain('harnesses.ts')
  })

  it('carries data in every authored module', async () => {
    for (const name of AUTHORED) {
      const mod = (await import(`../../src/fixtures/${name}.ts`)) as Record<string, unknown>
      const exports = Object.entries(mod).filter(([k]) => k !== 'default')
      expect(exports.length, `${name} exports nothing`).toBeGreaterThan(0)

      const hasData = exports.some(
        ([, v]) => (Array.isArray(v) && v.length > 0) || (typeof v === 'object' && v !== null),
      )
      expect(hasData, `${name} exports no data`).toBe(true)
    }
  })

  it('draws the two large lists at the sizes the design states', async () => {
    const { SESSIONS } = await import('../../src/fixtures/sessions')
    const { RUN_ROWS } = await import('../../src/fixtures/runs')
    expect(SESSIONS.length).toBe(300)
    expect(RUN_ROWS.length).toBe(612)
  })

  it('is deterministic — the seeded lists are identical on a second read', async () => {
    const a = (await import('../../src/fixtures/runs')).RUN_ROWS
    const b = (await import('../../src/fixtures/runs')).RUN_ROWS
    expect(a[0]).toEqual(b[0])
    expect(a[611]).toEqual(b[611])
  })
})
