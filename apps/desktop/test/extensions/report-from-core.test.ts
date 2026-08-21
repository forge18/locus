import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const tauriSource = readFileSync(resolve('src-tauri/src/lib.rs'), 'utf8')

describe('extensions/report-from-core', () => {
  it('registers the Extensions report IPC command and derives it from the core registry', () => {
    expect(tauriSource).toContain('fn materialization_report()')
    expect(tauriSource).toContain('reports_for_registry(&registry)')
    expect(tauriSource).toContain('materialization_report')
  })
})
