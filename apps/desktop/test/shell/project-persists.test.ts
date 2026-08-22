import { afterEach, describe, expect, it } from 'vitest'
import {
  SELECTED_PROJECT_STORAGE_KEY,
  createProjectSelection,
} from '../../src/shell/project-selection'

afterEach(() => localStorage.removeItem(SELECTED_PROJECT_STORAGE_KEY))

describe('shell/project-persists', () => {
  it('restores the project selected before restart', () => {
    const firstWindow = createProjectSelection('tapestry')
    firstWindow.selectProject('loom-db')

    const restartedWindow = createProjectSelection('tapestry')
    expect(restartedWindow.selectedProject()).toBe('loom-db')
  })

  it('uses the supplied default when storage has no valid selection', () => {
    localStorage.setItem(SELECTED_PROJECT_STORAGE_KEY, '')

    expect(createProjectSelection('tapestry').selectedProject()).toBe('tapestry')
  })
})
