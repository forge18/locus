import {
  BRANCH,
  FILE_TREE,
  GIT,
  HUNKS,
  LINKED_CHECKOUT,
  SELECTED_FILE,
  TABS,
} from '../fixtures/develop'
import type { GitPanel, Hunk, TreeNode } from '../fixtures/develop'

export {
  BRANCH,
  COMMIT_PLACEHOLDER,
  DIFF_LEFT_HEADER,
  DIFF_RIGHT_HEADER,
  LINKED_REPO_NOTE,
  OWNERSHIP_NOTE,
  PRIMARY_SURFACE_NOTE,
} from '../fixtures/develop'
export type { Commit, DiffCell, DiffRow, DiffRowKind, EditorTab, GitFile, GitPanel, Hunk, TreeNode } from '../fixtures/develop'

/** Becomes: invoke("file_tree", { repoId, branch }) */
export function useFileTree(): TreeNode[] {
  return FILE_TREE
}

/** Becomes: invoke("editor_tabs") — pane state, once the pane manager owns it. */
export function useEditorTabs() {
  return TABS
}

/** Becomes: invoke("diff_for_file", { repoId, branch, path }) */
export function useHunks(): Hunk[] {
  return HUNKS
}

/** Becomes: invoke("git_status", { repoId }) */
export function useGitPanel(): GitPanel {
  return GIT
}

/** Becomes: pane state, once the pane manager owns it. */
export function useSelectedFile(): string {
  return SELECTED_FILE
}

/** Becomes: invoke("repo_local_path", { repoId }) */
export function useLinkedCheckout(): string {
  return LINKED_CHECKOUT
}

/** Becomes: invoke("current_branch", { repoId }) */
export function useBranch(): string {
  return BRANCH
}
