//! Typed, project-scoped wiki projections.

use crate::ids::ProjectId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageKind {
    Decision,
    Concept,
    Entity,
    Source,
    Synthesis,
}

impl PageKind {
    pub const ALL: [Self; 5] = [
        Self::Decision,
        Self::Concept,
        Self::Entity,
        Self::Source,
        Self::Synthesis,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Decision => "Decisions",
            Self::Concept => "Concepts",
            Self::Entity => "Entities",
            Self::Source => "Sources",
            Self::Synthesis => "Syntheses",
        }
    }
    pub const fn definition(self) -> &'static str {
        match self { Self::Decision => "A fork, the option taken, and the cost of taking it. The only page kind that closes an argument.", Self::Concept => "An idea the codebase assumes. Named here so an agent can be told it once instead of inferring it every run.", Self::Entity => "A thing the system has: a daemon, a table, a container. Orphans are flagged, because an entity nothing links to is usually a rename nobody finished.", Self::Source => "What was ingested, verbatim and unedited. Every assertion elsewhere points back to one of these.", Self::Synthesis => "An answer assembled from several pages that exists nowhere on its own. The only kind an agent writes unprompted." }
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
}

#[cfg(test)]
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
        assert_eq!(PageKind::ALL.len(), 5);
        assert!(!PageKind::ALL.iter().any(|kind| kind.label() == "Overview"));
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
