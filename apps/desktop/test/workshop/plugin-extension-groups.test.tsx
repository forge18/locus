import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectRail } from '../../src/shell/ProjectRail'

describe('workshop/plugin-extension-groups', () => {
  it('exposes exactly Plugins and Extensions subgroup locators', () => {
    const view = render(() => <ProjectRail selectedProject="locus" />)
    fireEvent.click(view.getByRole('button', { name: 'Workshop' }))
    expect(view.getByTestId('workshop-plugins-group')).toBeTruthy()
    expect(view.getByTestId('workshop-extensions-group')).toBeTruthy()
    expect(view.getByTestId('workshop-plugins-group').textContent).toContain('Plugins')
    expect(view.getByTestId('workshop-extensions-group').textContent).toContain('Extensions')
  })

  it('keeps plugin links limited to CLI Tool, Harness, and Provider', () => {
    const view = render(() => <ProjectRail selectedProject="locus" />)
    fireEvent.click(view.getByRole('button', { name: 'Workshop' }))
    const labels = [...view.getByTestId('workshop-plugin-links').querySelectorAll('button')]
      .map((button) => button.textContent)
    expect(labels).toEqual(['CLI Tool', 'Harness', 'Provider'])
  })
})
