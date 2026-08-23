import { createSignal, type Accessor } from 'solid-js'

export const SELECTED_PROJECT_STORAGE_KEY = 'locus.selected-project'

export interface ProjectSelection {
  selectedProject: Accessor<string>
  selectProject: (project: string) => void
}

function storedProject(fallback: string): string {
  const stored = localStorage.getItem(SELECTED_PROJECT_STORAGE_KEY)
  return stored?.trim() || fallback
}

/** The selected-project card's state, shared by the desktop rail and route resolver. */
export function createProjectSelection(defaultProject: string): ProjectSelection {
  const [selectedProject, setSelectedProject] = createSignal(storedProject(defaultProject))

  return {
    selectedProject,
    selectProject: (project) => {
      const selected = project.trim()
      if (!selected) return
      localStorage.setItem(SELECTED_PROJECT_STORAGE_KEY, selected)
      setSelectedProject(selected)
    },
  }
}
