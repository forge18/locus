import { describe, expect, it } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

const EXTENSIONS = [
  'commands',
  'hooks',
  'linters',
  'output-styles',
  'rules',
  'skills',
] as const

describe('screens/desktop-workshop', () => {
  it('renders a named fixture for every Workshop route', () => {
    for (const fixture of ['agents', 'cli', ...EXTENSIONS, 'harnesses', 'providers', 'workflows-visual', 'workflows-governance'] as const) {
      const view = render(() => <WorkshopFixtureView fixture={fixture} />)
      expect(view.getByTestId(`workshop-${fixture}`)).toBeTruthy()
      view.unmount()
    }
  })

  it('keeps CLI tool scope live and calls out unsigned user tools', async () => {
    const view = render(() => <WorkshopFixtureView fixture="cli" />)
    const group = view.getByTestId('cli-group-source-control')
    expect(group.getAttribute('data-state')).toBe('mixed')
    await fireEvent.click(view.getByTestId('cli-group-toggle-source-control'))
    expect(group.getAttribute('data-state')).toBe('on')
    expect(view.getByTestId('cli-unsigned-note').textContent).toContain('read-only roles only')
  })

  it('renders masked provider credentials, aliases, verification, and selector preview', () => {
    const view = render(() => <WorkshopFixtureView fixture="providers" />)
    expect(view.getByTestId('provider-secret').textContent).toMatch(/^•+$/)
    expect(view.getByTestId('provider-keychain-note').textContent).toContain('OS keychain')
    expect(view.getByTestId('provider-verification').textContent).toContain('verified')
    expect(view.getByTestId('provider-model-alias-opus').textContent).toBe('opus')
    expect(view.getByTestId('provider-selector-preview').textContent).toContain('opus')
  })

  it('renders adapter-gated harness routing across all six bands', () => {
    const view = render(() => <WorkshopFixtureView fixture="harnesses" />)
    expect(view.getByTestId('harness-adapter-gate').textContent).toContain('adapter')
    expect(view.getAllByTestId(/autoroute-band-/)).toHaveLength(6)
    expect(view.getByTestId('autoroute-fallback').textContent).toContain('falls upward')
  })

  it('renders workflow governance without execution state', () => {
    const view = render(() => <WorkshopFixtureView fixture="workflows-governance" />)
    expect(view.getByTestId('workflow-governance-goal')).toBeTruthy()
    expect(view.getByTestId('workflow-success-criteria')).toBeTruthy()
    expect(view.queryByText(/running|queued|completed/i)).toBeNull()
  })
})
