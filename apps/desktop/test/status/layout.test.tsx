import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { StatusView } from '../../src/screens/status/StatusView'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)

describe('status/layout', () => {
  it('is a scrolling column', () => {
    const body = rule('.status')!.body
    expect(body).toContain('flex-direction: column')
    expect(body).toContain('overflow: auto')
  })

  it('uses the documented 15/18px padding and 14px gaps', () => {
    const body = rule('.status')!.body
    expect(body).toContain('padding: 15px 18px')
    expect(body).toContain('gap: var(--g-5)')
    expect(read('styles/tokens.css')).toContain('--g-5: 14px')
  })

  it('holds the metrics, the two middle panels and the project table', () => {
    const { getByTestId } = render(() => <StatusView />)
    for (const part of ['status-metrics', 'runs-by-hour', 'wants-attention', 'project-table']) {
      expect(getByTestId(part), part).toBeTruthy()
    }
  })

  it('reflows the middle row instead of pinning it at 1.55fr to 1fr', () => {
    expect(rule('.status-middle')!.body).toContain('repeat(auto-fit, minmax(280px, 1fr))')
  })
})
