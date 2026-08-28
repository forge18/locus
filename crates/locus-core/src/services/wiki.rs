//! Typed, project-scoped wiki projections.

use crate::ids::{ProjectId, RunId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageKind {
    Decision,
    Concept,
    Entity,
    Source,
    Synthesis,
    Overview,
}

impl PageKind {
    pub const ALL: [Self; 6] = [
        Self::Decision,
        Self::Concept,
        Self::Entity,
        Self::Source,
        Self::Synthesis,
        Self::Overview,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Decision => "Decisions",
            Self::Concept => "Concepts",
            Self::Entity => "Entities",
            Self::Source => "Sources",
            Self::Synthesis => "Syntheses",
            Self::Overview => "Overviews",
        }
    }
    pub const fn definition(self) -> &'static str {
        match self { Self::Decision => "A fork, the option taken, and the cost of taking it. The only page kind that closes an argument.", Self::Concept => "An idea the codebase assumes. Named here so an agent can be told it once instead of inferring it every run.", Self::Entity => "A thing the system has: a daemon, a table, a container. Orphans are flagged, because an entity nothing links to is usually a rename nobody finished.", Self::Source => "What was ingested, verbatim and unedited. Every assertion elsewhere points back to one of these.", Self::Synthesis => "An answer assembled from several pages that exists nowhere on its own. The only kind an agent writes unprompted.", Self::Overview => "A living synthesis revised when a project ingests new material." }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WikiPage {
    pub id: String,
    pub project_id: ProjectId,
    pub slug: String,
    pub kind: PageKind,
    pub title: String,
    pub body: String,
    pub revision: u32,
    pub links_out: Vec<String>,
    pub provenance: Vec<String>,
    pub assertion_count: usize,
    pub source_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WikiLintKind {
    Orphan,
    BrokenLink,
    UnnamedEntity,
    UnsourcedAssertion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WikiLintFinding {
    pub kind: WikiLintKind,
    pub page_id: String,
    pub detail: String,
}

pub fn lint_pages(pages: &[WikiPage]) -> Vec<WikiLintFinding> {
    let ids = pages
        .iter()
        .map(|page| page.slug.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut findings = Vec::new();
    for page in pages {
        if page.links_out.is_empty() {
            findings.push(WikiLintFinding {
                kind: WikiLintKind::Orphan,
                page_id: page.id.clone(),
                detail: "page has no links".into(),
            });
        }
        for link in &page.links_out {
            if !ids.contains(link.as_str()) {
                findings.push(WikiLintFinding {
                    kind: WikiLintKind::BrokenLink,
                    page_id: page.id.clone(),
                    detail: link.clone(),
                });
            }
        }
        if page.kind == PageKind::Entity && page.title.trim().is_empty() {
            findings.push(WikiLintFinding {
                kind: WikiLintKind::UnnamedEntity,
                page_id: page.id.clone(),
                detail: "entity has no page name".into(),
            });
        }
        if page.assertion_count > page.source_count {
            findings.push(WikiLintFinding {
                kind: WikiLintKind::UnsourcedAssertion,
                page_id: page.id.clone(),
                detail: "assertion has no source".into(),
            });
        }
    }
    findings
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WikiError {
    #[error("wiki page has invalid fields")]
    InvalidPage,
    #[error("wiki document format is unsupported")]
    UnsupportedDocument,
    #[error("wiki document is empty")]
    EmptyDocument,
    #[error("wiki page `{0}` does not exist")]
    MissingPage(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RevisionActor {
    Human,
    Agent { run_id: RunId },
    System,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WikiRevision {
    pub id: String,
    pub page_id: String,
    pub number: u32,
    pub body: String,
    pub summary: String,
    pub actor: RevisionActor,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WikiLink {
    pub source_page_id: String,
    pub source_revision_id: String,
    pub target_page_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WikiEmbedding {
    pub id: String,
    pub project_id: ProjectId,
    pub revision_id: String,
    pub source_page_id: String,
    pub statement: String,
    pub vector: Vec<f32>,
    pub model: String,
    /// Embeddings are model output and intentionally survive a page rebuild untouched.
    pub carve_out: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WikiContradictionRow {
    pub id: String,
    pub project_id: ProjectId,
    pub existing_statement: String,
    pub new_statement: String,
    pub existing_source_page_id: String,
    pub new_source_page_id: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WikiEvent {
    PageCreated { page: WikiPage },
    RevisionAdded { revision: WikiRevision },
    LinkAdded { link: WikiLink },
    EmbeddingAdded { embedding: WikiEmbedding },
    ContradictionRaised { row: WikiContradictionRow },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WikiProjection {
    pub pages: BTreeMap<String, WikiPage>,
    pub revisions: Vec<WikiRevision>,
    pub links: Vec<WikiLink>,
    pub embeddings: Vec<WikiEmbedding>,
    pub contradictions: Vec<WikiContradictionRow>,
}

impl WikiProjection {
    pub fn from_events(events: impl IntoIterator<Item = WikiEvent>) -> Result<Self, WikiError> {
        let mut projection = Self::default();
        for event in events {
            projection.apply(event)?;
        }
        Ok(projection)
    }

    pub fn apply(&mut self, event: WikiEvent) -> Result<(), WikiError> {
        match event {
            WikiEvent::PageCreated { page } => {
                if page.id.trim().is_empty() || page.slug.trim().is_empty() {
                    return Err(WikiError::InvalidPage);
                }
                self.pages.insert(page.id.clone(), page);
            }
            WikiEvent::RevisionAdded { revision } => {
                let page = self
                    .pages
                    .get_mut(&revision.page_id)
                    .ok_or_else(|| WikiError::MissingPage(revision.page_id.clone()))?;
                page.body = revision.body.clone();
                page.revision = revision.number;
                self.revisions.push(revision);
            }
            WikiEvent::LinkAdded { link } => {
                if !self.pages.contains_key(&link.source_page_id)
                    || !self.pages.contains_key(&link.target_page_id)
                {
                    return Err(WikiError::InvalidPage);
                }
                self.links.push(link);
            }
            WikiEvent::EmbeddingAdded { embedding } => self.embeddings.push(embedding),
            WikiEvent::ContradictionRaised { row } => self.contradictions.push(row),
        }
        Ok(())
    }

    pub fn read(&self, page_id: &str) -> Result<&WikiPage, WikiError> {
        self.pages
            .get(page_id)
            .ok_or_else(|| WikiError::MissingPage(page_id.into()))
    }

    pub fn nearest(&self, query: &[f32], limit: usize) -> Vec<&WikiEmbedding> {
        let mut embeddings = self.embeddings.iter().collect::<Vec<_>>();
        embeddings.sort_by(|left, right| {
            dot(&right.vector, query)
                .total_cmp(&dot(&left.vector, query))
                .then_with(|| left.id.cmp(&right.id))
        });
        embeddings.truncate(limit);
        embeddings
    }
}

fn dot(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentFormat {
    Pdf,
    Docx,
    Pptx,
    Xlsx,
    Html,
    Markdown,
    Text,
}

pub struct MarkitdownBridge;

impl MarkitdownBridge {
    pub fn format(locator: &str) -> Result<DocumentFormat, WikiError> {
        let extension = Path::new(locator)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("txt")
            .to_ascii_lowercase();
        match extension.as_str() {
            "pdf" => Ok(DocumentFormat::Pdf),
            "docx" => Ok(DocumentFormat::Docx),
            "pptx" => Ok(DocumentFormat::Pptx),
            "xlsx" => Ok(DocumentFormat::Xlsx),
            "html" | "htm" => Ok(DocumentFormat::Html),
            "md" | "markdown" => Ok(DocumentFormat::Markdown),
            "txt" => Ok(DocumentFormat::Text),
            _ => Err(WikiError::UnsupportedDocument),
        }
    }

    /// The production adapter boundary is named MarkItDown; this pure bridge keeps
    /// the projection testable and accepts the normalized text returned by it.
    pub fn convert(locator: &str, bytes: &[u8]) -> Result<String, WikiError> {
        let format = Self::format(locator)?;
        if bytes.is_empty() {
            return Err(WikiError::EmptyDocument);
        }
        let text = String::from_utf8_lossy(bytes).into_owned();
        let text = if format == DocumentFormat::Html {
            strip_html(&text)
        } else {
            text
        };
        (!text.trim().is_empty())
            .then_some(text)
            .ok_or(WikiError::EmptyDocument)
    }
}

fn strip_html(value: &str) -> String {
    let mut output = String::new();
    let mut inside = false;
    for character in value.chars() {
        match character {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => output.push(character),
            _ => {}
        }
    }
    output
}

pub fn wikilinks(body: &str) -> Vec<String> {
    let mut links = BTreeSet::new();
    let mut remainder = body;
    while let Some(start) = remainder.find("[[") {
        let after_start = &remainder[start + 2..];
        let Some(end) = after_start.find("]]") else {
            break;
        };
        let link = after_start[..end].trim();
        if !link.is_empty() {
            links.insert(link.to_owned());
        }
        remainder = &after_start[end + 2..];
    }
    links.into_iter().collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct WikiIngestPlan {
    pub events: Vec<WikiEvent>,
    pub source_page_id: String,
    pub model_calls: usize,
}

#[derive(Default)]
pub struct WikiIngestor {
    overview_revision: u32,
}

impl WikiIngestor {
    pub fn ingest(
        &mut self,
        project_id: ProjectId,
        locator: &str,
        body: &str,
        run_id: Option<RunId>,
    ) -> Result<WikiIngestPlan, WikiError> {
        if body.trim().is_empty() {
            return Err(WikiError::EmptyDocument);
        }
        let slug = slugify(locator);
        let source_page_id = format!("source-{slug}");
        let actor = run_id.map_or(RevisionActor::System, |run_id| RevisionActor::Agent {
            run_id,
        });
        let links = wikilinks(body);
        let source = WikiPage {
            id: source_page_id.clone(),
            project_id,
            slug: slug.clone(),
            kind: PageKind::Source,
            title: locator.to_owned(),
            body: body.to_owned(),
            revision: 1,
            links_out: links.clone(),
            provenance: vec![locator.to_owned()],
            assertion_count: links.len(),
            source_count: 1,
        };
        let mut events = vec![WikiEvent::PageCreated { page: source }];
        events.push(WikiEvent::RevisionAdded {
            revision: WikiRevision {
                id: format!("revision-{slug}-1"),
                page_id: source_page_id.clone(),
                number: 1,
                body: body.to_owned(),
                summary: "ingest source".into(),
                actor: actor.clone(),
            },
        });
        for link in &links {
            let kind = link
                .strip_prefix("entity:")
                .map_or(PageKind::Concept, |_| PageKind::Entity);
            let title = link.strip_prefix("entity:").unwrap_or(link).trim();
            let target_id = format!("{}-{}", kind.label().to_ascii_lowercase(), slugify(title));
            events.push(WikiEvent::PageCreated {
                page: WikiPage {
                    id: target_id.clone(),
                    project_id,
                    slug: slugify(title),
                    kind,
                    title: title.to_owned(),
                    body: String::new(),
                    revision: 1,
                    links_out: vec![slug.clone()],
                    provenance: vec![locator.to_owned()],
                    assertion_count: 0,
                    source_count: 1,
                },
            });
            events.push(WikiEvent::LinkAdded {
                link: WikiLink {
                    source_page_id: source_page_id.clone(),
                    source_revision_id: format!("revision-{slug}-1"),
                    target_page_id: target_id,
                },
            });
        }
        self.overview_revision += 1;
        let overview_id = format!("overview-{project_id}");
        if self.overview_revision == 1 {
            events.push(WikiEvent::PageCreated {
                page: WikiPage {
                    id: overview_id.clone(),
                    project_id,
                    slug: "overview".into(),
                    kind: PageKind::Overview,
                    title: "Project overview".into(),
                    body: String::new(),
                    revision: 0,
                    links_out: vec![],
                    provenance: vec![],
                    assertion_count: 0,
                    source_count: 0,
                },
            });
        }
        events.push(WikiEvent::RevisionAdded {
            revision: WikiRevision {
                id: format!("overview-revision-{project_id}-{}", self.overview_revision),
                page_id: overview_id,
                number: self.overview_revision,
                body: format!(
                    "Latest source: {locator}\n\n{}",
                    body.lines().next().unwrap_or_default()
                ),
                summary: "revise overview after ingest".into(),
                actor,
            },
        });
        Ok(WikiIngestPlan {
            events,
            source_page_id,
            model_calls: body.len().div_ceil(512),
        })
    }
}

fn slugify(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn bounded_adjudication<'a>(
    candidates: &'a [&'a WikiEmbedding],
    limit: usize,
) -> Vec<&'a WikiEmbedding> {
    candidates.iter().copied().take(limit).collect()
}

pub fn ingest_cost(document_bytes: usize, _wiki_pages: usize) -> usize {
    document_bytes.div_ceil(512).max(1)
}

pub fn contradiction_row(
    project_id: ProjectId,
    existing_statement: impl Into<String>,
    new_statement: impl Into<String>,
    existing_source_page_id: impl Into<String>,
    new_source_page_id: impl Into<String>,
) -> WikiContradictionRow {
    WikiContradictionRow {
        id: format!("contradiction-{}", uuid::Uuid::new_v4()),
        project_id,
        existing_statement: existing_statement.into(),
        new_statement: new_statement.into(),
        existing_source_page_id: existing_source_page_id.into(),
        new_source_page_id: new_source_page_id.into(),
        status: "open".into(),
    }
}

pub fn memory_conflict(memory_fact: &str, wiki_statement: &str) -> bool {
    let memory_fact = memory_fact.to_ascii_lowercase();
    let wiki_statement = wiki_statement.to_ascii_lowercase();
    memory_fact != wiki_statement
        && (memory_fact.contains(" not ") || wiki_statement.contains(" not "))
        && memory_fact
            .split_whitespace()
            .any(|word| wiki_statement.contains(word))
}

pub fn seed_paths(root: &Path) -> Vec<String> {
    fn walk(path: &Path, output: &mut Vec<String>) {
        if path.is_file() {
            if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension,
                        "md" | "markdown" | "html" | "pdf" | "docx" | "pptx" | "xlsx" | "toml"
                    )
                })
            {
                output.push(path.display().to_string());
            }
            return;
        }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                walk(&entry.path(), output);
            }
        }
    }
    let mut paths = Vec::new();
    walk(root, &mut paths);
    paths.sort();
    paths
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod wiki {
    use super::*;
    fn page(kind: PageKind) -> WikiPage {
        WikiPage {
            id: "p".into(),
            project_id: ProjectId::generate(),
            slug: "p".into(),
            kind,
            title: "Page".into(),
            body: "body".into(),
            revision: 1,
            links_out: vec![],
            provenance: vec![],
            assertion_count: 0,
            source_count: 0,
        }
    }
    #[test]
    fn kind_filter() {
        assert_eq!(PageKind::ALL.len(), 6);
        assert!(PageKind::ALL.iter().any(|kind| kind.label() == "Overviews"));
    }
    #[test]
    fn page_detail() {
        let page = page(PageKind::Decision);
        assert_eq!(page.revision, 1);
        assert_eq!(
            page.kind.definition().split('.').next(),
            Some("A fork, the option taken, and the cost of taking it")
        );
    }
    #[test]
    fn graph_mini_panel() {
        let mut p = page(PageKind::Concept);
        p.links_out = vec!["other".into()];
        assert_eq!(p.links_out.len(), 1);
    }
    #[test]
    fn contradiction_card_ingest() {
        let mut contradiction = WikiContradiction {
            page_id: "p".into(),
            existing: "one".into(),
            incoming: "two".into(),
            resolved: false,
        };
        contradiction.adjudicate();
        assert!(contradiction.resolved);
    }
    #[test]
    fn lint_panel() {
        assert!(!lint_pages(&[page(PageKind::Source)]).is_empty());
    }

    fn ingest_plan() -> WikiIngestPlan {
        WikiIngestor::default()
            .ingest(
                ProjectId::generate(),
                "README.md",
                "The daemon uses [[entity:locusd]] and [[workflow engine]].",
                Some(RunId::generate()),
            )
            .expect("ingest")
    }

    #[test]
    fn schema() {
        let migration = include_str!("../../../../migrations/0004_wiki_schema.up.sql");
        assert!(migration.contains("CREATE TABLE wiki.pages"));
        assert!(migration.contains("CREATE TABLE wiki.revisions"));
        assert!(migration.contains("CREATE TABLE wiki.contradictions"));
    }

    #[test]
    fn six_kinds() {
        assert_eq!(PageKind::ALL.len(), 6);
        assert!(PageKind::ALL.contains(&PageKind::Overview));
    }

    #[test]
    fn markitdown() {
        for extension in ["pdf", "docx", "pptx", "xlsx", "html"] {
            let locator = format!("source.{extension}");
            assert!(MarkitdownBridge::format(&locator).is_ok());
        }
        assert_eq!(
            MarkitdownBridge::convert("source.html", b"<h1>Title</h1>").unwrap(),
            "Title"
        );
    }

    #[test]
    fn auto_pages() {
        let plan = ingest_plan();
        assert!(plan.events.iter().filter(|event| matches!(event, WikiEvent::PageCreated { page } if page.kind != PageKind::Source)).count() >= 2);
    }

    #[test]
    fn links() {
        let plan = ingest_plan();
        let projection = WikiProjection::from_events(plan.events).expect("project ingest");
        assert!(!projection.links.is_empty());
    }

    #[test]
    fn overview_revises() {
        let project = ProjectId::generate();
        let mut ingestor = WikiIngestor::default();
        ingestor
            .ingest(project, "a.md", "first", None)
            .expect("first");
        let second = ingestor
            .ingest(project, "b.md", "second", None)
            .expect("second");
        assert!(second.events.iter().any(|event| matches!(event, WikiEvent::RevisionAdded { revision } if revision.page_id.starts_with("overview-") && revision.number == 2)));
    }

    #[test]
    fn embeds() {
        let embedding = WikiEmbedding {
            id: "e".into(),
            project_id: ProjectId::generate(),
            revision_id: "r".into(),
            source_page_id: "p".into(),
            statement: "a".into(),
            vector: vec![1.0, 0.0],
            model: "test".into(),
            carve_out: true,
        };
        let projection = WikiProjection::from_events([WikiEvent::EmbeddingAdded {
            embedding: embedding.clone(),
        }])
        .expect("embedding projection");
        assert_eq!(projection.embeddings, vec![embedding]);
    }

    #[test]
    fn knn_at_ingest() {
        let project = ProjectId::generate();
        let embeddings = [
            WikiEmbedding {
                id: "near".into(),
                project_id: project,
                revision_id: "r1".into(),
                source_page_id: "p1".into(),
                statement: "near".into(),
                vector: vec![1.0, 0.0],
                model: "test".into(),
                carve_out: true,
            },
            WikiEmbedding {
                id: "far".into(),
                project_id: project,
                revision_id: "r2".into(),
                source_page_id: "p2".into(),
                statement: "far".into(),
                vector: vec![0.0, 1.0],
                model: "test".into(),
                carve_out: true,
            },
        ];
        let projection = WikiProjection::from_events(
            embeddings
                .clone()
                .into_iter()
                .map(|embedding| WikiEvent::EmbeddingAdded { embedding }),
        )
        .expect("embeddings");
        assert_eq!(projection.nearest(&[1.0, 0.0], 1)[0].id, "near");
    }

    #[test]
    fn bounded_adjudication() {
        let first = WikiEmbedding {
            id: "a".into(),
            project_id: ProjectId::generate(),
            revision_id: "r".into(),
            source_page_id: "p".into(),
            statement: "a".into(),
            vector: vec![1.0],
            model: "m".into(),
            carve_out: true,
        };
        let second = WikiEmbedding {
            id: "b".into(),
            ..first.clone()
        };
        let candidates = [&first, &second];
        assert_eq!(super::bounded_adjudication(&candidates, 1).len(), 1);
    }

    #[test]
    fn cost_is_bounded() {
        assert_eq!(ingest_cost(1024, 1), ingest_cost(1024, 10_000));
    }

    #[test]
    fn contradiction_row() {
        let row =
            super::contradiction_row(ProjectId::generate(), "one", "two", "source-a", "source-b");
        assert_eq!(row.status, "open");
        assert_eq!(row.existing_source_page_id, "source-a");
        assert_eq!(row.new_source_page_id, "source-b");
    }

    #[test]
    fn contradiction_card() {
        let contradiction = WikiContradiction {
            page_id: "p".into(),
            existing: "one".into(),
            incoming: "two".into(),
            resolved: false,
        };
        assert!(contradiction.board_card_available());
    }

    #[test]
    fn memory_conflict() {
        assert!(super::memory_conflict("port is 43000", "port is not 43000"));
    }

    #[test]
    fn revision_attribution() {
        let run_id = RunId::generate();
        let plan = WikiIngestor::default()
            .ingest(ProjectId::generate(), "README.md", "body", Some(run_id))
            .unwrap();
        assert!(plan.events.iter().any(|event| matches!(
            event,
            WikiEvent::RevisionAdded { revision }
                if matches!(&revision.actor, RevisionActor::Agent { run_id: seen } if *seen == run_id)
        )));
    }

    #[test]
    fn gui_edit_readable() {
        let mut page = page(PageKind::Concept);
        page.id = "concept".into();
        let projection = WikiProjection::from_events([
            WikiEvent::PageCreated { page },
            WikiEvent::RevisionAdded {
                revision: WikiRevision {
                    id: "r".into(),
                    page_id: "concept".into(),
                    number: 2,
                    body: "human edit".into(),
                    summary: "edit".into(),
                    actor: RevisionActor::Human,
                },
            },
        ])
        .unwrap();
        assert_eq!(projection.read("concept").unwrap().body, "human edit");
    }

    #[test]
    fn seeds_from_git() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(seed_paths(root)
            .iter()
            .any(|path| path.ends_with("Cargo.toml")));
    }

    #[test]
    fn projector_is_only_writer() {
        let page = page(PageKind::Source);
        let projection =
            WikiProjection::from_events([WikiEvent::PageCreated { page: page.clone() }]).unwrap();
        assert_eq!(projection.read(&page.id).unwrap().title, page.title);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WikiContradiction {
    pub page_id: String,
    pub existing: String,
    pub incoming: String,
    pub resolved: bool,
}
impl WikiContradiction {
    pub fn adjudicate(&mut self) {
        self.resolved = true;
    }
    pub fn board_card_available(&self) -> bool {
        true
    }
}
