import { LOCAL_REMOTES, PROJECTS, REPOS } from '../fixtures/core'
import type { LocalRemote, Project, Repo } from '../types/core'

/** Becomes: invoke("projects_list") */
export function useProjects(): Project[] {
  return PROJECTS
}

/** Becomes: invoke("repos_list", { projectId }) */
export function useRepos(projectId?: string): Repo[] {
  return projectId ? REPOS.filter((r) => r.projectId === projectId) : REPOS
}

/** Becomes: invoke("local_remotes_list") */
export function useLocalRemotes(): LocalRemote[] {
  return LOCAL_REMOTES
}
