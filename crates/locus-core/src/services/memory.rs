//! Project-scoped memory folds and curated fact revisions.
//!
//! The Postgres schema is the durable source; these types are the service boundary used by the
//! CLI, store adapters, and desktop projections.  Editing is append-only: revision one remains
//! the agent's assertion and revision two is the human curation returned by recall.

use crate::{
    ids::{ProjectId, RunId},
    services::artifact::{ArtifactContent, SessionResearchFeed},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::f64::consts::LN_2;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceState {
    Verified,
    Asserted,
    Decaying,
    Contradicted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactRevision {
    pub rev: u32,
    pub value: String,
    pub written_by_run: Option<RunId>,
    pub curated_by: Option<String>,
    pub written_at: i64,
    pub score: Option<f32>,
}

impl FactRevision {
    pub fn new(
        rev: u32,
        value: impl Into<String>,
        written_by_run: Option<RunId>,
        written_at: i64,
    ) -> Result<Self, MemoryError> {
        let value = value.into();
        if rev == 0 || value.trim().is_empty() {
            return Err(MemoryError::InvalidFact);
        }
        if contains_secret(&value) {
            return Err(MemoryError::SecretRejected);
        }
        Ok(Self {
            rev,
            value,
            written_by_run,
            curated_by: None,
            written_at,
            score: None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub project_id: ProjectId,
    pub subject: String,
    pub confidence: ConfidenceState,
    pub revisions: Vec<FactRevision>,
    pub recall_count: u32,
}

impl Fact {
    pub fn new(
        id: impl Into<String>,
        project_id: ProjectId,
        subject: impl Into<String>,
        value: impl Into<String>,
        run: Option<RunId>,
        now: i64,
    ) -> Result<Self, MemoryError> {
        let id = id.into();
        let subject = subject.into();
        if id.trim().is_empty() || subject.trim().is_empty() {
            return Err(MemoryError::InvalidFact);
        }
        if contains_secret(&subject) {
            return Err(MemoryError::SecretRejected);
        }
        Ok(Self {
            id,
            project_id,
            subject,
            confidence: ConfidenceState::Asserted,
            revisions: vec![FactRevision::new(1, value, run, now)?],
            recall_count: 0,
        })
    }

    pub fn latest(&self) -> &FactRevision {
        self.revisions
            .last()
            .expect("a Fact always has revision one")
    }
    pub fn score(&self) -> Option<f32> {
        (self.confidence != ConfidenceState::Contradicted)
            .then(|| self.latest().score.unwrap_or(0.0))
    }
    pub fn edit(
        &mut self,
        value: impl Into<String>,
        human: impl Into<String>,
        now: i64,
    ) -> Result<&FactRevision, MemoryError> {
        let value = value.into();
        let human = human.into();
        if value.trim().is_empty() || human.trim().is_empty() {
            return Err(MemoryError::InvalidFact);
        }
        let revision = FactRevision {
            rev: self.latest().rev + 1,
            value,
            written_by_run: self
                .revisions
                .first()
                .and_then(|revision| revision.written_by_run),
            curated_by: Some(human),
            written_at: now,
            score: self.latest().score,
        };
        self.revisions.push(revision);
        Ok(self.latest())
    }
    pub fn recall(&mut self) -> &FactRevision {
        self.recall_count += 1;
        self.latest()
    }
    pub fn contradict(&mut self) {
        self.confidence = ConfidenceState::Contradicted;
        for revision in &mut self.revisions {
            revision.score = None;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    facts: BTreeMap<String, Fact>,
}

impl MemoryStore {
    pub fn insert(&mut self, fact: Fact) -> Result<(), MemoryError> {
        if self.facts.insert(fact.id.clone(), fact).is_some() {
            return Err(MemoryError::DuplicateFact);
        }
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&Fact> {
        self.facts.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Fact> {
        self.facts.get_mut(id)
    }
    pub fn recall(&mut self, id: &str) -> Result<FactRevision, MemoryError> {
        self.facts
            .get_mut(id)
            .map(|fact| fact.recall().clone())
            .ok_or(MemoryError::UnknownFact)
    }
    pub fn project_facts(&self, project: ProjectId) -> Vec<&Fact> {
        self.facts
            .values()
            .filter(|fact| fact.project_id == project)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contradiction {
    pub fact_id: String,
    pub existing_value: String,
    pub incoming_value: String,
    pub existing_source: Option<RunId>,
    pub incoming_source: Option<RunId>,
    pub resolved: bool,
}

impl Contradiction {
    pub fn adjudicate(&mut self) {
        self.resolved = true;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceGroup {
    ShortTerm,
    LongTerm,
    Artifacts,
}

impl PersistenceGroup {
    pub fn can_delete(self) -> bool {
        matches!(self, Self::LongTerm | Self::Artifacts)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum MemoryError {
    #[error("invalid memory fact")]
    InvalidFact,
    #[error("memory fact already exists")]
    DuplicateFact,
    #[error("memory fact does not exist")]
    UnknownFact,
    #[error("memory tier is at capacity")]
    AtCapacity,
    #[error("memory scope does not match the project or agent")]
    ScopeMismatch,
    #[error("memory contains a credential-like value")]
    SecretRejected,
    #[error("session research findings require a closed review")]
    ResearchFeedNotClosed,
    #[error("memory candidate failed re-verification")]
    ReverificationFailed,
}

/// The small, run-local tier is deliberately bounded.  Refusing an insert preserves the
/// provenance of entries already in the tier; callers must consolidate rather than silently
/// evicting an observation.
pub const CORE_MEMORY_CAPACITY: usize = 40;
pub const CATALOG_MAX_ENTRIES: usize = 40;
pub const CATALOG_MAX_TOKENS: usize = 800;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScopeKind {
    Project,
    Agent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    Strategy,
    Fact,
    Assumption,
    Failure,
}

impl MemoryCategory {
    pub fn half_life_days(self) -> f64 {
        match self {
            Self::Strategy => 38.0,
            Self::Fact => 24.0,
            Self::Assumption => 19.0,
            Self::Failure => 11.0,
        }
    }
}

/// The only eviction labels a durable memory may carry.  Tool output is not a memory
/// record, so `disposable` remains intentionally outside this enum.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionClass {
    Sticky,
    #[default]
    Standard,
}

impl EvictionClass {
    pub const ALL: [Self; 2] = [Self::Sticky, Self::Standard];

    pub const fn is_sticky(self) -> bool {
        matches!(self, Self::Sticky)
    }
}

/// Capture provenance used to derive the stored eviction label.  The origin is kept in
/// the service model so a fold can validate that `eviction_class` was not assigned by
/// an unrelated caller.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCaptureOrigin {
    #[default]
    Observation,
    UnresolvedError,
    DeclaredPromotion,
}

impl MemoryCaptureOrigin {
    pub const fn eviction_class(self) -> EvictionClass {
        match self {
            Self::UnresolvedError | Self::DeclaredPromotion => EvictionClass::Sticky,
            Self::Observation => EvictionClass::Standard,
        }
    }
}

/// Compatibility alias for callers that use the shorter domain term.
pub type CaptureOrigin = MemoryCaptureOrigin;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromotionMetadata {
    pub density_bypassed: bool,
    pub reverified: bool,
    pub deduplicated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub project_id: ProjectId,
    pub scope: MemoryScopeKind,
    pub agent_id: Option<String>,
    pub path: String,
    pub subject: String,
    pub category: MemoryCategory,
    pub body: String,
    pub provenance: serde_json::Value,
    #[serde(default)]
    pub capture_origin: MemoryCaptureOrigin,
    #[serde(default)]
    pub eviction_class: EvictionClass,
    #[serde(default)]
    pub promotion: Option<PromotionMetadata>,
    pub embedding: Vec<f32>,
    pub embedding_model: String,
    pub confidence: f64,
    pub importance: f64,
    pub recall_count: u32,
    pub active_days: u32,
    pub strength: f64,
    pub had_match: bool,
    pub archived: bool,
    pub invalidated: bool,
    pub reverify_requested: bool,
}

impl MemoryEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        project_id: ProjectId,
        scope: MemoryScopeKind,
        agent_id: Option<String>,
        path: impl Into<String>,
        subject: impl Into<String>,
        category: MemoryCategory,
        body: impl Into<String>,
        provenance: serde_json::Value,
        embedding: Vec<f32>,
        embedding_model: impl Into<String>,
        importance: f64,
    ) -> Result<Self, MemoryError> {
        let entry = Self {
            id: id.into(),
            project_id,
            scope,
            agent_id,
            path: path.into(),
            subject: subject.into(),
            category,
            body: body.into(),
            provenance,
            capture_origin: MemoryCaptureOrigin::default(),
            eviction_class: EvictionClass::default(),
            promotion: None,
            embedding,
            embedding_model: embedding_model.into(),
            confidence: 1.0,
            importance: importance.clamp(0.0, 1.0),
            recall_count: 0,
            active_days: 0,
            strength: importance.clamp(0.0, 1.0),
            had_match: false,
            archived: false,
            invalidated: false,
            reverify_requested: false,
        };
        if contains_secret(&entry.body) || contains_secret(&entry.provenance.to_string()) {
            return Err(MemoryError::SecretRejected);
        }
        if entry.id.trim().is_empty()
            || entry.path.trim().is_empty()
            || entry.subject.trim().is_empty()
            || entry.body.trim().is_empty()
            || entry.embedding_model.trim().is_empty()
            || (entry.scope == MemoryScopeKind::Agent && entry.agent_id.is_none())
            || (entry.scope == MemoryScopeKind::Project && entry.agent_id.is_some())
        {
            return Err(MemoryError::InvalidFact);
        }
        Ok(entry)
    }

    /// Construct a record with an explicit capture origin and its derived eviction class.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_origin(
        id: impl Into<String>,
        project_id: ProjectId,
        scope: MemoryScopeKind,
        agent_id: Option<String>,
        path: impl Into<String>,
        subject: impl Into<String>,
        category: MemoryCategory,
        body: impl Into<String>,
        provenance: serde_json::Value,
        embedding: Vec<f32>,
        embedding_model: impl Into<String>,
        importance: f64,
        origin: MemoryCaptureOrigin,
    ) -> Result<Self, MemoryError> {
        let mut entry = Self::new(
            id,
            project_id,
            scope,
            agent_id,
            path,
            subject,
            category,
            body,
            provenance,
            embedding,
            embedding_model,
            importance,
        )?;
        entry.set_capture_origin(origin);
        Ok(entry)
    }

    pub fn set_capture_origin(&mut self, origin: MemoryCaptureOrigin) {
        self.capture_origin = origin;
        self.eviction_class = origin.eviction_class();
    }

    pub fn with_capture_origin(mut self, origin: MemoryCaptureOrigin) -> Self {
        self.set_capture_origin(origin);
        self
    }

    pub fn eviction_class_is_derived(&self) -> bool {
        self.eviction_class == self.capture_origin.eviction_class()
    }

    pub fn decay_strength(&self) -> f64 {
        let base_lambda = LN_2 / self.category.half_life_days();
        let effective_lambda = base_lambda * (1.0 - self.importance * 0.8);
        (self.importance
            * (-effective_lambda * f64::from(self.active_days)).exp()
            * (1.0 + f64::from(self.recall_count) * 0.2))
            .clamp(0.0, 1.0)
    }

    pub fn refresh_decay(&mut self) {
        self.strength = self.decay_strength();
    }
}

#[derive(Clone, Debug)]
pub struct CoreMemory {
    entries: Vec<MemoryEntry>,
    capacity: usize,
}

impl CoreMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }
    pub fn insert(&mut self, entry: MemoryEntry) -> Result<(), MemoryError> {
        if self.entries.len() >= self.capacity {
            return Err(MemoryError::AtCapacity);
        }
        self.entries.push(entry);
        Ok(())
    }
    pub fn entries(&self) -> &[MemoryEntry] {
        &self.entries
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CoreMemory {
    fn default() -> Self {
        Self::new(CORE_MEMORY_CAPACITY)
    }
}

/// Short-term observations are keyed by project, never by a process-wide shared buffer.
#[derive(Clone, Debug, Default)]
pub struct ProbationBuffer {
    projects: HashMap<ProjectId, Vec<MemoryEntry>>,
}

impl ProbationBuffer {
    pub fn add(&mut self, entry: MemoryEntry) -> Result<(), MemoryError> {
        if entry.scope != MemoryScopeKind::Agent {
            return Err(MemoryError::ScopeMismatch);
        }
        self.projects
            .entry(entry.project_id)
            .or_default()
            .push(entry);
        Ok(())
    }
    pub fn for_project(&self, project: ProjectId) -> &[MemoryEntry] {
        self.projects
            .get(&project)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub fn projects(&self) -> usize {
        self.projects.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogEntry {
    pub path: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub entries: Vec<CatalogEntry>,
    pub text: String,
    pub consolidation_required: bool,
    /// Human-readable records for entries excluded by the catalog budget.
    pub dropped: Vec<String>,
}

impl Catalog {
    pub fn drop_log(&self) -> &[String] {
        &self.dropped
    }
}

fn token_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn catalog_line(entry: &CatalogEntry) -> String {
    format!("- {}: {}", entry.path, entry.summary)
}

struct CatalogCandidate<'a> {
    source: &'a MemoryEntry,
    entry: CatalogEntry,
}

pub fn build_catalog(entries: &[MemoryEntry]) -> Catalog {
    let mut candidates: Vec<_> = entries
        .iter()
        .map(|source| CatalogCandidate {
            source,
            entry: CatalogEntry {
                path: source.path.clone(),
                summary: source.body.lines().next().unwrap_or("").trim().to_owned(),
            },
        })
        .collect();
    let all_tokens = candidates
        .iter()
        .map(|candidate| token_count(&catalog_line(&candidate.entry)))
        .sum::<usize>();
    let overflow = candidates.len() > CATALOG_MAX_ENTRIES || all_tokens > CATALOG_MAX_TOKENS;
    if overflow {
        // Sticky entries are selected first. Standard entries use strength as the
        // overflow ordering, with path as a deterministic tie-breaker.
        candidates.sort_by(|left, right| {
            right
                .source
                .eviction_class
                .is_sticky()
                .cmp(&left.source.eviction_class.is_sticky())
                .then_with(|| {
                    if left.source.eviction_class == EvictionClass::Standard {
                        right
                            .source
                            .strength
                            .partial_cmp(&left.source.strength)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .then_with(|| left.entry.path.cmp(&right.entry.path))
        });
    } else {
        candidates.sort_by(|left, right| left.entry.path.cmp(&right.entry.path));
    }

    let mut selected = Vec::new();
    let mut dropped = Vec::new();
    let mut used = 0;
    for candidate in candidates {
        let line = catalog_line(&candidate.entry);
        let cost = token_count(&line);
        let sticky = candidate.source.eviction_class.is_sticky();
        let fits = selected.len() < CATALOG_MAX_ENTRIES && used + cost <= CATALOG_MAX_TOKENS;
        // Sticky records are never discarded in favor of standard records. The
        // normal corpus has far fewer sticky entries than the fixed catalog cap;
        // retaining them here also makes the precedence explicit at pathological
        // budgets rather than silently losing a safety decision.
        if sticky || fits {
            used += cost;
            selected.push(candidate.entry);
        } else {
            dropped.push(format!(
                "dropped memory candidate {} after catalog overflow",
                candidate.source.id
            ));
        }
    }
    let text = selected
        .iter()
        .map(catalog_line)
        .collect::<Vec<_>>()
        .join("\n");
    Catalog {
        entries: selected,
        text,
        consolidation_required: overflow,
        dropped,
    }
}

/// A tail fraction is intentionally a policy value, not a fixed token budget. The
/// effective window is supplied by the bound model/runtime.
pub const CATALOG_TAIL_FRACTION_PERCENT: usize = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextBudget {
    pub effective_window_tokens: usize,
    pub catalog_tokens: usize,
    pub tail_fraction_percent: usize,
}

impl ContextBudget {
    pub fn from_effective_window(effective_window_tokens: usize) -> Self {
        Self {
            effective_window_tokens,
            catalog_tokens: CATALOG_MAX_TOKENS,
            tail_fraction_percent: CATALOG_TAIL_FRACTION_PERCENT,
        }
    }

    pub fn with_catalog_tokens(mut self, catalog_tokens: usize) -> Self {
        self.catalog_tokens = catalog_tokens;
        self
    }

    pub fn with_tail_fraction_percent(mut self, tail_fraction_percent: usize) -> Self {
        self.tail_fraction_percent = tail_fraction_percent.min(100);
        self
    }

    pub fn tail_budget_tokens(self) -> usize {
        self.effective_window_tokens
            .saturating_sub(self.catalog_tokens)
            .saturating_mul(self.tail_fraction_percent)
            / 100
    }
}

pub fn derived_tail_budget(effective_window_tokens: usize) -> usize {
    ContextBudget::from_effective_window(effective_window_tokens).tail_budget_tokens()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogTail {
    entries: Vec<CatalogEntry>,
    text: String,
    budget_tokens: usize,
    seen_paths: BTreeSet<String>,
    dropped: Vec<String>,
}

impl CatalogTail {
    pub fn new(budget: ContextBudget) -> Self {
        Self {
            entries: Vec::new(),
            text: String::new(),
            budget_tokens: budget.tail_budget_tokens(),
            seen_paths: BTreeSet::new(),
            dropped: Vec::new(),
        }
    }

    pub fn for_snapshot(snapshot: &Catalog, budget: ContextBudget) -> Self {
        let mut tail = Self::new(budget);
        tail.seen_paths
            .extend(snapshot.entries.iter().map(|entry| entry.path.clone()));
        tail
    }

    pub fn entries(&self) -> &[CatalogEntry] {
        &self.entries
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn budget_tokens(&self) -> usize {
        self.budget_tokens
    }

    pub fn dropped(&self) -> &[String] {
        &self.dropped
    }

    /// Append one captured memory in arrival order. A path already present in
    /// the frozen snapshot or tail is not duplicated.
    pub fn append(&mut self, source: &MemoryEntry) -> bool {
        if !self.seen_paths.insert(source.path.clone()) {
            return false;
        }
        let entry = CatalogEntry {
            path: source.path.clone(),
            summary: source.body.lines().next().unwrap_or("").trim().to_owned(),
        };
        let cost = token_count(&catalog_line(&entry));
        if cost > self.budget_tokens {
            self.dropped.push(format!(
                "dropped memory candidate {} after tail overflow",
                source.id
            ));
            return false;
        }

        // Check remaining capacity before adding the new item. Older tail
        // entries are dropped first, but survivors never change order or text.
        while self.used_tokens() + cost > self.budget_tokens && !self.entries.is_empty() {
            let removed = self.entries.remove(0);
            self.dropped.push(format!(
                "dropped memory candidate {} after tail overflow",
                removed.path
            ));
        }
        if self.used_tokens() + cost > self.budget_tokens {
            self.dropped.push(format!(
                "dropped memory candidate {} after tail overflow",
                source.id
            ));
            return false;
        }
        self.entries.push(entry);
        self.rebuild_text();
        true
    }

    pub fn used_tokens(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| token_count(&catalog_line(entry)))
            .sum()
    }

    fn rebuild_text(&mut self) {
        self.text = self
            .entries
            .iter()
            .map(catalog_line)
            .collect::<Vec<_>>()
            .join("\n");
    }
}

#[derive(Clone, Debug)]
pub struct FrozenCatalog(Catalog);
impl FrozenCatalog {
    pub fn start(entries: &[MemoryEntry]) -> Self {
        Self(build_catalog(entries))
    }
    pub fn snapshot(&self) -> &Catalog {
        &self.0
    }
    pub fn tail(&self, budget: ContextBudget) -> CatalogTail {
        CatalogTail::for_snapshot(&self.0, budget)
    }
}

pub fn materialize_catalog_context(base: &str, catalog: &Catalog) -> String {
    if catalog.text.is_empty() {
        return base.to_owned();
    }
    if base.is_empty() {
        return catalog.text.clone();
    }
    format!("{base}\n\n{catalog_text}", catalog_text = catalog.text)
}

/// Add mutable, append-only content after the frozen catalog head.
pub fn materialize_catalog_context_with_tail(
    base: &str,
    catalog: &Catalog,
    tail: &CatalogTail,
) -> String {
    let head = materialize_catalog_context(base, catalog);
    if tail.text.is_empty() {
        return head;
    }
    if head.is_empty() {
        return tail.text.clone();
    }
    format!("{head}\n\n{tail}", tail = tail.text)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionReceipt {
    pub id: String,
    pub metadata: PromotionMetadata,
}

#[derive(Clone, Debug, Default)]
pub struct DurableMemoryStore {
    entries: BTreeMap<String, MemoryEntry>,
    edges: BTreeSet<(String, String)>,
}

impl DurableMemoryStore {
    pub fn insert(&mut self, entry: MemoryEntry) -> Result<(), MemoryError> {
        if self.entries.contains_key(&entry.id) {
            return Err(MemoryError::DuplicateFact);
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.entries.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut MemoryEntry> {
        self.entries.get_mut(id)
    }
    pub fn entries(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.values()
    }
    pub fn add_edge(&mut self, source: &str, target: &str) {
        if source != target {
            self.edges.insert((source.to_owned(), target.to_owned()));
        }
    }
    pub fn promote(&mut self, candidate: MemoryEntry) -> Result<(), MemoryError> {
        self.promote_with_metadata(
            candidate,
            PromotionMetadata {
                density_bypassed: false,
                reverified: false,
                deduplicated: false,
            },
        )
        .map(|_| ())
    }

    /// Explicit promotion is the declared path for one-off decisions. It bypasses
    /// cluster density, but not re-verification or path/subject deduplication.
    pub fn promote_declared(
        &mut self,
        candidate: MemoryEntry,
        method: Reverification,
        passed: bool,
    ) -> Result<PromotionReceipt, MemoryError> {
        if !reverify(&candidate, method, passed) {
            return Err(MemoryError::ReverificationFailed);
        }
        let candidate = candidate.with_capture_origin(MemoryCaptureOrigin::DeclaredPromotion);
        self.promote_with_metadata(
            candidate,
            PromotionMetadata {
                density_bypassed: true,
                reverified: true,
                deduplicated: false,
            },
        )
    }

    /// Name the R2 operation explicitly for callers that do not use the CLI term.
    pub fn promote_without_density(
        &mut self,
        candidate: MemoryEntry,
        method: Reverification,
        passed: bool,
    ) -> Result<PromotionReceipt, MemoryError> {
        self.promote_declared(candidate, method, passed)
    }

    fn promote_with_metadata(
        &mut self,
        mut candidate: MemoryEntry,
        mut metadata: PromotionMetadata,
    ) -> Result<PromotionReceipt, MemoryError> {
        if let Some(previous) = self
            .entries
            .values_mut()
            .find(|entry| entry.path == candidate.path && !entry.archived)
        {
            candidate.id = format!("{}-promoted", candidate.id);
            previous.archived = true;
            metadata.deduplicated = true;
        }
        candidate.promotion = Some(metadata.clone());
        let id = candidate.id.clone();
        self.insert(candidate)?;
        Ok(PromotionReceipt { id, metadata })
    }

    pub fn prune(&mut self) {
        let strong: HashSet<String> = self
            .entries
            .values()
            .filter(|e| e.strength >= 0.05 && !e.invalidated)
            .map(|e| e.id.clone())
            .collect();
        let keep_from_chain: HashSet<String> = self
            .edges
            .iter()
            .filter_map(|(a, b)| {
                if strong.contains(a) {
                    Some(b.clone())
                } else if strong.contains(b) {
                    Some(a.clone())
                } else {
                    None
                }
            })
            .collect();
        for entry in self.entries.values_mut() {
            if entry.had_match && entry.strength < 0.05 && !keep_from_chain.contains(&entry.id) {
                entry.archived = true;
            }
        }
    }
}

pub fn promote_reviewed_session_findings(
    feed: &mut SessionResearchFeed,
    memory: &mut DurableMemoryStore,
) -> Result<usize, MemoryError> {
    if !feed.is_closed() {
        return Err(MemoryError::ResearchFeedNotClosed);
    }
    let candidates = feed
        .reviewed_findings()
        .filter(|finding| !finding.promoted)
        .map(|finding| {
            let body = match &finding.artifact.content {
                ArtifactContent::Text(body) => body.clone(),
                ArtifactContent::Blob { .. } => return Err(MemoryError::InvalidFact),
            };
            let artifact_id = finding.artifact.id;
            let provenance = serde_json::json!({
                "source": "session_research",
                "session_id": feed.session_id().to_string(),
                "artifact_id": artifact_id.to_string(),
                "provenance": finding.provenance.label(),
            });
            let subject = finding
                .artifact
                .summary
                .clone()
                .unwrap_or_else(|| "Session research finding".into());
            let candidate = MemoryEntry::new(
                format!("finding-{artifact_id}"),
                finding.artifact.project_id,
                MemoryScopeKind::Project,
                None,
                format!("session-research/{artifact_id}"),
                subject,
                MemoryCategory::Fact,
                body,
                provenance,
                Vec::new(),
                "session-research",
                0.5,
            )?;
            Ok((artifact_id, candidate))
        })
        .collect::<Result<Vec<_>, MemoryError>>()?;
    let mut promoted = 0;
    for (artifact_id, candidate) in candidates {
        memory.promote(candidate)?;
        feed.mark_promoted(artifact_id)
            .map_err(|_| MemoryError::InvalidFact)?;
        promoted += 1;
    }
    Ok(promoted)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskClass {
    Code,
    Plan,
    Research,
}
impl TaskClass {
    pub fn retrieval(self) -> (usize, bool) {
        match self {
            Self::Code | Self::Plan => (1, false),
            Self::Research => (8, true),
        }
    }
    pub fn turn_injection(self) -> bool {
        matches!(self, Self::Research)
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f64 {
    let (mut dot, mut aa, mut bb) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (x, y) in a.iter().zip(b) {
        dot += f64::from(*x) * f64::from(*y);
        aa += f64::from(*x).powi(2);
        bb += f64::from(*y).powi(2);
    }
    if aa == 0.0 || bb == 0.0 {
        0.0
    } else {
        dot / (aa.sqrt() * bb.sqrt())
    }
}

pub fn embedding_similarity(a: &[f32], b: &[f32]) -> f64 {
    cosine(a, b)
}

/// Selection-time settings for research recall. The conservative default only
/// removes near-identical embeddings; code and plan recall never consults it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecallSettings {
    pub research_similarity_threshold: f64,
}

impl Default for RecallSettings {
    fn default() -> Self {
        Self {
            research_similarity_threshold: 0.95,
        }
    }
}

impl RecallSettings {
    pub fn with_research_similarity_threshold(mut self, threshold: f64) -> Self {
        if threshold.is_finite() {
            self.research_similarity_threshold = threshold.clamp(0.0, 1.0);
        }
        self
    }
}

pub fn recall(
    store: &mut DurableMemoryStore,
    project: ProjectId,
    agent: Option<&str>,
    query: &str,
    embedding: &[f32],
    task: TaskClass,
) -> Vec<(String, f64)> {
    recall_with_settings(
        store,
        project,
        agent,
        query,
        embedding,
        task,
        RecallSettings::default(),
    )
}

pub fn recall_with_similarity_threshold(
    store: &mut DurableMemoryStore,
    project: ProjectId,
    agent: Option<&str>,
    query: &str,
    embedding: &[f32],
    task: TaskClass,
    threshold: f64,
) -> Vec<(String, f64)> {
    recall_with_settings(
        store,
        project,
        agent,
        query,
        embedding,
        task,
        RecallSettings::default().with_research_similarity_threshold(threshold),
    )
}

pub fn recall_with_settings(
    store: &mut DurableMemoryStore,
    project: ProjectId,
    agent: Option<&str>,
    query: &str,
    embedding: &[f32],
    task: TaskClass,
    settings: RecallSettings,
) -> Vec<(String, f64)> {
    let (k, expand) = task.retrieval();
    let terms: Vec<_> = query
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .collect();
    let mut ranked: Vec<_> = store
        .entries
        .values_mut()
        .filter(|entry| {
            entry.project_id == project
                && !entry.archived
                && !entry.invalidated
                && (entry.scope == MemoryScopeKind::Project || entry.agent_id.as_deref() == agent)
        })
        .map(|entry| {
            let lexical = terms
                .iter()
                .filter(|term| {
                    entry.body.to_ascii_lowercase().contains(term.as_str())
                        || entry.subject.to_ascii_lowercase().contains(term.as_str())
                })
                .count() as f64;
            let similarity = (lexical + cosine(&entry.embedding, embedding).max(0.0)).min(1.0);
            let score = similarity * entry.strength;
            (entry, score)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut result = Vec::new();
    let mut selected_embeddings = Vec::<Vec<f32>>::new();
    for (entry, score) in ranked {
        if result.len() >= k {
            break;
        }
        if task == TaskClass::Research
            && selected_embeddings.iter().any(|selected| {
                !selected.is_empty()
                    && !entry.embedding.is_empty()
                    && cosine(selected, &entry.embedding) >= settings.research_similarity_threshold
            })
        {
            continue;
        }
        selected_embeddings.push(entry.embedding.clone());
        entry.recall_count += 1;
        entry.had_match = true;
        entry.refresh_decay();
        result.push((entry.id.clone(), score));
    }
    if expand {
        let ids: HashSet<_> = result.iter().map(|(id, _)| id.clone()).collect();
        let existing: HashSet<_> = ids.clone();
        for (source, target) in &store.edges {
            if ids.contains(source) && !existing.contains(target) {
                result.push((target.clone(), 0.0));
            }
        }
    }
    result
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolChange {
    SignatureChanged,
    BodyChanged,
    AstUnchanged,
}
pub fn apply_symbol_change(entry: &mut MemoryEntry, change: SymbolChange) {
    match change {
        SymbolChange::SignatureChanged => entry.invalidated = true,
        SymbolChange::BodyChanged => entry.reverify_requested = true,
        SymbolChange::AstUnchanged => {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reverification {
    Codanna,
    Test,
    VerifyResult,
    Provenance,
}
pub fn reverify(candidate: &MemoryEntry, method: Reverification, passed: bool) -> bool {
    matches!(
        method,
        Reverification::Codanna | Reverification::Test | Reverification::VerifyResult
    ) && passed
        || matches!(method, Reverification::Provenance) && candidate.confidence > 0.0
}

#[derive(Clone, Debug, Default)]
pub struct Keeper;
impl Keeper {
    pub fn is_agent_definition() -> bool {
        true
    }
    pub fn triggers_on_idle(running_runs: usize, queued_runs: usize) -> bool {
        running_runs == 0 && queued_runs == 0
    }
    pub fn pass(store: &mut DurableMemoryStore) {
        for entry in store.entries.values_mut() {
            entry.refresh_decay();
        }
        store.prune();
    }
}

pub fn promotion_count(candidates: &[MemoryEntry], path: &str) -> usize {
    candidates.iter().filter(|entry| entry.path == path).count()
}

pub fn should_promote(candidates: &[MemoryEntry], path: &str) -> bool {
    promotion_count(candidates, path) >= 3
}

pub fn measured_importance(injected: bool, recalled: bool, verify_passed: bool) -> f64 {
    match (injected, recalled, verify_passed) {
        (true, true, true) => 1.0,
        (true, true, false) => 0.6,
        (true, false, _) => 0.2,
        _ => 0.0,
    }
}

fn contains_secret(value: &str) -> bool {
    ["sk-", "ghp_", "github_pat_", "AKIA", "xoxb-"]
        .iter()
        .any(|marker| value.contains(marker))
}

/// Non-foldable values are explicit so schema additions cannot accidentally become part of the
/// event fold. Embeddings are model output; decay state depends on the wall clock.
pub const MEMORY_CARVE_OUTS: &[&str] = &[
    "embedding",
    "confidence",
    "active_days",
    "strength",
    "last_active",
];

pub fn overflow_drop_log(entries: &[MemoryEntry], capacity: usize) -> Vec<String> {
    entries
        .iter()
        .skip(capacity)
        .map(|entry| format!("dropped memory candidate {} after promotion", entry.id))
        .collect()
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod memory {
    use super::*;
    #[test]
    fn fact_revision_schema() {
        let fact = Fact::new(
            "f",
            ProjectId::generate(),
            "subject",
            "one",
            Some(RunId::generate()),
            1,
        )
        .unwrap();
        assert_eq!(fact.latest().rev, 1);
    }
    #[test]
    fn edit_appends_revision() {
        let run = RunId::generate();
        let mut fact =
            Fact::new("f", ProjectId::generate(), "subject", "one", Some(run), 1).unwrap();
        fact.edit("two", "human", 2).unwrap();
        assert_eq!(fact.revisions[0].value, "one");
        assert_eq!(fact.latest().rev, 2);
    }
    #[test]
    fn recall_returns_curated() {
        let run = RunId::generate();
        let mut fact =
            Fact::new("f", ProjectId::generate(), "subject", "one", Some(run), 1).unwrap();
        fact.edit("two", "human", 2).unwrap();
        let revision = fact.recall().clone();
        assert_eq!(revision.value, "two");
        assert_eq!(revision.written_by_run, Some(run));
    }
    #[test]
    fn confidence_state_enum() {
        assert_eq!(
            serde_json::to_string(&ConfidenceState::Verified).unwrap(),
            "\"verified\""
        );
    }
    #[test]
    fn contradicted_has_no_score() {
        let mut fact = Fact::new("f", ProjectId::generate(), "subject", "one", None, 1).unwrap();
        fact.contradict();
        assert_eq!(fact.score(), None);
    }
    #[test]
    fn persistence_groups() {
        assert!(!PersistenceGroup::ShortTerm.can_delete());
        assert!(PersistenceGroup::LongTerm.can_delete());
    }
    #[test]
    fn delete_scoped_to_long_term_and_artifacts() {
        assert!(!PersistenceGroup::ShortTerm.can_delete());
        assert!(PersistenceGroup::Artifacts.can_delete());
    }

    #[test]
    fn promotes_reviewed_session_findings() {
        let project = ProjectId::generate();
        let run = RunId::generate();
        let first = crate::services::artifact::ArtifactRow::text(
            project,
            run,
            crate::services::artifact::ArtifactKind::Finding,
            "keep this finding",
        );
        let second = crate::services::artifact::ArtifactRow::text(
            project,
            run,
            crate::services::artifact::ArtifactKind::Finding,
            "leave this finding in the feed",
        );
        let first_id = first.id;
        let mut feed = SessionResearchFeed::new(crate::ids::SessionId::generate());
        feed.record_run_finding(first).unwrap();
        feed.record_run_finding(second).unwrap();
        let mut memory = DurableMemoryStore::default();
        assert_eq!(
            promote_reviewed_session_findings(&mut feed, &mut memory),
            Err(MemoryError::ResearchFeedNotClosed)
        );
        assert_eq!(feed.review_at_close([first_id]).unwrap(), 1);
        assert_eq!(
            promote_reviewed_session_findings(&mut feed, &mut memory).unwrap(),
            1
        );
        assert_eq!(memory.entries().count(), 1);
        let promoted = feed
            .findings()
            .find(|finding| finding.artifact.id == first_id)
            .unwrap();
        assert!(promoted.reviewed);
        assert!(promoted.promoted);
        assert_eq!(
            promoted.provenance,
            crate::services::artifact::ResearchProvenance::SessionClose
        );
    }

    fn entry(project: ProjectId, id: usize, path: &str) -> MemoryEntry {
        MemoryEntry::new(
            format!("m{id}"),
            project,
            MemoryScopeKind::Project,
            None,
            path,
            "subject",
            MemoryCategory::Fact,
            "one line summary",
            serde_json::json!({"run": id}),
            vec![1.0, 0.0],
            "test",
            0.8,
        )
        .unwrap()
    }

    #[test]
    fn core_refuses_over_cap() {
        let project = ProjectId::generate();
        let mut core = CoreMemory::new(2);
        core.insert(entry(project, 1, "a")).unwrap();
        core.insert(entry(project, 2, "b")).unwrap();
        assert_eq!(
            core.insert(entry(project, 3, "c")),
            Err(MemoryError::AtCapacity)
        );
        assert_eq!(core.len(), 2);
    }

    #[test]
    fn store_schema() {
        let value = entry(ProjectId::generate(), 1, "src/lib.rs");
        assert!(!value.provenance.is_null());
        assert_eq!(value.embedding_model, "test");
        assert!((0.0..=1.0).contains(&value.confidence));
    }

    #[test]
    fn eviction_class_schema() {
        let value = entry(ProjectId::generate(), 1, "src/lib.rs");
        assert_eq!(value.eviction_class, EvictionClass::Standard);
        assert_eq!(EvictionClass::default(), EvictionClass::Standard);
        assert_eq!(
            serde_json::to_string(&EvictionClass::Sticky).unwrap(),
            "\"sticky\""
        );
        assert_eq!(EvictionClass::ALL.len(), 2);
    }

    #[test]
    fn eviction_class_assignment() {
        let project = ProjectId::generate();
        let ordinary = entry(project, 1, "ordinary");
        let error =
            entry(project, 2, "error").with_capture_origin(MemoryCaptureOrigin::UnresolvedError);
        let declared = entry(project, 3, "declared")
            .with_capture_origin(MemoryCaptureOrigin::DeclaredPromotion);
        assert!(ordinary.eviction_class_is_derived());
        assert_eq!(error.eviction_class, EvictionClass::Sticky);
        assert_eq!(declared.eviction_class, EvictionClass::Sticky);
        assert!(error.eviction_class_is_derived());
        assert!(declared.eviction_class_is_derived());
    }

    #[test]
    fn probation_is_project_scoped() {
        let one = ProjectId::generate();
        let two = ProjectId::generate();
        let mut buffer = ProbationBuffer::default();
        let mut candidate = entry(one, 1, "a");
        candidate.scope = MemoryScopeKind::Agent;
        candidate.agent_id = Some("agent".into());
        buffer.add(candidate).unwrap();
        assert_eq!(buffer.for_project(one).len(), 1);
        assert!(buffer.for_project(two).is_empty());
    }

    #[test]
    fn no_shared_short_term() {
        let mut buffer = ProbationBuffer::default();
        let project = ProjectId::generate();
        let mut candidate = entry(project, 1, "a");
        candidate.scope = MemoryScopeKind::Project;
        assert_eq!(buffer.add(candidate), Err(MemoryError::ScopeMismatch));
        assert_eq!(buffer.projects(), 0);
    }

    #[test]
    fn catalog_cap() {
        let project = ProjectId::generate();
        let entries: Vec<_> = (0..100)
            .map(|id| entry(project, id, &format!("p/{id}")))
            .collect();
        let catalog = build_catalog(&entries);
        assert!(catalog.text.split_whitespace().count() <= CATALOG_MAX_TOKENS);
        assert!(catalog.entries.len() <= CATALOG_MAX_ENTRIES);
    }

    #[test]
    fn overflow_consolidates() {
        let project = ProjectId::generate();
        let entries: Vec<_> = (0..41)
            .map(|id| entry(project, id, &format!("p/{id}")))
            .collect();
        assert!(build_catalog(&entries).consolidation_required);
    }

    #[test]
    fn catalog_overflow_respects_eviction_class() {
        let project = ProjectId::generate();
        let mut entries = Vec::new();
        let mut sticky = entry(project, 0, "z-sticky");
        sticky.eviction_class = EvictionClass::Sticky;
        sticky.capture_origin = MemoryCaptureOrigin::UnresolvedError;
        entries.push(sticky);
        for id in 1..=40 {
            let mut value = entry(project, id, &format!("standard/{id}"));
            value.strength = if id == 40 { 0.01 } else { 0.99 };
            entries.push(value);
        }
        let catalog = build_catalog(&entries);
        assert!(catalog.entries.iter().any(|entry| entry.path == "z-sticky"));
        assert!(catalog
            .drop_log()
            .iter()
            .any(|message| message.contains("m40")));
        assert!(catalog
            .drop_log()
            .iter()
            .all(|message| !message.contains("m0")));
    }

    #[test]
    fn catalog_is_frozen() {
        let project = ProjectId::generate();
        let mut entries = vec![entry(project, 1, "a")];
        let frozen = FrozenCatalog::start(&entries);
        entries.push(entry(project, 2, "b"));
        assert_eq!(frozen.snapshot().entries.len(), 1);
    }

    #[test]
    fn no_leading_brace() {
        let catalog = build_catalog(&[entry(ProjectId::generate(), 1, "a")]);
        assert!(!catalog.text.starts_with('{'));
    }

    #[test]
    fn catalog_fallback_path() {
        let catalog = build_catalog(&[entry(ProjectId::generate(), 1, "a")]);
        let context = materialize_catalog_context("base", &catalog);
        assert!(context.contains("base") && context.contains("a"));
    }

    #[test]
    fn catalog_tail_append() {
        let project = ProjectId::generate();
        let initial = vec![entry(project, 1, "initial")];
        let frozen = FrozenCatalog::start(&initial);
        let budget = ContextBudget::from_effective_window(1_000)
            .with_catalog_tokens(100)
            .with_tail_fraction_percent(50);
        let mut tail = frozen.tail(budget);
        let first = entry(project, 2, "new/first");
        let second = entry(project, 3, "new/second");
        assert!(tail.append(&first));
        assert!(tail.append(&second));
        assert_eq!(
            tail.entries()
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            ["new/first", "new/second"]
        );
        assert!(tail.used_tokens() <= tail.budget_tokens());
        assert!(derived_tail_budget(2_000) > derived_tail_budget(1_000));

        let mut small = CatalogTail::new(
            ContextBudget::from_effective_window(109)
                .with_catalog_tokens(100)
                .with_tail_fraction_percent(100),
        );
        assert!(small.append(&first));
        assert!(small.append(&second));
        assert!(!small.dropped().is_empty());
        assert!(small.text().contains("new/second"));
    }

    #[test]
    fn snapshot_head_stable() {
        let project = ProjectId::generate();
        let frozen = FrozenCatalog::start(&[entry(project, 1, "initial")]);
        let head = materialize_catalog_context("base", frozen.snapshot());
        let mut tail = frozen.tail(
            ContextBudget::from_effective_window(1_000)
                .with_catalog_tokens(100)
                .with_tail_fraction_percent(100),
        );
        let new = entry(project, 2, "new");
        assert!(tail.append(&new));
        let with_tail = materialize_catalog_context_with_tail("base", frozen.snapshot(), &tail);
        assert!(with_tail.starts_with(&head));
        assert_eq!(&with_tail[head.len()..], "\n\n- new: one line summary");
        assert!(tail.text().lines().all(|line| !line.starts_with('{')));
    }

    #[test]
    fn reverification() {
        let value = entry(ProjectId::generate(), 1, "a");
        assert!(reverify(&value, Reverification::Codanna, true));
        assert!(!reverify(&value, Reverification::Test, false));
    }

    #[test]
    fn dedup_by_path() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        store.insert(entry(project, 1, "same")).unwrap();
        store.promote(entry(project, 2, "same")).unwrap();
        assert!(store.get("m1").unwrap().archived);
    }

    #[test]
    fn promoted_without_density_reverifies_and_is_sticky() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        let receipt = store
            .promote_declared(
                entry(project, 1, "one-off"),
                Reverification::Provenance,
                true,
            )
            .expect("declared promotion passes provenance verification");
        assert!(receipt.metadata.density_bypassed);
        assert!(receipt.metadata.reverified);
        let promoted = store.get(&receipt.id).expect("promoted record");
        assert_eq!(promoted.eviction_class, EvictionClass::Sticky);
        assert_eq!(
            promoted.capture_origin,
            MemoryCaptureOrigin::DeclaredPromotion
        );
        assert!(promoted.promotion.is_some());
        assert!(store
            .promote_declared(entry(project, 2, "rejected"), Reverification::Test, false)
            .is_err());
    }

    #[test]
    fn promoted_decay_normally() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        let mut candidate = entry(project, 1, "one-off");
        candidate.active_days = 24;
        let receipt = store
            .promote_declared(candidate, Reverification::Provenance, true)
            .expect("promote");
        Keeper::pass(&mut store);
        let promoted = store.get(&receipt.id).expect("promoted record");
        assert!(promoted.strength < promoted.importance);
        assert!(!promoted.archived, "cold-start guard still applies");
    }

    #[test]
    fn importance_is_measured() {
        assert_eq!(measured_importance(true, true, true), 1.0);
        assert_eq!(measured_importance(false, false, false), 0.0);
    }

    #[test]
    fn promotes_at_three() {
        let project = ProjectId::generate();
        let values: Vec<_> = (0..3).map(|id| entry(project, id, "same")).collect();
        assert!(!should_promote(&values[..2], "same"));
        assert!(should_promote(&values, "same"));
    }

    #[test]
    fn archives_originals() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        store.insert(entry(project, 1, "same")).unwrap();
        store.promote(entry(project, 2, "same")).unwrap();
        assert!(store.entries().any(|entry| entry.archived));
    }

    #[test]
    fn overflow_logs_drops() {
        let project = ProjectId::generate();
        let values: Vec<_> = (0..3).map(|id| entry(project, id, "p")).collect();
        let log = overflow_drop_log(&values, 2);
        assert_eq!(log.len(), 1);
        assert!(log[0].contains("m2"));
    }

    #[test]
    fn signature_change_invalidates() {
        let mut value = entry(ProjectId::generate(), 1, "symbol");
        apply_symbol_change(&mut value, SymbolChange::SignatureChanged);
        assert!(value.invalidated);
    }
    #[test]
    fn body_change_flags() {
        let mut value = entry(ProjectId::generate(), 1, "symbol");
        apply_symbol_change(&mut value, SymbolChange::BodyChanged);
        assert!(value.reverify_requested);
    }
    #[test]
    fn ast_stable_is_noop() {
        let mut value = entry(ProjectId::generate(), 1, "symbol");
        apply_symbol_change(&mut value, SymbolChange::AstUnchanged);
        assert!(!value.invalidated && !value.reverify_requested);
    }

    #[test]
    fn decay_curve() {
        let project = ProjectId::generate();
        for category in [
            MemoryCategory::Strategy,
            MemoryCategory::Fact,
            MemoryCategory::Assumption,
            MemoryCategory::Failure,
        ] {
            let mut value = MemoryEntry::new(
                "id",
                project,
                MemoryScopeKind::Project,
                None,
                "p",
                "s",
                category,
                "b",
                serde_json::json!({}),
                vec![1.0],
                "test",
                0.8,
            )
            .unwrap();
            value.active_days = 10;
            assert!(value.decay_strength() < 0.8);
        }
    }
    #[test]
    fn chain_aware_pruning() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        let mut weak = entry(project, 1, "weak");
        weak.active_days = 1000;
        weak.had_match = true;
        store.insert(weak).unwrap();
        store.insert(entry(project, 2, "strong")).unwrap();
        store.add_edge("m1", "m2");
        Keeper::pass(&mut store);
        assert!(!store.get("m1").unwrap().archived);
    }
    #[test]
    fn cold_start_guard() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        let mut value = entry(project, 1, "new");
        value.active_days = 1000;
        store.insert(value).unwrap();
        Keeper::pass(&mut store);
        assert!(!store.get("m1").unwrap().archived);
    }

    #[test]
    fn hybrid_recall() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        store.insert(entry(project, 1, "p")).unwrap();
        assert_eq!(
            recall(
                &mut store,
                project,
                None,
                "one",
                &[1.0, 0.0],
                TaskClass::Code
            )
            .len(),
            1
        );
    }
    #[test]
    fn rank_by_similarity_times_strength() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        let mut low = entry(project, 1, "a");
        low.strength = 0.1;
        let high = entry(project, 2, "b");
        store.insert(low).unwrap();
        store.insert(high).unwrap();
        assert_eq!(
            recall(
                &mut store,
                project,
                None,
                "one",
                &[1.0, 0.0],
                TaskClass::Research
            )[0]
            .0,
            "m2"
        );
    }

    #[test]
    fn research_diversity_dedup() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        let mut near = entry(project, 1, "near");
        near.embedding = vec![0.999, 0.01];
        let mut distinct = entry(project, 2, "distinct");
        distinct.embedding = vec![0.0, 1.0];
        store.insert(entry(project, 0, "first")).unwrap();
        store.insert(near).unwrap();
        store.insert(distinct).unwrap();
        let selected = recall_with_similarity_threshold(
            &mut store,
            project,
            None,
            "one",
            &[1.0, 0.0],
            TaskClass::Research,
            0.95,
        );
        assert!(selected.iter().any(|(id, _)| id == "m0"));
        assert!(!selected.iter().any(|(id, _)| id == "m1"));
        let embeddings = [vec![1.0, 0.0], vec![0.0, 1.0]];
        for (left, right) in embeddings.iter().zip(embeddings.iter().skip(1)) {
            assert!(embedding_similarity(left, right) < 0.95);
        }
    }

    #[test]
    fn k1_selection_unchanged() {
        let project = ProjectId::generate();
        let mut first = DurableMemoryStore::default();
        first.insert(entry(project, 1, "first")).unwrap();
        first.insert(entry(project, 2, "second")).unwrap();
        let mut second = first.clone();
        let old = recall(
            &mut first,
            project,
            None,
            "one",
            &[1.0, 0.0],
            TaskClass::Code,
        );
        let configured = recall_with_similarity_threshold(
            &mut second,
            project,
            None,
            "one",
            &[1.0, 0.0],
            TaskClass::Code,
            0.0,
        );
        assert_eq!(old, configured);
    }

    #[test]
    fn depth_by_task_class() {
        assert_eq!(TaskClass::Code.retrieval(), (1, false));
        assert_eq!(TaskClass::Plan.retrieval(), (1, false));
        assert!(TaskClass::Research.retrieval().1);
    }
    #[test]
    fn no_turn_injection_for_code() {
        assert!(!TaskClass::Code.turn_injection());
        assert!(!TaskClass::Plan.turn_injection());
    }
    #[test]
    fn keeper_is_an_agent() {
        assert!(Keeper::is_agent_definition());
    }
    #[test]
    fn keeper_triggers_on_idle() {
        assert!(Keeper::triggers_on_idle(0, 0));
        assert!(!Keeper::triggers_on_idle(1, 0));
    }
    #[test]
    fn keeper_pass() {
        let mut store = DurableMemoryStore::default();
        store.insert(entry(ProjectId::generate(), 1, "p")).unwrap();
        Keeper::pass(&mut store);
        assert!(store.get("m1").is_some());
    }
    #[test]
    fn primary_cannot_edit() {
        use crate::services::agents::MemoryWrite;
        assert_eq!(MemoryWrite::Propose, MemoryWrite::default());
        assert_ne!(MemoryWrite::Propose, MemoryWrite::Direct);
    }
    #[test]
    fn empty_store_degrades_cleanly() {
        let mut store = DurableMemoryStore::default();
        assert!(recall(
            &mut store,
            ProjectId::generate(),
            None,
            "q",
            &[],
            TaskClass::Code
        )
        .is_empty());
        assert!(build_catalog(&[]).text.is_empty());
    }
    #[test]
    fn cross_harness_recall() {
        let project = ProjectId::generate();
        let mut store = DurableMemoryStore::default();
        store.insert(entry(project, 1, "shared")).unwrap();
        assert!(!recall(&mut store, project, None, "one", &[1.0], TaskClass::Code).is_empty());
    }
    #[test]
    fn carve_outs_declared() {
        assert!(MEMORY_CARVE_OUTS.contains(&"embedding"));
        assert!(MEMORY_CARVE_OUTS.contains(&"strength"));
    }
    #[test]
    fn memory_foldable() {
        let value = entry(ProjectId::generate(), 1, "p");
        assert!(!value.path.is_empty() && !value.provenance.is_null());
    }
    #[test]
    fn decay_read_time_no_ticks() {
        let mut value = entry(ProjectId::generate(), 1, "p");
        value.active_days = 5;
        let before = value.active_days;
        let _ = value.decay_strength();
        assert_eq!(value.active_days, before);
    }
    #[test]
    fn memory_vectors_untouched() {
        let value = entry(ProjectId::generate(), 1, "p");
        let vector = value.embedding.clone();
        let _ = value.decay_strength();
        assert_eq!(value.embedding, vector);
    }
}

#[cfg(test)]
mod project {
    #[test]
    fn memory_foldable() {
        assert!(super::MEMORY_CARVE_OUTS.contains(&"embedding"));
    }
}

#[cfg(test)]
mod rebuild {
    #[test]
    fn memory_vectors_untouched() {
        let project = super::ProjectId::generate();
        let value = super::MemoryEntry::new(
            "rebuild",
            project,
            super::MemoryScopeKind::Project,
            None,
            "p",
            "s",
            super::MemoryCategory::Fact,
            "body",
            serde_json::json!({}),
            vec![1.0],
            "test",
            0.5,
        )
        .unwrap();
        assert_eq!(value.embedding, vec![1.0]);
    }
}
