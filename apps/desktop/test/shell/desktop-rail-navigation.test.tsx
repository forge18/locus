import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectRail } from '../../src/shell/ProjectRail'

describe('desktop rail navigation', () => {
  it('emits a canonical desktop locator when a route is clicked', () => {
    const locators: string[] = []
    const { getByText } = render(() => <ProjectRail selectedProject="demo" onNavigate={(locator) => locators.push(locator)} />)
    fireEvent.click(getByText('Inbox'))
    expect(locators).toEqual(['locus://global/inbox'])
  })
})
