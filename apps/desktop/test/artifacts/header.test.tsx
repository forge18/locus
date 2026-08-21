import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { ARTIFACT_LOCATOR, ONE_VIEWER_NOTE } from '../../src/data/artifacts'
import { parse } from '../../src/nav'
import { read, rules } from '../css'

const mount = () => render(() => <ArtifactsView />)

describe('artifacts/header', () => {
  it('tags the kind in accent', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-kind').textContent).toBe('diff')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.wiki-kind')!.body,
    ).toContain('color: var(--ac)')
  })

  it('shows the file name in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-name').textContent).toBe('crates/locus-core/src/store/notify.rs')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.artifact-name')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('shows a locator that parses as an artifact', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-locator').textContent).toBe(ARTIFACT_LOCATOR)
    expect(parse(ARTIFACT_LOCATOR).kind).toBe('artifact')
  })

  it('says one viewer per kind, three entry points', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-one-viewer-note').textContent).toBe(ONE_VIEWER_NOTE)
    expect(ONE_VIEWER_NOTE).toBe('one viewer per kind · three entry points')
  })

  it('pushes the note right, under a bottom hairline', () => {
    const css = rules(read('screens/screens.css'))
    expect(css.find((r) => r.selector === '.artifact-note')!.body).toContain('margin-left: auto')
    expect(css.find((r) => r.selector === '.artifact-head')!.body).toContain(
      'border-bottom: 1px solid var(--line)',
    )
  })
})
