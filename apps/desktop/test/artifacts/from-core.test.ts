import { describe, expect, it } from 'vitest'
import { read } from '../css'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const tauriSource = readFileSync(resolve('src-tauri/src/lib.rs'), 'utf8')

describe('artifacts/from-core', () => {
  it('loads review artifacts and their comments through Tauri IPC', () => {
    const source = read('data/artifacts.ts')
    expect(source).toContain('invoke<Artifact[]>("artifacts_list")')
    expect(source).toContain('invoke<ArtifactComment[]>("artifact_comments", { artifactId })')
  })

  it('registers artifact IPC commands backed by the core ArtifactStore', () => {
    expect(tauriSource).toContain('fn artifacts_list(artifacts: State')
    expect(tauriSource).toContain('review_inbox()')
    expect(tauriSource).toContain('fn artifact_comments(')
    expect(tauriSource).toContain('.manage(seeded_artifact_store())')
  })

  it('refreshes the review surface from core-owned artifact data after mount', () => {
    const source = read('screens/review/ArtifactsView.tsx')
    expect(source).toContain('fetchArtifactsFromCore')
    expect(source).toContain('fetchArtifactCommentsFromCore')
    expect(source).toContain('onMount')
  })
})
