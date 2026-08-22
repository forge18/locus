import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ProjectRail } from '../../src/shell/ProjectRail'
import { read, rules } from '../css'

const rule = (selector: string) => rules(read('shell/shell.css')).find((candidate) => candidate.selector === selector)

describe('shell/project-rail', () => {
  it('separates global and selected-project route regions in a 212px rail', () => {
    const { getByTestId } = render(() => <ProjectRail selectedProject="tapestry" />)

    expect(getByTestId('global-rail-routes').querySelectorAll('button')).not.toHaveLength(0)
    expect(getByTestId('project-rail-routes').querySelectorAll('button')).not.toHaveLength(0)
    expect(getByTestId('selected-project-card').textContent).toContain('tapestry')
    expect(rule('.project-rail')?.body).toContain('width: 212px')
  })
})
