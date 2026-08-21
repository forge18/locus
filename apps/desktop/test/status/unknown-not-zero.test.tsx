import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { MetricCard } from '../../src/screens/status/MetricCard'
import { StatusView } from '../../src/screens/status/StatusView'
import { useProjectRows } from '../../src/data/status'

const UNKNOWN = {
  label: 'Cache read',
  value: null,
  unit: '%',
  note: 'texere reports no usage',
  attention: false,
  badNote: null,
}

describe('status/unknown-not-zero', () => {
  it('renders unknown when the value is null', () => {
    const { getByTestId } = render(() => <MetricCard metric={UNKNOWN} />)
    expect(getByTestId('metric-unknown').textContent).toBe('unknown')
  })

  it('renders no numeral and no unit alongside it — 0% would be a measurement', () => {
    const { queryByTestId } = render(() => <MetricCard metric={UNKNOWN} />)
    expect(queryByTestId('metric-numeral')).toBe(null)
    expect(queryByTestId('metric-unit')).toBe(null)
  })

  it('never renders a zero for a null value', () => {
    const { container } = render(() => <MetricCard metric={UNKNOWN} />)
    expect(container.querySelector('.metric-value')!.textContent).toBe('unknown')
  })

  it('carries the same rule into the project table', () => {
    const { getByTestId } = render(() => <StatusView />)
    const texere = [...getByTestId('project-table').querySelectorAll('tbody tr')].find((r) =>
      r.textContent?.includes('texere'),
    )!
    const cells = [...texere.querySelectorAll('td')].map((c) => c.textContent)
    expect(cells).toContain('unknown')
    expect(cells.filter((c) => c === 'unknown').length).toBe(2)
  })

  it('has a fixture project whose usage really is null, so the path is exercised', () => {
    const texere = useProjectRows().find((r) => r.project === 'texere')!
    expect(texere.tokensToday).toBeNull()
    expect(texere.cache).toBeNull()
  })

  it('still shows a real number where one was reported', () => {
    const { getByTestId } = render(() => <StatusView />)
    const tapestry = [...getByTestId('project-table').querySelectorAll('tbody tr')].find((r) =>
      r.textContent?.includes('tapestry'),
    )!
    expect(tapestry.textContent).toContain('1.71M')
    expect(tapestry.textContent).toContain('88%')
  })
})
