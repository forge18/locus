//! Project-wide content and symbol search.
//!
//! Search deliberately operates on editor checkouts, never run workspaces.  A project may own
//! several repositories, so the repository identity is part of every result and of the sort key.
//! Symbol data is supplied by the optional `codanna` index; an unindexed repository falls back to
//! the same content search used by the file search surface.

use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::editor::{open_checkout, EditorRepository};

/// The deliberately selected indexing policy.  Indexing is explicit because a linked checkout
/// may be changing while it is being searched; neither a timer nor an implicit git hook should
/// surprise the editor with a process or stale index state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexTrigger {
    #[default]
    OnDemand,
}

impl IndexTrigger {
    pub const fn description(self) -> &'static str {
        "codanna is indexed on demand"
    }
}

/// The one trigger policy used by project search.  Keeping this named makes the decision visible
/// to callers and leaves no accidental schedule or git-hook behavior to infer.
pub const CODANNA_INDEX_TRIGGER: IndexTrigger = IndexTrigger::OnDemand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchKind {
    Content,
    Symbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    /// `None` searches every project; `Some` limits results to that project.
    pub project: Option<String>,
    pub kind: SearchKind,
}

impl SearchRequest {
    pub fn content(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            project: None,
            kind: SearchKind::Content,
        }
    }

    pub fn symbols(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            project: None,
            kind: SearchKind::Symbol,
        }
    }

    pub fn in_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRepository {
    pub project: String,
    pub name: String,
    pub checkout: PathBuf,
    pub codanna: Option<CodannaIndex>,
}

impl SearchRepository {
    pub fn new(
        project: impl Into<String>,
        name: impl Into<String>,
        checkout: impl Into<PathBuf>,
    ) -> Result<Self> {
        let repository = Self {
            project: project.into(),
            name: name.into(),
            checkout: checkout.into(),
            codanna: None,
        };
        if repository.project.trim().is_empty() || repository.name.trim().is_empty() {
            bail!("search repository project and name are required")
        }
        if !repository.checkout.is_dir() {
            bail!(
                "search repository checkout does not exist: {}",
                repository.checkout.display()
            )
        }
        Ok(repository)
    }

    /// Resolve the ordinary editor checkout, never a run worktree.
    pub fn from_editor_repository(
        project: impl Into<String>,
        repository: &EditorRepository,
    ) -> Result<Self> {
        let checkout = open_checkout(repository)?;
        Self::new(project, &repository.name, checkout.path)
    }

    pub fn with_codanna(mut self, index: CodannaIndex) -> Self {
        self.codanna = Some(index);
        self
    }
}

/// A small, JSON-compatible projection of codanna's structural results.  The actual codanna
/// process owns index construction; Locus only consumes its result and never writes index files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodannaIndex {
    symbols: Vec<SymbolRecord>,
}

impl CodannaIndex {
    pub fn from_symbols(symbols: impl IntoIterator<Item = SymbolRecord>) -> Self {
        Self {
            symbols: symbols.into_iter().collect(),
        }
    }

    /// Read the small JSON projection emitted by an index adapter.  Both a bare array and a
    /// `{ "symbols": [...] }` envelope are accepted so codanna upgrades do not affect search.
    pub fn from_json(input: &str) -> Result<Self> {
        let symbols = serde_json::from_str::<Vec<SymbolRecord>>(input)
            .or_else(|_| serde_json::from_str::<CodannaEnvelope>(input).map(|value| value.symbols))
            .context("decode codanna symbol index")?;
        Ok(Self::from_symbols(symbols))
    }

    pub fn symbols(&self) -> &[SymbolRecord] {
        &self.symbols
    }
}

#[derive(Deserialize)]
struct CodannaEnvelope {
    symbols: Vec<SymbolRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub name: String,
    pub path: String,
    pub line: usize,
    #[serde(default = "one")]
    pub column: usize,
    #[serde(default)]
    pub kind: String,
}

impl SymbolRecord {
    pub fn new(name: impl Into<String>, path: impl Into<String>, line: usize) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            line,
            column: 1,
            kind: String::new(),
        }
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }
}

