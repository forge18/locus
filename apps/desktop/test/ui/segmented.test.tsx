import { createSignal } from 'solid-js'
import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Segmented } from '../../src/ui/Segmented'
import { read, rules } from '../css'

const OPTIONS = [
  { value: 'runs', label: 'Runs' },
  { value: 'events', label: 'Events' },
]

function Harness() {
  const [value, setValue] = createSignal('runs')
  return (
    <>
      <Segmented options={OPTIONS} value={value()} onChange={setValue} label="Grouping" />
      <span data-testid="value">{value()}</span>
    </>
  )
}

describe('ui/segmented', () => {
  it('renders one segment per option', () => {
    const { getByText } = render(() => <Harness />)
    expect(getByText('Runs')).toBeTruthy()
    expect(getByText('Events')).toBeTruthy()
  })

  it('reports the segment the reader picked', () => {
    const { getByText, getByTestId, container } = render(() => <Harness />)
    expect(getByTestId('value').textContent).toBe('runs')
    const events = container.querySelector('input[value="events"]') as HTMLInputElement
    events.click()
    expect(getByTestId('value').textContent).toBe('events')
    void getByText
  })

  it('marks the active segment in the DOM, not just in paint', () => {
    const { container } = render(() => <Harness />)
    const checked = container.querySelector('.seg-opt[data-checked]')
    expect(checked?.textContent).toContain('Runs')
  })

  it('draws the active segment in accent — line and text, no fill', () => {
    const css = read('ui/ui.css')
    const rule = rules(css).find(
      (r) => r.selector === '.seg-opt[data-selected],\n.seg-opt[data-checked]',
    )!
    expect(rule.body).toContain('color: var(--action-attention)')
    expect(rule.body).toContain('box-shadow: var(--ring-sel-soft)')
    expect(rule.body).not.toContain('background: var(--action-attention)')
  })

  it('names the group for a reader who cannot see the segments', () => {
    const { container } = render(() => <Harness />)
    expect(container.querySelector('.seg')!.getAttribute('aria-label')).toBe('Grouping')
  })
})
