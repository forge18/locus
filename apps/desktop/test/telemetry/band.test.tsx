import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <TelemetryView />)

describe('telemetry/band', () => {
  it('is three columns, none of which can be squeezed to nothing', () => {
    expect(rule('.tm-band').body).toContain('grid-template-columns: repeat(3, minmax(0, 1fr))')
  })

  it('takes the height it needs rather than being pinned to 434px', () => {
    expect(rule('.tm-band').body).not.toContain('height: 434px')
    expect(rule('.tm-band').body).toContain('min-height: 0')
  })

  it('holds Filters, Actions and Tools, in that order', () => {
    const { getByTestId } = mount()
    expect([...getByTestId('tm-band').children].map((c) => c.getAttribute('data-testid'))).toEqual([
      'tm-filters',
      'tm-actions',
      'tm-tools',
    ])
  })

  it('scrolls each panel on its own', () => {
    expect(rule('.tm-panel').body).toContain('overflow: auto')
  })

  it('heads each panel', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-filters').textContent).toContain('Filters')
    expect(getByTestId('tm-actions').textContent).toContain('Actions')
    expect(getByTestId('tm-tools').textContent).toContain('Tools')
  })
})