fn one() -> usize {
    1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultKind {
    Content,
    Symbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub project: String,
    pub repo: String,
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub excerpt: String,
    pub kind: ResultKind,
    pub score: u32,
}

impl SearchResult {
    /// A stable editor locator for opening this hit at its matching line.
    pub fn locator(&self) -> (&Path, usize, usize) {
        (&self.path, self.line, self.column)
    }
}

pub struct SearchEngine {
    repositories: Vec<SearchRepository>,
    /// Agent clones are explicitly excluded even if a caller accidentally supplies one as a
    /// repository.  This is a defense-in-depth check for the editor/tree boundary.
    run_clone_roots: Vec<PathBuf>,
}

impl SearchEngine {
    pub fn new(repositories: impl IntoIterator<Item = SearchRepository>) -> Self {
        Self {
            repositories: repositories.into_iter().collect(),
            run_clone_roots: Vec::new(),
        }
    }

    pub fn excluding_run_clones(
        repositories: impl IntoIterator<Item = SearchRepository>,
        run_clone_roots: impl IntoIterator<Item = PathBuf>,
    ) -> Self {
        Self {
            repositories: repositories.into_iter().collect(),
            run_clone_roots: run_clone_roots.into_iter().collect(),
        }
    }

    pub fn repositories(&self) -> &[SearchRepository] {
        &self.repositories
    }

    pub fn search(&self, request: &SearchRequest) -> Result<Vec<SearchResult>> {
        let query = request.query.trim();
        if query.is_empty() {
            bail!("search query must not be empty")
        }
        let mut results = Vec::new();
        for repository in &self.repositories {
            if request
                .project
                .as_deref()
                .is_some_and(|project| project != repository.project)
            {
                continue;
            }
            if self.is_run_clone(&repository.checkout) {
                continue;
            }
            match request.kind {
                SearchKind::Content => search_content(repository, query, &mut results)?,
                SearchKind::Symbol => {
                    if let Some(index) = &repository.codanna {
                        search_symbols(repository, index, query, &mut results);
                    } else {
                        // Structural search is useful where indexed and must still be useful
                        // immediately after a repo is added.  Content fallback is intentional.
                        search_content(repository, query, &mut results)?;
                    }
                }
            }
        }
        results.sort_by(compare_results);
        Ok(results)
    }

    fn is_run_clone(&self, checkout: &Path) -> bool {
        self.run_clone_roots
            .iter()
            .any(|root| same_or_child(checkout, root) || same_or_child(root, checkout))
    }

    /// Build the explicit command used when the user asks Locus to refresh a codanna index.
    /// Search itself never invokes this command.
    pub fn index_command(&self, project: &str, repo: &str) -> Result<IndexCommand> {
        let repository = self
            .repositories
            .iter()
            .find(|repository| repository.project == project && repository.name == repo)
            .context("repository is not in the search scope")?;
        if self.is_run_clone(&repository.checkout) {
            bail!("codanna cannot index an agent run clone")
        }
        Ok(IndexCommand {
            trigger: CODANNA_INDEX_TRIGGER,
            program: "codanna".into(),
            args: vec!["index".into(), repository.checkout.display().to_string()],
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexCommand {
    pub trigger: IndexTrigger,
    pub program: String,
    pub args: Vec<String>,
}

impl IndexCommand {
    pub fn run(&self) -> Result<()> {
        let status = Command::new(&self.program)
            .args(&self.args)
            .status()
            .with_context(|| format!("run {} index command", self.program))?;
        if status.success() {
            Ok(())
        } else {
            bail!("{} index command failed", self.program)
        }
    }
}

fn search_symbols(
    repository: &SearchRepository,
    index: &CodannaIndex,
    query: &str,
    results: &mut Vec<SearchResult>,
) {
    let needle = query.to_ascii_lowercase();
    for symbol in index.symbols() {
        if !symbol.name.to_ascii_lowercase().contains(&needle) {
            continue;
        }
        let path = repository.checkout.join(&symbol.path);
        results.push(SearchResult {
            project: repository.project.clone(),
            repo: repository.name.clone(),
            path,
            line: symbol.line.max(1),
            column: symbol.column.max(1),
            excerpt: if symbol.kind.is_empty() {
                symbol.name.clone()
            } else {
                format!("{} {}", symbol.kind, symbol.name)
            },
            kind: ResultKind::Symbol,
            score: symbol_score(&symbol.name, query),
        });
    }
}

fn symbol_score(name: &str, query: &str) -> u32 {
    let name = name.to_ascii_lowercase();
    let query = query.to_ascii_lowercase();
    if name == query {
        1_000
    } else if name.starts_with(&query) {
        750
    } else {
        500
    }
}

fn search_content(
    repository: &SearchRepository,
    query: &str,
    results: &mut Vec<SearchResult>,
) -> Result<()> {
    let query_lower = query.to_ascii_lowercase();
    let mut files = Vec::new();
    collect_files(&repository.checkout, &mut files)?;
    files.sort();
    for file in files {
        let bytes =
            fs::read(&file).with_context(|| format!("read search file {}", file.display()))?;
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        for (line_index, line) in text.lines().enumerate() {
            let line_lower = line.to_ascii_lowercase();
            if !line_lower.contains(&query_lower) {
                continue;
            }
            let column = line_lower.find(&query_lower).unwrap_or(0) + 1;
            let occurrences = line_lower.matches(&query_lower).count() as u32;
            results.push(SearchResult {
                project: repository.project.clone(),
                repo: repository.name.clone(),
                path: file.clone(),
                line: line_index + 1,
                column,
                excerpt: line.to_owned(),
                kind: ResultKind::Content,
                score: 100 + occurrences * 10 + filename_score(&file, query),
            });
        }
    }
    Ok(())
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        fs::read_dir(root).with_context(|| format!("read search tree {}", root.display()))?
    {
        let entry = entry.context("read search directory entry")?;
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        let file_type = entry.file_type().context("read search entry type")?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn filename_score(path: &Path, query: &str) -> u32 {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            let name = name.to_ascii_lowercase();
            let query = query.to_ascii_lowercase();
            if name == query {
                50
            } else if name.contains(&query) {
                20
            } else {
                0
            }
        })
        .unwrap_or(0)
}

fn compare_results(left: &SearchResult, right: &SearchResult) -> Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.project.cmp(&right.project))
        .then_with(|| left.repo.cmp(&right.repo))
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.line.cmp(&right.line))
        .then_with(|| left.column.cmp(&right.column))
}

fn same_or_child(path: &Path, root: &Path) -> bool {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    path == root || path.starts_with(root)
}

/// A stable projection useful to the UI and IPC boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SearchResultRow {
    pub project: String,
    pub repo: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub excerpt: String,
    pub kind: String,
    pub score: u32,
}

impl From<SearchResult> for SearchResultRow {
    fn from(result: SearchResult) -> Self {
        Self {
            project: result.project,
            repo: result.repo,
            path: result.path.display().to_string(),
            line: result.line,
            column: result.column,
            excerpt: result.excerpt,
            kind: match result.kind {
                ResultKind::Content => "content".into(),
                ResultKind::Symbol => "symbol".into(),
            },
            score: result.score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("locus-search-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn repo(&self, project: &str, name: &str, content: &str) -> SearchRepository {
            let path = self.0.join(project).join(name);
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("src.txt"), content).unwrap();
            SearchRepository::new(project, name, path).unwrap()
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn single_repo() {
        let temp = TempDir::new();
        let engine = SearchEngine::new([temp.repo("p", "one", "needle\n")]);
        let results = engine.search(&SearchRequest::content("needle")).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].repo, "one");
        assert_eq!(results[0].line, 1);
    }

    #[test]
    fn all_project_repos() {
        let temp = TempDir::new();
        let engine = SearchEngine::new([
            temp.repo("p", "one", "needle\n"),
            temp.repo("p", "two", "needle\n"),
        ]);
        assert_eq!(
            engine
                .search(&SearchRequest::content("needle"))
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn results_carry_repo() {
        let temp = TempDir::new();
        let result = SearchEngine::new([temp.repo("p", "named", "needle")])
            .search(&SearchRequest::content("needle"))
            .unwrap()
            .remove(0);
        assert_eq!(
            (result.project.as_str(), result.repo.as_str()),
            ("p", "named")
        );
    }

    #[test]
    fn unified_ranking() {
        let temp = TempDir::new();
        let engine = SearchEngine::new([
            temp.repo("p", "weak", "needle\n"),
            temp.repo("p", "strong", "needle needle\n"),
        ]);
        let results = engine.search(&SearchRequest::content("needle")).unwrap();
        assert_eq!(results[0].repo, "strong");
    }

    #[test]
    fn respects_scope() {
        let temp = TempDir::new();
        let engine = SearchEngine::new([
            temp.repo("included", "one", "needle"),
            temp.repo("excluded", "two", "needle"),
        ]);
        let results = engine
            .search(&SearchRequest::content("needle").in_project("included"))
            .unwrap();
        assert_eq!(
            results
                .iter()
                .map(|result| result.project.as_str())
                .collect::<Vec<_>>(),
            ["included"]
        );
    }

    #[test]
    fn symbols() {
        let temp = TempDir::new();
        let repo =
            temp.repo("p", "one", "not the symbol text")
                .with_codanna(CodannaIndex::from_symbols([SymbolRecord::new(
                    "process_payment",
                    "src/lib.rs",
                    8,
                )]));
        let results = SearchEngine::new([repo])
            .search(&SearchRequest::symbols("process_payment"))
            .unwrap();
        assert_eq!(results[0].kind, ResultKind::Symbol);
        assert_eq!(results[0].line, 8);
    }

    #[test]
    fn degrades_gracefully() {
        let temp = TempDir::new();
        let results = SearchEngine::new([temp.repo("p", "one", "process_payment")])
            .search(&SearchRequest::symbols("process_payment"))
            .unwrap();
        assert_eq!(results[0].kind, ResultKind::Content);
    }

    #[test]
    fn never_reads_run_clones() {
        let temp = TempDir::new();
        let run = temp.0.join("run");
        fs::create_dir_all(&run).unwrap();
        fs::write(run.join("secret.txt"), "needle").unwrap();
        let engine = SearchEngine::excluding_run_clones([temp.repo("p", "one", "needle")], [run]);
        assert_eq!(
            engine
                .search(&SearchRequest::content("needle"))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn index_trigger() {
        assert_eq!(CODANNA_INDEX_TRIGGER, IndexTrigger::OnDemand);
        assert_eq!(
            CODANNA_INDEX_TRIGGER.description(),
            "codanna is indexed on demand"
        );
    }
}
