// schema: core.projects + core.repos + core.local_remotes
// replaced by: invoke("projects_list")

import type { LocalRemote, Project, Repo } from '../types/core'

export const PROJECTS: Project[] = [
  { id: 'p-tapestry', name: 'tapestry', repoIds: ['r-tapestry-app', 'r-tapestry-web'], createdAt: '2026-05-02T09:00:00Z' },
  { id: 'p-loom-db', name: 'loom-db', repoIds: ['r-loom-db'], createdAt: '2026-05-14T09:00:00Z' },
  { id: 'p-weaver', name: 'weaver', repoIds: ['r-weaver', 'r-weaver-cli', 'r-weaver-docs'], createdAt: '2026-06-01T09:00:00Z' },
  { id: 'p-texere', name: 'texere', repoIds: ['r-texere'], createdAt: '2026-07-11T09:00:00Z' },
]

export const REPOS: Repo[] = [
  { id: 'r-tapestry-app', projectId: 'p-tapestry', name: 'tapestry', localPath: '~/Repos/tapestry', defaultBranch: 'main', localRemoteId: 'lr-tapestry-app' },
  { id: 'r-tapestry-web', projectId: 'p-tapestry', name: 'tapestry-web', localPath: '~/Repos/tapestry-web', defaultBranch: 'main', localRemoteId: 'lr-tapestry-web' },
  { id: 'r-loom-db', projectId: 'p-loom-db', name: 'loom-db', localPath: '~/Repos/loom-db', defaultBranch: 'main', localRemoteId: 'lr-loom-db' },
  { id: 'r-weaver', projectId: 'p-weaver', name: 'weaver', localPath: '~/Repos/weaver', defaultBranch: 'main', localRemoteId: 'lr-weaver' },
  { id: 'r-weaver-cli', projectId: 'p-weaver', name: 'weaver-cli', localPath: '~/Repos/weaver-cli', defaultBranch: 'main', localRemoteId: 'lr-weaver-cli' },
  { id: 'r-weaver-docs', projectId: 'p-weaver', name: 'weaver-docs', localPath: '~/Repos/weaver-docs', defaultBranch: 'main', localRemoteId: 'lr-weaver-docs' },
  { id: 'r-texere', projectId: 'p-texere', name: 'texere', localPath: '~/Repos/texere', defaultBranch: 'main', localRemoteId: 'lr-texere' },
]

export const LOCAL_REMOTES: LocalRemote[] = REPOS.map((r) => ({
  id: r.localRemoteId,
  repoId: r.id,
  path: `~/.locus/remotes/${r.name}.git`,
  branches: [`agent/8f21-notify`, `agent/3c04-index`],
}))
