import { MergeEditor } from "../../editor/MergeEditor";
import {
      SearchView,
      type DevelopSearchResult,
} from "./SearchView";

export interface DevelopGitState {
      branch: string;
      dirty: boolean;
      ahead: number;
      behind: number;
      agentBranches: string[];
}

export interface DevelopViewProps {
      base?: string;
      agent?: string;
      /** Populated by the `repo_git_state` Tauri command for a real checkout. */
      gitState?: DevelopGitState;
      searchResults?: readonly DevelopSearchResult[];
      searchProject?: string;
      onSearch?: (query: string) => void;
      /** The editor host opens the ordinary checkout at this exact location. */
      onOpenSearchResult?: (result: DevelopSearchResult) => void;
}

/** Develop's review surface is the real CodeMirror merge view, not a fixture diff. */
export function DevelopView(props: DevelopViewProps) {
      return (
            <div class="develop" data-testid="develop-real-diff">
                  <SearchView
                        results={props.searchResults}
                        project={props.searchProject}
                        onSearch={props.onSearch}
                        onOpenResult={props.onOpenSearchResult}
                  />
                  <aside
                        class="develop-git-state"
                        data-testid="develop-git-from-core"
                  >
                        <strong>
                              {props.gitState?.branch ??
                                    "checkout not attached"}
                        </strong>
                        <span>{props.gitState?.dirty ? "dirty" : "clean"}</span>
                        <span>
                              {props.gitState
                                    ? `${props.gitState.ahead} ahead · ${props.gitState.behind} behind`
                                    : "Git state arrives from repo_git_state"}
                        </span>
                  </aside>
                  <MergeEditor
                        base={props.base ?? "const value = 1;\n"}
                        agent={props.agent ?? "const value = 2;\n"}
                  />
            </div>
      );
}

export default DevelopView;
