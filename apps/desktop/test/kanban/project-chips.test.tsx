import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { KanbanView } from '../../src/screens/automate/KanbanView'
import { useProjects } from '../../src/data/core'
import { read, rules } from '../css'

const mount = () => render(() => <KanbanView />)

describe('kanban/project-chips', () => {
  it('draws one chip per project', () => {
    const { getByTestId } = mount()
    expect(getByTestId('kanban-chips').querySelectorAll('.tag').length).toBe(useProjects().length)
  })

  it('names each project', () => {
    const { getByTestId } = mount()
    const names = [...getByTestId('kanban-chips').querySelectorAll('.tag')].map((t) => t.textContent)
    expect(names).toEqual(useProjects().map((p) => p.name))
  })

  it('uses the neutral variant, which carries the min-width that lines them up', () => {
    const { getByTestId } = mount()
    for (const chip of getByTestId('kanban-chips').querySelectorAll('.tag')) {
      expect(chip.className).toContain('tag-neutral')
    }
    expect(
      rules(read('ui/ui.css')).find((r) => r.selector === '.tag-neutral')!.body,
    ).toMatch(/min-width:\s*\d+px/)
  })

  it('right-aligns them in the header', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.kanban-chips')!.body,
    ).toContain('margin-left: auto')
  })
})
