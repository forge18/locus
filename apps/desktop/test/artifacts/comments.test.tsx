import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { COMMENTS_TITLE, LIVE_COMMENT_NOTE, useArtifactComments } from '../../src/data/artifacts'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <ArtifactsView />)

describe('artifacts/comments', () => {
  it('is headed COMMENTS STEER THE AGENT', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comments-title').textContent).toBe(COMMENTS_TITLE)
    expect(COMMENTS_TITLE).toBe('Comments steer the agent')
  })

  it('shows one entry per comment in the thread', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-rail').querySelectorAll('.comment').length).toBe(
      useArtifactComments('a-1').length,
    )
  })

  it('grounds your comment on --sf', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-c-1').getAttribute('data-author')).toBe('you')
    expect(getByTestId('comment-c-1').className).not.toContain('comment-agent')
    expect(rule('.comment').body).toContain('background: var(--surface-raised)')
  })

  it('grounds the agent reply on --sf2 with a --line2 ring', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-c-2').className).toContain('comment-agent')
    const body = rule('.comment-agent').body
    expect(body).toContain('background: var(--surface-selected)')
    expect(body).toContain('inset 0 0 0 1px var(--border-strong)')
  })

  it('gives each a 16px mono-initial avatar', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-c-2').querySelector('.comment-avatar')!.textContent).toBe('BU')
    const body = rule('.comment-avatar').body
    expect(body).toContain('width: 16px')
    expect(body).toContain('height: 16px')
    expect(body).toContain('font-family: var(--fm)')
  })

  it('pulses a note saying the comment is routed into a live session', () => {
    const { getByTestId } = mount()
    expect(getByTestId('comment-live').textContent).toContain(LIVE_COMMENT_NOTE)
    expect(getByTestId('comment-live-dot').className).toContain('pulse')
    expect(LIVE_COMMENT_NOTE).toContain('routed into the session')
  })
})
