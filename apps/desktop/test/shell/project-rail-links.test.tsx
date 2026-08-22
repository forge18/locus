import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectRail } from '../../src/shell/ProjectRail'

describe('shell/project-rail-links', () => {
  it('gives the selected project its Plan, Develop, Automate, and Review links', () => {
    const { getByTestId } = render(() => <ProjectRail selectedProject="locus" />)
    const links = getByTestId('project-rail-routes')

    expect([...links.querySelectorAll('button')].map((link) => link.textContent)).toEqual([
      'Plan',
      'Develop',
      'Automate',
      'Review',
    ])
  })
})
