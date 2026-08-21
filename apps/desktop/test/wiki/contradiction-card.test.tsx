import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiSidebar } from '../../src/screens/wiki/WikiSidebar'
import { useContradictions } from '../../src/data/wiki'
import { read, rules } from '../css'

const [contradiction] = useContradictions()
const mount = () => render(() => <WikiSidebar />)

describe('wiki/contradiction-card', () => {
  it('states the claim', () => {
    const { getByTestId } = mount()
    expect(getByTestId('contradiction-claim').textContent).toBe(
      'Port range disagrees across two sources.',
    )
  })

  it('shows both values, in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('contradiction-value-0').textContent).toBe('43800-43999')
    expect(getByTestId('contradiction-value-1').textContent).toBe('44000-44999')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.contradiction-side')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('shows both sources, with their ages', () => {
    const { getByTestId } = mount()
    expect(getByTestId('contradiction-source-0').textContent).toContain('PLAN.md')
    expect(getByTestId('contradiction-source-0').textContent).toContain('4d')
    expect(getByTestId('contradiction-source-1').textContent).toContain('ADR-007')
    expect(getByTestId('contradiction-source-1').textContent).toContain('6h')
  })

  it('offers both actions', () => {
    const { getByTestId } = mount()
    expect(getByTestId('contradiction-adjudicate').textContent).toBe('Adjudicate')
    expect(getByTestId('contradiction-board-card').textContent).toBe('Board card')
  })

  it('rings the card in accent', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.contradiction')!.body,
    ).toContain('box-shadow: var(--ring-sel-soft)')
  })

  it('says it was flagged at ingest, not at query', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-side').textContent).toContain('flagged at ingest, not at query')
    expect(contradiction.note).toBe('flagged at ingest, not at query')
  })
})
