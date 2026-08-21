import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { RESOLVE, SEND_TO_SESSION } from '../../src/data/artifacts'
import { read, rules } from '../css'

const mount = () => render(() => <ArtifactsView />)

describe('artifacts/comment-actions', () => {
  it('offers a textarea to write in', () => {
    const { getByTestId } = mount()
    const box = getByTestId('comment-input')
    expect(box.tagName).toBe('TEXTAREA')
    expect(box.className).toContain('input')
  })

  it('offers Send to session as the primary', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-send').textContent).toBe(SEND_TO_SESSION)
    expect(getByTestId('comment-send').className).toContain('btn-primary')
  })

  it('offers Resolve as the secondary', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-resolve').textContent).toBe(RESOLVE)
    expect(getByTestId('comment-resolve').className).toContain('btn-secondary')
  })

  it('names sending as sending to the session, not to a queue', () => {
    expect(SEND_TO_SESSION).toBe('Send to session')
  })

  it('sits at the foot of the rail under a top hairline', () => {
    const { getByTestId } = mount()
    const rail = getByTestId('comment-rail')
    const foot = getByTestId('comment-foot')
    expect([...rail.children].indexOf(foot)).toBe(2)
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.comment-foot')!.body,
    ).toContain('border-top: 1px solid var(--line)')
  })
})
