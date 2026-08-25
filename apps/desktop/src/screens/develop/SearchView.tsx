import { For, Show, createSignal } from "solid-js";

export interface DevelopSearchResult {
      project: string;
      repo: string;
      path: string;
      line: number;
      column: number;
      excerpt: string;
      kind?: "content" | "symbol";
      score?: number;
}

export interface SearchViewProps {
      results?: readonly DevelopSearchResult[];
      initialQuery?: string;
      project?: string;
      onSearch?: (query: string) => void;
      /** The editor owns opening; SearchView only supplies the exact file/line/column. */
      onOpenResult?: (result: DevelopSearchResult) => void;
}

/** Project-scoped content/symbol results in Develop. */
export function SearchView(props: SearchViewProps) {
      const [query, setQuery] = createSignal(props.initialQuery ?? "");
      const submit = (event: SubmitEvent) => {
            event.preventDefault();
            props.onSearch?.(query().trim());
      };

      return (
            <section class="develop-search" data-testid="develop-search">
                  <form onSubmit={submit}>
                        <label for="develop-search-query">Search project files</label>
                        <div class="develop-search-input-row">
                              <input
                                    id="develop-search-query"
                                    type="search"
                                    value={query()}
                                    placeholder="Search files and symbols"
                                    aria-label="Search project files"
                                    data-testid="develop-search-input"
                                    onInput={(event) =>
                                          setQuery(event.currentTarget.value)
                                    }
                              />
                              <button
                                    type="submit"
                                    data-testid="develop-search-submit"
                              >
                                    Search
                              </button>
                        </div>
                  </form>
                  <Show when={props.project}>
                        <small data-testid="develop-search-scope">
                              Scope: {props.project}
                        </small>
                  </Show>
                  <Show
                        when={props.results?.length}
                        fallback={
                              <p data-testid="develop-search-empty">
                                    {query().trim()
                                          ? "No matches"
                                          : "Search across this project's repositories"}
                              </p>
                        }
                  >
                        <ol data-testid="develop-search-results">
                              <For each={props.results}>
                                    {(result) => (
                                          <li>
                                                <button
                                                      type="button"
                                                      data-testid="develop-search-result"
                                                      data-repo={result.repo}
                                                      data-line={result.line}
                                                      data-column={result.column}
                                                      onClick={() =>
                                                            props.onOpenResult?.(
                                                                  result,
                                                            )
                                                      }
                                                >
                                                      <span>
                                                            {result.repo} · {result.path}:
                                                            {result.line}
                                                      </span>
                                                      <small>{result.excerpt}</small>
                                                </button>
                                          </li>
                                    )}
                              </For>
                        </ol>
                  </Show>
            </section>
      );
}

export default SearchView;
