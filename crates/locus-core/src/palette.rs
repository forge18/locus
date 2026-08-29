//! Unified global search over code, wiki pages, board tasks, and run history.
//!
//! The palette is a caller of project search and the navigation resolver. It
//! returns one locator-shaped result type rather than four bespoke destinations.

use crate::{
    search::{SearchEngine, SearchRequest},
    services::{board::BoardTask, wiki::WikiPage},
};
use anyhow::Result;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PaletteResultKind {
    Code,
    Wiki,
    Task,
    Run,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaletteResult {
    pub kind: PaletteResultKind,
    pub project: String,
    pub label: String,
    pub locator: String,
    pub score: u32,
}

impl PaletteResult {
    pub fn is_locator(&self) -> bool {
        self.locator.starts_with("locus://")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WikiSearchPage {
    pub project: String,
    pub page: WikiPage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskSearchRow {
    pub project: String,
    pub task: BoardTask,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunSearchRow {
    pub project: String,
    pub run_id: String,
    pub label: String,
}

pub struct GlobalSearch {
    project_search: SearchEngine,
    wiki: Vec<WikiSearchPage>,
    tasks: Vec<TaskSearchRow>,
    runs: Vec<RunSearchRow>,
}

impl GlobalSearch {
    pub fn new(
        project_search: SearchEngine,
        wiki: impl IntoIterator<Item = WikiSearchPage>,
        tasks: impl IntoIterator<Item = TaskSearchRow>,
        runs: impl IntoIterator<Item = RunSearchRow>,
    ) -> Self {
        Self {
            project_search,
            wiki: wiki.into_iter().collect(),
            tasks: tasks.into_iter().collect(),
            runs: runs.into_iter().collect(),
        }
    }

    /// Code hits deliberately delegate to SearchEngine, the project-search implementation.
    pub fn search_code(&self, query: &str) -> Result<Vec<PaletteResult>> {
        self.project_search
            .search(&SearchRequest::content(query))
            .map(|results| {
                results
                    .into_iter()
                    .map(|result| PaletteResult {
                        kind: PaletteResultKind::Code,
                        project: result.project.clone(),
                        label: format!("{}:{}", result.path.display(), result.line),
                        locator: format!("locus://{}/view/develop", result.project),
                        score: result.score,
                    })
                    .collect()
            })
    }

    pub fn search_wiki(&self, query: &str) -> Vec<PaletteResult> {
        let query = query.to_ascii_lowercase();
        self.wiki
            .iter()
            .filter(|row| {
                row.page.title.to_ascii_lowercase().contains(&query)
                    || row.page.body.to_ascii_lowercase().contains(&query)
                    || row.page.slug.to_ascii_lowercase().contains(&query)
            })
            .map(|row| PaletteResult {
                kind: PaletteResultKind::Wiki,
                project: row.project.clone(),
                label: row.page.title.clone(),
                locator: format!("locus://{}/page/{}", row.project, row.page.slug),
                score: match row.page.title.to_ascii_lowercase().contains(&query) {
                    true => 3,
                    false => 1,
                },
            })
            .collect()
    }

    pub fn search_tasks(&self, query: &str) -> Vec<PaletteResult> {
        let query = query.to_ascii_lowercase();
        self.tasks
            .iter()
            .filter(|row| row.task.summary.to_ascii_lowercase().contains(&query))
            .map(|row| PaletteResult {
                kind: PaletteResultKind::Task,
                project: row.project.clone(),
                label: row.task.summary.clone(),
                locator: format!("locus://{}/task/{}", row.project, row.task.id),
                score: 2,
            })
            .collect()
    }

    pub fn search_runs(&self, query: &str) -> Vec<PaletteResult> {
        let query = query.to_ascii_lowercase();
        self.runs
            .iter()
            .filter(|row| row.label.to_ascii_lowercase().contains(&query))
            .map(|row| PaletteResult {
                kind: PaletteResultKind::Run,
                project: row.project.clone(),
                label: row.label.clone(),
                locator: format!(
                    "locus://{}/session/{}/run/{}",
                    row.project, row.run_id, row.run_id
                ),
                score: 2,
            })
            .collect()
    }

    pub fn search_all(&self, query: &str) -> Result<Vec<PaletteResult>> {
        let mut results = self.search_code(query)?;
        results.extend(self.search_wiki(query));
        results.extend(self.search_tasks(query));
        results.extend(self.search_runs(query));
        results.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.project.cmp(&right.project))
                .then_with(|| left.label.cmp(&right.label))
        });
        Ok(results)
    }

    pub fn is_cross_project_by_default(&self, query: &str) -> Result<bool> {
        let projects = self
            .search_all(query)?
            .into_iter()
            .map(|result| result.project)
            .collect::<std::collections::BTreeSet<_>>();
        Ok(projects.len() > 1)
    }
}

/// The seven callers all hand the same locator to navigation; this function is
/// intentionally tiny so palette search cannot become a second resolver.
pub fn resolve_palette_locator(locator: &str) -> Result<String> {
    if !locator.starts_with("locus://") {
        anyhow::bail!("scheme: locator must start with locus://")
    }
    Ok(locator.to_owned())
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod palette {
    use super::*;
    use crate::services::manage::TaskColumn;
    use std::{fs, path::PathBuf};
    use uuid::Uuid;

    fn engine() -> SearchEngine {
        let root = std::env::temp_dir().join(format!("locus-palette-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("temp checkout");
        fs::write(root.join("README.md"), "daemon workflow task").expect("source");
        let repository =
            crate::search::SearchRepository::new("alpha", "core", PathBuf::from(&root))
                .expect("repository");
        SearchEngine::new([repository])
    }

    fn task(project: &str) -> BoardTask {
        let mut task = BoardTask::new(
            crate::ids::ProjectId::generate(),
            crate::ids::TaskId::generate(),
            "workflow task",
            Some("cargo test".into()),
        );
        task.column = TaskColumn::Ready;
        let _ = project;
        task
    }

    fn page(project: &str) -> WikiSearchPage {
        WikiSearchPage {
            project: project.into(),
            page: WikiPage {
                id: "page".into(),
                project_id: crate::ids::ProjectId::generate(),
                slug: "workflow".into(),
                kind: crate::services::wiki::PageKind::Concept,
                title: "Workflow concept".into(),
                body: "daemon workflow".into(),
                revision: 1,
                links_out: vec![],
                provenance: vec![],
                assertion_count: 0,
                source_count: 0,
            },
        }
    }

    fn search() -> GlobalSearch {
        GlobalSearch::new(
            engine(),
            [page("alpha"), page("beta")],
            [TaskSearchRow {
                project: "alpha".into(),
                task: task("alpha"),
            }],
            [RunSearchRow {
                project: "beta".into(),
                run_id: "run-1".into(),
                label: "workflow run".into(),
            }],
        )
    }

    #[test]
    fn search_code() {
        assert!(!search().search_code("daemon").unwrap().is_empty());
    }

    #[test]
    fn search_wiki() {
        assert_eq!(search().search_wiki("workflow").len(), 2);
    }

    #[test]
    fn search_tasks() {
        assert_eq!(search().search_tasks("workflow").len(), 1);
    }

    #[test]
    fn search_runs() {
        assert_eq!(search().search_runs("workflow").len(), 1);
    }

    #[test]
    fn results_are_locators() {
        assert!(search()
            .search_all("workflow")
            .unwrap()
            .iter()
            .all(PaletteResult::is_locator));
    }

    #[test]
    fn unified_ranking() {
        let results = search().search_all("workflow").unwrap();
        assert!(results
            .windows(2)
            .all(|pair| pair[0].score >= pair[1].score));
    }

    #[test]
    fn cross_project() {
        assert!(search().is_cross_project_by_default("workflow").unwrap());
        assert!(search()
            .search_all("workflow")
            .unwrap()
            .iter()
            .all(|result| !result.project.is_empty()));
    }

    #[test]
    fn reuses_project_search() {
        assert!(!search().search_code("daemon").unwrap().is_empty());
    }
}
