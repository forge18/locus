import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project extension toggle', () => {
  it('explains the next materialization consequence', () => {
    const { getByLabelText, getByTestId } = render(() => <ProjectsView />)
    fireEvent.click(getByLabelText('Enable Agents'))
    expect(getByTestId('project-extension-groups').textContent).toContain('Excluded from the materialized tree')
  })
})
