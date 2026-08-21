import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { StatusView } from '../../src/screens/status/StatusView'
import { read } from '../css'

const mount = () => render(() => <StatusView />)

describe('status/no-query-tool', () => {
  it('has no input of any kind', () => {
    const { container } = mount()
    expect(container.querySelectorAll('input').length).toBe(0)
    expect(container.querySelectorAll('textarea').length).toBe(0)
    expect(container.querySelectorAll('select').length).toBe(0)
  })

  it('has no filter chips', () => {
    const { container } = mount()
    expect(container.querySelectorAll('.tag').length).toBe(0)
    expect(container.querySelectorAll('.seg').length).toBe(0)
  })

  it('has no facet control', () => {
    const { container } = mount()
    expect(container.querySelectorAll('.combobox-control').length).toBe(0)
    expect(container.querySelectorAll('.menu').length).toBe(0)
  })

  it('imports nothing that could become one', () => {
    const source = [
      read('screens/status/StatusView.tsx'),
      read('screens/status/MetricCard.tsx'),
      read('screens/status/RunsByHour.tsx'),
      read('screens/status/WantsAttention.tsx'),
    ].join('\n')
    expect(source).not.toMatch(/Combobox|Segmented|Input|Tag\b/)
  })

  it('leaves digging into a run to Review — the only button here is Reassign', () => {
    const { container } = mount()
    const buttons = [...container.querySelectorAll('button')].map((b) => b.textContent)
    expect(buttons).toEqual(['Reassign'])
  })
})
