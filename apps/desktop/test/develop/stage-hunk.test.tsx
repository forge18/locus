import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { SideBySideDiff } from '../../src/screens/develop/SideBySideDiff'
import { useHunks } from '../../src/data/develop'

const mount = () => render(() => <DevelopView />)

describe('develop/stage-hunk', () => {
  it('offers a stage control per hunk, not just per file', () => {
    const { getByTestId } = mount()
    for (const hunk of useHunks()) {
      expect(getByTestId(`hunk-toggle-${hunk.id}`), hunk.id).toBeTruthy()
    }
  })

  it('labels each by what it would do', () => {
    const { getByTestId } = mount()
    // h-1 arrives staged, h-2 does not.
    expect(getByTestId('hunk-toggle-h-1').textContent).toContain('Unstage hunk')
    expect(getByTestId('hunk-toggle-h-2').textContent).toContain('Stage hunk')
  })

  it('moves only the hunk that was toggled', () => {
    const { getByTestId } = mount()
    getByTestId('hunk-toggle-h-2').click()
    expect(getByTestId('hunk-toggle-h-2').textContent).toContain('Unstage hunk')
    expect(getByTestId('hunk-toggle-h-1').textContent).toContain('Unstage hunk')
  })

  it('leaves the other hunk alone when the first is unstaged', () => {
    const { getByTestId } = mount()
    getByTestId('hunk-toggle-h-1').click()
    expect(getByTestId('hunk-toggle-h-1').textContent).toContain('Stage hunk')
    expect(getByTestId('hunk-toggle-h-2').textContent).toContain('Stage hunk')
  })

  it('names the range each control acts on', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hunk-toggle-h-1').textContent).toContain('-18,7')
    expect(getByTestId('hunk-toggle-h-2').textContent).toContain('-71,4')
  })

  it('reports the hunk id, so the caller stages exactly one', () => {
    const toggled: string[] = []
    const { getByTestId } = render(() => (
      <SideBySideDiff hunks={useHunks()} onToggleHunk={(id) => toggled.push(id)} />
    ))
    getByTestId('hunk-toggle-h-2').click()
    expect(toggled).toEqual(['h-2'])
  })
})
