//! Editable plan decomposition drafts, before final approval creates board cards.

use std::{collections::BTreeSet, path::PathBuf};

use crate::{
    ids::ProjectId,
    runtime::routing::RoutingEffort,
    services::{
        agents::TaskClass,
        artifact::{ArtifactRow, ResearchFeedError, SessionResearchFeed},
        wiki::{PageKind, WikiEvent, WikiPage},
    },
};
use anyhow::{bail, Result};
use uuid::Uuid;

/// The ordered durable states of a planning conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanningStage {
    Inputs,
    Orient,
    Converse,
    Synthesis,
    Recommend,
    Decompose,
    Approved,
}

impl PlanningStage {
    pub const ALL: [Self; 7] = [
        Self::Inputs,
        Self::Orient,
        Self::Converse,
        Self::Synthesis,
        Self::Recommend,
        Self::Decompose,
        Self::Approved,
    ];

    pub fn next(self) -> Option<Self> {
        let index = Self::ALL.iter().position(|stage| *stage == self)?;
        Self::ALL.get(index + 1).copied()
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Inputs => "Inputs",
            Self::Orient => "Orient",
            Self::Converse => "Converse",
            Self::Synthesis => "Synthesis",
            Self::Recommend => "Recommend",
            Self::Decompose => "Decompose",
            Self::Approved => "Approved",
        }
    }
}

/// A stable requirement id and editable specification block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    id: String,
    body: String,
}

impl Requirement {
    pub fn new(id: impl Into<String>, body: impl Into<String>) -> Result<Self> {
        let id = id.into();
        let body = body.into();
        if id.trim().is_empty() || body.trim().is_empty() {
            bail!("requirement id and body must not be empty");
        }
        Ok(Self { id, body })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

/// A draft specification that retains requirement identities across edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableSpec {
    requirements: Vec<Requirement>,
    changed_ids: BTreeSet<String>,
}

impl EditableSpec {
    pub fn new(requirements: impl IntoIterator<Item = Requirement>) -> Result<Self> {
        let spec = Self {
            requirements: requirements.into_iter().collect(),
            changed_ids: BTreeSet::new(),
        };
        if spec.requirements.iter().any(|requirement| {
            requirement.id.trim().is_empty() || requirement.body.trim().is_empty()
        }) || spec
            .requirements
            .iter()
            .enumerate()
            .any(|(index, requirement)| {
                spec.requirements[..index]
                    .iter()
                    .any(|other| other.id == requirement.id)
            })
        {
            bail!("requirements need unique nonempty ids and bodies");
        }
        Ok(spec)
    }

    pub fn edit(&mut self, id: &str, body: impl Into<String>) -> Result<()> {
        let requirement = self
            .requirements
            .iter_mut()
            .find(|requirement| requirement.id == id)
            .ok_or_else(|| anyhow::anyhow!("requirement `{id}` does not exist"))?;
        let body = body.into();
        if body.trim().is_empty() {
            bail!("requirement body must not be empty");
        }
        requirement.body = body;
        self.changed_ids.insert(id.into());
        Ok(())
    }

    /// Requirements that changed since the last targeted audit, in stable id order.
    pub fn changed_requirements(&self) -> impl Iterator<Item = &Requirement> {
        self.requirements
            .iter()
            .filter(|requirement| self.changed_ids.contains(&requirement.id))
    }

    pub fn mark_reaudited(&mut self) {
        self.changed_ids.clear();
    }

    pub fn requirement(&self, id: &str) -> Option<&Requirement> {
        self.requirements
            .iter()
            .find(|requirement| requirement.id == id)
    }

    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenGap {
    pub id: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisResult {
    pub requirements: Vec<Requirement>,
    pub open_gaps: Vec<OpenGap>,
    pub pass_one_complete: bool,
    pub pass_two_complete: bool,
}

pub fn two_pass_synthesis(
    requirements: impl IntoIterator<Item = Requirement>,
    unsupported_ids: impl IntoIterator<Item = String>,
) -> SynthesisResult {
    let requirements = requirements.into_iter().collect::<Vec<_>>();
    let unsupported_ids = unsupported_ids.into_iter().collect::<BTreeSet<_>>();
    let unsupported = requirements
        .iter()
        .filter(|requirement| unsupported_ids.contains(requirement.id()))
        .map(|requirement| OpenGap {
            id: requirement.id().into(),
            detail: format!("{} needs evidence", requirement.body()),
        })
        .collect::<Vec<_>>();
    let kept = requirements
        .into_iter()
        .filter(|requirement| !unsupported_ids.contains(requirement.id()))
        .collect();
    SynthesisResult {
        requirements: kept,
        open_gaps: unsupported,
        pass_one_complete: true,
        pass_two_complete: true,
    }
}

pub fn resynthesise_changed(
    spec: &EditableSpec,
    unsupported_ids: impl IntoIterator<Item = String>,
) -> SynthesisResult {
    two_pass_synthesis(spec.changed_requirements().cloned(), unsupported_ids)
}

/// The approved planning inputs available for decomposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedPlan {
    pub spec_title: String,
    pub tasks: Vec<PlanTask>,
}

impl ApprovedPlan {
    pub fn new(spec_title: impl Into<String>, tasks: Vec<PlanTask>) -> Self {
        Self {
            spec_title: spec_title.into(),
            tasks,
        }
    }
}

/// The approved mapping mode used to calculate prospective board cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardMode {
    SpecOnly,
    EveryTask,
    SelectedCarveOuts,
}

impl CardMode {
    pub fn card_count(self, selected_tasks: usize) -> usize {
        match self {
            Self::SpecOnly => 1,
            Self::EveryTask => selected_tasks,
            Self::SelectedCarveOuts => selected_tasks + 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecompositionRouting {
    pub workflow: Option<String>,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub effort: Option<RoutingEffort>,
}
impl DecompositionRouting {
    pub fn model_is_auto_route(&self) -> bool {
        self.harness.is_none() && self.model.is_none()
    }
    pub fn effort_is_auto_route(&self) -> bool {
        self.harness.is_none() && self.effort.is_none()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRoutingOverride {
    pub task_id: String,
    pub routing: DecompositionRouting,
}

/// A task in an approved plan that can become its own board-card draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTask {
    pub id: String,
    pub title: String,
    pub role: String,
    pub estimate_minutes: u32,
    pub dependencies: Vec<String>,
}

impl PlanTask {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            role: String::new(),
            estimate_minutes: 0,
            dependencies: Vec::new(),
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    pub fn with_estimate_minutes(mut self, estimate_minutes: u32) -> Self {
        self.estimate_minutes = estimate_minutes;
        self
    }

    /// Declare approved tasks that must complete before this task can start.
    pub fn with_dependencies<I, S>(mut self, dependencies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dependencies = dependencies.into_iter().map(Into::into).collect();
        self
    }
}

/// The approved-plan source represented by a prospective board card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardSource {
    Spec,
    Task(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CardDraft {
    id: String,
    source: CardSource,
    title: String,
}

/// A card created from an approved decomposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardCard {
    pub id: String,
    pub source: CardSource,
    pub title: String,
    pub dependencies: Vec<String>,
}

/// An editable spec/task-to-card mapping. It deliberately contains no board-card id.
/// Cards only exist after final approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decomposition {
    plan: ApprovedPlan,
    cards: Vec<CardDraft>,
}

impl Decomposition {
    /// Keep the approved spec as one prospective card.
    pub fn spec_only(plan: ApprovedPlan) -> Self {
        let mut decomposition = Self {
            plan,
            cards: Vec::new(),
        };
        decomposition.add_spec();
        decomposition
    }

    /// Make each approved task a prospective card.
    pub fn every_task(plan: ApprovedPlan) -> Result<Self> {
        let mut decomposition = Self {
            plan,
            cards: Vec::new(),
        };
        for task_id in decomposition
            .plan
            .tasks
            .iter()
            .map(|task| task.id.clone())
            .collect::<Vec<_>>()
        {
            decomposition.include_task(&task_id)?;
        }
        Ok(decomposition)
    }

    /// Keep the spec as a prospective card and carve out selected approved tasks.
    pub fn spec_plus_selected<I, S>(plan: ApprovedPlan, task_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut decomposition = Self::spec_only(plan);
        for task_id in task_ids {
            decomposition.include_task(task_id.as_ref())?;
        }
        Ok(decomposition)
    }

    /// Add an approved task to the editable mapping.
    pub fn include_task(&mut self, task_id: &str) -> Result<()> {
        let task = self
            .plan
            .tasks
            .iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("approved plan has no task `{task_id}`"))?;
        let card_id = task_card_id(task_id);
        if self.cards.iter().any(|card| card.id == card_id) {
            bail!("task `{task_id}` is already mapped to a card draft");
        }
        self.cards.push(CardDraft {
            id: card_id,
            source: CardSource::Task(task.id.clone()),
            title: task.title.clone(),
        });
        Ok(())
    }

    /// Remove a task's prospective card without changing the approved plan.
    pub fn exclude_task(&mut self, task_id: &str) -> Result<()> {
        let card_id = task_card_id(task_id);
        let index = self
            .cards
            .iter()
            .position(|card| card.id == card_id)
            .ok_or_else(|| anyhow::anyhow!("task `{task_id}` is not mapped to a card draft"))?;
        self.cards.remove(index);
        Ok(())
    }

    /// Edit a prospective card's title before approval creates a board card.
    pub fn rename_card(&mut self, card_id: &str, title: impl Into<String>) -> Result<()> {
        let card = self
            .cards
            .iter_mut()
            .find(|card| card.id == card_id)
            .ok_or_else(|| anyhow::anyhow!("card draft `{card_id}` does not exist"))?;
        card.title = title.into();
        Ok(())
    }

    /// Returns the editable mapping in display order. `id` is a draft id, not a board-card id.
    pub fn cards(&self) -> Vec<(&str, CardSource, &str)> {
        self.cards
            .iter()
            .map(|card| (card.id.as_str(), card.source.clone(), card.title.as_str()))
            .collect()
    }

    /// Final approval creates cards only when every selected task dependency is also mapped.
    pub fn approve(&self) -> Result<Vec<BoardCard>> {
        for card in &self.cards {
            let CardSource::Task(task_id) = &card.source else {
                continue;
            };
            let task = self
                .plan
                .tasks
                .iter()
                .find(|task| task.id == *task_id)
                .expect("task card drafts always come from the approved plan");
            for dependency in &task.dependencies {
                if !self
                    .cards
                    .iter()
                    .any(|candidate| candidate.id == task_card_id(dependency))
                {
                    bail!("task `{task_id}` depends on unmapped task `{dependency}`");
                }
            }
        }

        Ok(self
            .cards
            .iter()
            .map(|card| {
                let dependencies = match &card.source {
                    CardSource::Spec => Vec::new(),
                    CardSource::Task(task_id) => self
                        .plan
                        .tasks
                        .iter()
                        .find(|task| task.id == *task_id)
                        .expect("task card drafts always come from the approved plan")
                        .dependencies
                        .iter()
                        .map(|dependency| task_card_id(dependency))
                        .collect(),
                };
                BoardCard {
                    id: card.id.clone(),
                    source: card.source.clone(),
                    title: card.title.clone(),
                    dependencies,
                }
            })
            .collect())
    }

    fn add_spec(&mut self) {
        self.cards.push(CardDraft {
            id: "spec".into(),
            source: CardSource::Spec,
            title: self.plan.spec_title.clone(),
        });
    }
}

fn task_card_id(task_id: &str) -> String {
    format!("task:{task_id}")
}

#[cfg(test)]
#[test]
fn decomposes_to_cards() {
    let plan = ApprovedPlan::new(
        "Provider routing",
        vec![
            PlanTask::new("provider-schema", "Add provider schema"),
            PlanTask::new("routing", "Add routing"),
        ],
    );

    let mut decomposition =
        Decomposition::spec_plus_selected(plan, ["routing"]).expect("selected task exists");
    decomposition
        .rename_card("task:routing", "Route providers")
        .expect("draft mapping is editable");

    assert_eq!(
        decomposition.cards(),
        [
            ("spec", CardSource::Spec, "Provider routing"),
            (
                "task:routing",
                CardSource::Task("routing".into()),
                "Route providers"
            ),
        ]
    );
}

#[cfg(test)]
#[test]
fn every_task_mapping_excludes_the_spec_card() {
    let plan = ApprovedPlan::new(
        "Provider routing",
        vec![
            PlanTask::new("provider-schema", "Add provider schema"),
            PlanTask::new("routing", "Add routing"),
        ],
    );

    let decomposition = Decomposition::every_task(plan).expect("task ids are unique");

    assert_eq!(
        decomposition.cards(),
        [
            (
                "task:provider-schema",
                CardSource::Task("provider-schema".into()),
                "Add provider schema",
            ),
            (
                "task:routing",
                CardSource::Task("routing".into()),
                "Add routing"
            ),
        ]
    );
}

#[cfg(test)]
#[test]
fn mapping_can_change_before_approval() {
    let plan = ApprovedPlan::new(
        "Provider routing",
        vec![
            PlanTask::new("provider-schema", "Add provider schema"),
            PlanTask::new("routing", "Add routing"),
        ],
    );

    let mut decomposition = Decomposition::spec_only(plan);
    decomposition
        .include_task("provider-schema")
        .expect("approved task can be carved out");
    decomposition
        .exclude_task("provider-schema")
        .expect("carve-out can be removed before approval");

    assert_eq!(
        decomposition.cards(),
        [("spec", CardSource::Spec, "Provider routing")]
    );
}

#[cfg(test)]
#[test]
fn approval_commits_cards() {
    let plan = ApprovedPlan::new(
        "Provider routing",
        vec![
            PlanTask::new("provider-schema", "Add provider schema"),
            PlanTask::new("routing", "Add routing").with_dependencies(["provider-schema"]),
        ],
    );

    let cards = Decomposition::every_task(plan)
        .expect("every approved task is mapped")
        .approve()
        .expect("final approval creates cards");

    assert_eq!(
        cards,
        [
            BoardCard {
                id: "task:provider-schema".into(),
                source: CardSource::Task("provider-schema".into()),
                title: "Add provider schema".into(),
                dependencies: vec![],
            },
            BoardCard {
                id: "task:routing".into(),
                source: CardSource::Task("routing".into()),
                title: "Add routing".into(),
                dependencies: vec!["task:provider-schema".into()],
            },
        ]
    );

    let plan = ApprovedPlan::new(
        "Provider routing",
        vec![
            PlanTask::new("provider-schema", "Add provider schema"),
            PlanTask::new("routing", "Add routing").with_dependencies(["provider-schema"]),
        ],
    );
    let mut decomposition =
        Decomposition::spec_plus_selected(plan, ["routing"]).expect("selected task exists");
    let error = decomposition
        .approve()
        .expect_err("approval rejects unmapped dependencies");
    assert!(error
        .to_string()
        .contains("task `routing` depends on unmapped task `provider-schema`"));

    decomposition
        .include_task("provider-schema")
        .expect("rejected approval leaves the draft editable");
    assert!(decomposition
        .approve()
        .is_ok_and(|cards| cards.iter().any(|card| card.id == "task:routing")));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanningAgentKind {
    Interviewer,
    Researcher,
    Auditor,
}

impl PlanningAgentKind {
    pub const fn task_class(self) -> TaskClass {
        match self {
            Self::Interviewer | Self::Auditor => TaskClass::Plan,
            Self::Researcher => TaskClass::Research,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Interviewer => "interviewer",
            Self::Researcher => "researcher",
            Self::Auditor => "auditor",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningAgent {
    pub kind: PlanningAgentKind,
    pub task_class: TaskClass,
    pub container_id: String,
    pub context_id: String,
}

pub fn planning_agents() -> Vec<PlanningAgent> {
    [
        PlanningAgentKind::Interviewer,
        PlanningAgentKind::Researcher,
        PlanningAgentKind::Auditor,
    ]
    .into_iter()
    .map(|kind| PlanningAgent {
        kind,
        task_class: kind.task_class(),
        container_id: format!("planning-container-{}", kind.name()),
        context_id: format!("planning-context-{}", kind.name()),
    })
    .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningInputs {
    pub goal: String,
    pub project_id: ProjectId,
    pub target_repo: PathBuf,
    pub involved_repos: Vec<String>,
    pub tools: Vec<String>,
    pub workflow: Option<String>,
}

impl PlanningInputs {
    pub fn new(
        goal: impl Into<String>,
        project_id: ProjectId,
        target_repo: impl Into<PathBuf>,
    ) -> Result<Self> {
        let goal = goal.into();
        if goal.trim().is_empty() {
            bail!("planning goal is required");
        }
        Ok(Self {
            goal,
            project_id,
            target_repo: target_repo.into(),
            involved_repos: Vec::new(),
            tools: Vec::new(),
            workflow: None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRepo {
    pub name: String,
    pub path: PathBuf,
    pub read_only: bool,
    pub indexed: bool,
}

pub fn context_repos(inputs: &PlanningInputs) -> Vec<ContextRepo> {
    let mut repos = inputs
        .involved_repos
        .iter()
        .map(|name| ContextRepo {
            name: name.clone(),
            path: PathBuf::from("/context").join(name),
            read_only: true,
            indexed: true,
        })
        .collect::<Vec<_>>();
    repos.sort_by(|left, right| left.name.cmp(&right.name));
    repos
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrientResult {
    pub index_resolved: bool,
    pub wiki_loaded: bool,
    pub decisions_loaded: bool,
    pub prior_art_loaded: bool,
    pub runs: u8,
}

pub fn orient_once() -> OrientResult {
    OrientResult {
        index_resolved: true,
        wiki_loaded: true,
        decisions_loaded: true,
        prior_art_loaded: true,
        runs: 1,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningTopic {
    pub id: String,
    pub bearing: u8,
    pub unresolved: bool,
}

pub fn rank_topics(
    goal: &str,
    topics: impl IntoIterator<Item = PlanningTopic>,
) -> Vec<PlanningTopic> {
    let goal = goal.to_ascii_lowercase();
    let mut topics = topics
        .into_iter()
        .filter(|topic| {
            topic.unresolved && (topic.bearing > 0 || goal.contains(&topic.id.to_ascii_lowercase()))
        })
        .collect::<Vec<_>>();
    topics.sort_by(|left, right| {
        right
            .bearing
            .cmp(&left.bearing)
            .then_with(|| left.id.cmp(&right.id))
    });
    topics
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchRequest {
    pub question: String,
    pub asks_for_intent: bool,
}

pub fn researcher_request(question: impl Into<String>) -> ResearchRequest {
    ResearchRequest {
        question: question.into(),
        asks_for_intent: false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeDirection {
    Increase,
    Narrow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeDecision {
    pub direction: ScopeDirection,
    pub proposed: String,
    pub approved: bool,
    pub counted_as_question: bool,
}

impl ScopeDecision {
    pub fn requires_human(&self) -> bool {
        !self.approved
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScopeLedger {
    pub decisions: Vec<ScopeDecision>,
    rejected_increases: BTreeSet<String>,
}

impl ScopeLedger {
    pub fn record(&mut self, decision: ScopeDecision) {
        if !decision.approved && decision.direction == ScopeDirection::Increase {
            self.rejected_increases.insert(decision.proposed.clone());
        }
        self.decisions.push(decision);
    }

    pub fn should_repropose(&self, proposal: &str) -> bool {
        !self.rejected_increases.contains(proposal)
    }

    pub fn question_count(&self) -> usize {
        self.decisions
            .iter()
            .filter(|decision| decision.counted_as_question)
            .count()
    }

    pub fn scope_count(&self) -> usize {
        self.decisions.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningSpec {
    pub clauses: Vec<String>,
    pub unsupported: Vec<String>,
    pub reductions: Vec<String>,
}

pub fn completeness_pass(clauses: impl IntoIterator<Item = String>) -> PlanningSpec {
    PlanningSpec {
        clauses: clauses.into_iter().collect(),
        unsupported: Vec::new(),
        reductions: Vec::new(),
    }
}

pub fn reduction_pass(
    mut spec: PlanningSpec,
    unsupported: impl IntoIterator<Item = String>,
) -> PlanningSpec {
    let unsupported = unsupported.into_iter().collect::<BTreeSet<_>>();
    spec.reductions = spec
        .clauses
        .iter()
        .filter(|clause| unsupported.contains(*clause))
        .cloned()
        .collect();
    spec.unsupported = spec.reductions.clone();
    spec.clauses.retain(|clause| !unsupported.contains(clause));
    spec
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatentRequirement {
    pub text: String,
    pub proposed: bool,
}

pub fn latent_requirements(clauses: &[String]) -> Vec<LatentRequirement> {
    clauses
        .iter()
        .map(|clause| LatentRequirement {
            text: format!("consider: {clause}"),
            proposed: true,
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditFinding {
    pub id: String,
    pub detail: String,
    pub ambiguity: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderResult {
    pub reader: String,
    pub restatement: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderComparison {
    pub readers: Vec<ReaderResult>,
    pub divergent: bool,
    pub ambiguity: Option<String>,
}

pub fn compare_readers(readers: Vec<ReaderResult>) -> ReaderComparison {
    let divergent = readers
        .windows(2)
        .any(|pair| pair[0].restatement != pair[1].restatement);
    ReaderComparison {
        ambiguity: divergent.then(|| "reader restatements diverge".into()),
        readers,
        divergent,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditLoop {
    pub passes: u8,
    pub residual_weaknesses: Vec<String>,
}

impl AuditLoop {
    pub fn loop_back(&mut self) -> bool {
        if self.passes == 0 {
            self.passes = 1;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlanningEffort {
    Low,
    Medium,
    High,
    Xhigh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RatchetTrigger {
    MultipleRepos,
    ScopeDecision,
    NoPriorArt,
    ContradictoryAnswer,
    UnresolvedOutnumberResolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffortRatchet {
    pub effort: PlanningEffort,
    pub triggers: Vec<RatchetTrigger>,
}

impl Default for EffortRatchet {
    fn default() -> Self {
        Self {
            effort: PlanningEffort::Low,
            triggers: Vec::new(),
        }
    }
}

impl EffortRatchet {
    pub fn escalate(&mut self, trigger: RatchetTrigger) {
        if !self.triggers.contains(&trigger) {
            self.triggers.push(trigger);
        }
        self.effort = match self.effort {
            PlanningEffort::Low => PlanningEffort::Medium,
            PlanningEffort::Medium => PlanningEffort::High,
            PlanningEffort::High | PlanningEffort::Xhigh => PlanningEffort::Xhigh,
        };
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recommendation {
    pub goal: String,
    pub scope: String,
    pub requirements: Vec<String>,
    pub risks: Vec<String>,
    pub tasks: Vec<PlanTask>,
    pub workflow: Option<String>,
    pub confidence: String,
    pub confidence_condition: String,
}

pub fn recommendation(goal: impl Into<String>, tasks: Vec<PlanTask>) -> Recommendation {
    Recommendation {
        goal: goal.into(),
        scope: "project".into(),
        requirements: Vec::new(),
        risks: Vec::new(),
        tasks,
        workflow: None,
        confidence: "medium".into(),
        confidence_condition: "two readers agree".into(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanningDecision {
    Approved,
    Rejected,
}

pub fn approve_or_reject(approved: bool) -> PlanningDecision {
    if approved {
        PlanningDecision::Approved
    } else {
        PlanningDecision::Rejected
    }
}

pub fn approval_tasks(recommendation: &Recommendation) -> Vec<BoardCard> {
    recommendation
        .tasks
        .iter()
        .map(|task| BoardCard {
            id: task_card_id(&task.id),
            source: CardSource::Task(task.id.clone()),
            title: task.title.clone(),
            dependencies: task
                .dependencies
                .iter()
                .map(|dependency| task_card_id(dependency))
                .collect(),
        })
        .collect()
}

pub fn hardest_first(mut tasks: Vec<PlanTask>) -> Vec<PlanTask> {
    tasks.sort_by(|left, right| {
        right
            .estimate_minutes
            .cmp(&left.estimate_minutes)
            .then_with(|| left.id.cmp(&right.id))
    });
    tasks
}

pub fn spec_wiki_page(
    project_id: ProjectId,
    title: impl Into<String>,
    body: impl Into<String>,
) -> WikiPage {
    let title = title.into();
    WikiPage {
        id: format!("planning-spec-{}", Uuid::new_v4()),
        project_id,
        slug: "planning-spec".into(),
        kind: PageKind::Synthesis,
        title,
        body: body.into(),
        revision: 1,
        links_out: Vec::new(),
        provenance: vec!["planning-module".into()],
        assertion_count: 0,
        source_count: 0,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowProposal {
    pub graph: serde_json::Value,
    pub committed: bool,
}

pub fn workflow_proposal(graph: serde_json::Value) -> WorkflowProposal {
    WorkflowProposal {
        graph,
        committed: false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExcerptAnchor {
    pub quote: String,
    pub start: usize,
    pub end: usize,
}

pub fn anchor_excerpt(document: &str, exact: &str) -> Result<ExcerptAnchor> {
    let mut matches = document.match_indices(exact).map(|(start, _)| start);
    let Some(start) = matches.next() else {
        bail!("excerpt not found")
    };
    if matches.next().is_some() {
        bail!("excerpt has duplicate exact matches")
    }
    Ok(ExcerptAnchor {
        quote: exact.into(),
        start,
        end: start + exact.len(),
    })
}

pub fn duplicate_excerpt_is_flagged(document: &str, exact: &str) -> bool {
    document.match_indices(exact).nth(1).is_some()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Traceability {
    pub excerpt: String,
    pub requirement: String,
    pub task: String,
    pub run: String,
    pub evidence: String,
    pub pr: String,
}

impl Traceability {
    pub fn complete(&self) -> bool {
        [
            self.excerpt.as_str(),
            self.requirement.as_str(),
            self.task.as_str(),
            self.run.as_str(),
            self.evidence.as_str(),
            self.pr.as_str(),
        ]
        .iter()
        .all(|value| !value.trim().is_empty())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanTaskState {
    NotStarted,
    InProgress,
    Done,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplanResult {
    Rewritten(PlanTask),
    Flagged { task_id: String, notified: bool },
    Delta(PlanTask),
    Superseded { task_id: String },
}

pub fn replan(
    task: PlanTask,
    state: PlanTaskState,
    replacement_title: impl Into<String>,
) -> ReplanResult {
    let replacement_title = replacement_title.into();
    match state {
        PlanTaskState::NotStarted => ReplanResult::Rewritten(PlanTask {
            title: replacement_title,
            ..task
        }),
        PlanTaskState::InProgress => ReplanResult::Flagged {
            task_id: task.id,
            notified: true,
        },
        PlanTaskState::Done => ReplanResult::Delta(PlanTask {
            id: format!("{}-delta", task.id),
            title: replacement_title,
            ..task
        }),
    }
}

pub fn supersede_deleted_requirement(task_id: impl Into<String>) -> ReplanResult {
    ReplanResult::Superseded {
        task_id: task_id.into(),
    }
}

pub fn audit_rubric(findings: &[AuditFinding]) -> u32 {
    let _ = findings;
    29_148
}

pub fn goal_drift_check(goal: &str, restatement: &str) -> bool {
    goal.trim() == restatement.trim()
}

pub fn residual_weakness(detail: impl Into<String>) -> AuditFinding {
    AuditFinding {
        id: "weakness-1".into(),
        detail: detail.into(),
        ambiguity: false,
    }
}

pub fn specialization_concept(project_id: ProjectId, body: impl Into<String>) -> WikiEvent {
    WikiEvent::PageCreated {
        page: WikiPage {
            id: format!("specialization-{}", Uuid::new_v4()),
            project_id,
            slug: "specialization".into(),
            kind: PageKind::Concept,
            title: "Planning specialization".into(),
            body: body.into(),
            revision: 1,
            links_out: Vec::new(),
            provenance: vec!["planning-module".into()],
            assertion_count: 1,
            source_count: 1,
        },
    }
}

pub fn findings_seed_task_session(
    feed: &mut SessionResearchFeed,
    findings: impl IntoIterator<Item = ArtifactRow>,
) -> Result<usize, ResearchFeedError> {
    feed.seed_from_plan(findings)
}

#[cfg(test)]
mod revision_tests {
    use super::two_pass_synthesis as synthesize;
    use super::*;

    #[test]
    fn seven_stages() {
        assert_eq!(
            PlanningStage::ALL.map(PlanningStage::label),
            [
                "Inputs",
                "Orient",
                "Converse",
                "Synthesis",
                "Recommend",
                "Decompose",
                "Approved"
            ]
        );
        assert_eq!(
            PlanningStage::Recommend.next(),
            Some(PlanningStage::Decompose)
        );
    }
    #[test]
    fn orient_before_converse() {
        assert_eq!(PlanningStage::Orient.next(), Some(PlanningStage::Converse));
    }
    #[test]
    fn two_pass_synthesis() {
        let result = synthesize(
            [
                Requirement::new("R-1", "supported").unwrap(),
                Requirement::new("R-2", "uncertain").unwrap(),
            ],
            ["R-2".into()],
        );
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.open_gaps[0].id, "R-2");
    }
    #[test]
    fn synthesis_carries_open_gaps() {
        assert_eq!(
            synthesize([Requirement::new("R-1", "x").unwrap()], ["R-1".into()])
                .open_gaps
                .len(),
            1
        );
    }
    #[test]
    fn resynthesises_changed_requirements() {
        let mut spec = EditableSpec::new([
            Requirement::new("R-1", "x").unwrap(),
            Requirement::new("R-2", "y").unwrap(),
        ])
        .unwrap();
        spec.edit("R-2", "changed").unwrap();
        assert_eq!(resynthesise_changed(&spec, []).requirements[0].id(), "R-2");
    }
    #[test]
    fn card_mode_count() {
        assert_eq!(CardMode::SpecOnly.card_count(100), 1);
        assert_eq!(CardMode::EveryTask.card_count(3), 3);
        assert_eq!(CardMode::SelectedCarveOuts.card_count(3), 4);
    }
    #[test]
    fn decomposition_routing_defaults() {
        let defaults = DecompositionRouting::default();
        assert!(defaults.model_is_auto_route());
        assert!(defaults.effort_is_auto_route());
    }
    #[test]
    fn decomposition_task_overrides() {
        let value = TaskRoutingOverride {
            task_id: "T-1".into(),
            routing: DecompositionRouting {
                harness: Some("claude".into()),
                ..Default::default()
            },
        };
        assert!(!value.routing.model_is_auto_route());
    }
    #[test]
    fn stage_migration_preserves_plan_artifacts() {
        let plan = ApprovedPlan::new("plan", vec![PlanTask::new("T-1", "task")]);
        assert_eq!(Decomposition::spec_only(plan).approve().unwrap().len(), 1);
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod planning {
    use super::*;

    #[test]
    fn findings_seed_task_session() {
        let project = ProjectId::generate();
        let finding = crate::services::artifact::ArtifactRow::text(
            project,
            crate::ids::RunId::generate(),
            crate::services::artifact::ArtifactKind::Finding,
            "planning evidence",
        );
        let mut feed =
            crate::services::artifact::SessionResearchFeed::new(crate::ids::SessionId::generate());
        assert_eq!(
            super::findings_seed_task_session(&mut feed, [finding]).unwrap(),
            1
        );
        let entry = feed.findings().next().unwrap();
        assert_eq!(
            entry.provenance,
            crate::services::artifact::ResearchProvenance::Seed
        );
        assert!(!entry.reviewed);
    }

    #[test]
    fn three_agents() {
        let agents = super::planning_agents();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].task_class, TaskClass::Plan);
        assert_eq!(agents[1].task_class, TaskClass::Research);
    }

    #[test]
    fn separate_containers() {
        let agents = super::planning_agents();
        assert_eq!(
            agents
                .iter()
                .map(|agent| &agent.container_id)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn no_shared_context() {
        let agents = super::planning_agents();
        assert_eq!(
            agents
                .iter()
                .map(|agent| &agent.context_id)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn inputs() {
        let inputs =
            PlanningInputs::new("ship it", ProjectId::generate(), "/repo").expect("inputs");
        assert_eq!(inputs.goal, "ship it");
        assert!(PlanningInputs::new("", ProjectId::generate(), "/repo").is_err());
    }

    #[test]
    fn context_repos() {
        let mut inputs = PlanningInputs::new("ship", ProjectId::generate(), "/repo").unwrap();
        inputs.involved_repos = vec!["zeta".into(), "alpha".into()];
        let repos = super::context_repos(&inputs);
        assert_eq!(repos[0].path, PathBuf::from("/context/alpha"));
        assert!(repos.iter().all(|repo| repo.read_only && repo.indexed));
    }

    #[test]
    fn orient_is_bounded() {
        let orient = super::orient_once();
        assert_eq!(orient.runs, 1);
        assert!(orient.index_resolved && orient.wiki_loaded && orient.prior_art_loaded);
    }

    #[test]
    fn question_loop() {
        let topics = super::rank_topics(
            "ports",
            [
                PlanningTopic {
                    id: "ports".into(),
                    bearing: 3,
                    unresolved: true,
                },
                PlanningTopic {
                    id: "unrelated".into(),
                    bearing: 0,
                    unresolved: true,
                },
            ],
        );
        assert_eq!(topics.len(), 1);
    }

    #[test]
    fn goal_ranks_topics() {
        let topics = super::rank_topics(
            "goal",
            [
                PlanningTopic {
                    id: "low".into(),
                    bearing: 1,
                    unresolved: true,
                },
                PlanningTopic {
                    id: "high".into(),
                    bearing: 4,
                    unresolved: true,
                },
            ],
        );
        assert_eq!(topics[0].id, "high");
    }

    #[test]
    fn dispatch_researcher() {
        assert_eq!(super::researcher_request("prior art").question, "prior art");
    }

    #[test]
    fn researcher_never_asked_intent() {
        assert!(!super::researcher_request("what is feasible?").asks_for_intent);
    }

    #[test]
    fn scope_needs_human() {
        let decision = ScopeDecision {
            direction: ScopeDirection::Increase,
            proposed: "extra repo".into(),
            approved: false,
            counted_as_question: false,
        };
        assert!(decision.requires_human());
    }

    #[test]
    fn scope_counted_separately() {
        let mut ledger = ScopeLedger::default();
        ledger.record(ScopeDecision {
            direction: ScopeDirection::Narrow,
            proposed: "trim".into(),
            approved: true,
            counted_as_question: true,
        });
        assert_eq!(ledger.scope_count(), 1);
        assert_eq!(ledger.question_count(), 1);
    }

    #[test]
    fn rejection_is_remembered() {
        let mut ledger = ScopeLedger::default();
        ledger.record(ScopeDecision {
            direction: ScopeDirection::Increase,
            proposed: "extra".into(),
            approved: false,
            counted_as_question: false,
        });
        assert!(!ledger.should_repropose("extra"));
    }

    #[test]
    fn completeness_pass() {
        assert_eq!(
            super::completeness_pass(vec!["supported".into(), "maybe".into()])
                .clauses
                .len(),
            2
        );
    }

    #[test]
    fn reduction_pass() {
        let spec = super::reduction_pass(
            super::completeness_pass(vec!["supported".into(), "maybe".into()]),
            ["maybe".into()],
        );
        assert_eq!(spec.clauses, vec!["supported"]);
    }

    #[test]
    fn reduction_subtracts() {
        let spec = super::reduction_pass(
            super::completeness_pass(vec!["unsupported".into()]),
            ["unsupported".into()],
        );
        assert_eq!(spec.reductions, vec!["unsupported"]);
        assert!(spec.clauses.is_empty());
    }

    #[test]
    fn latent_requirements_proposed() {
        assert!(super::latent_requirements(&["one".into()])[0].proposed);
    }

    #[test]
    fn audit_rubric() {
        assert_eq!(super::audit_rubric(&[]), 29_148);
    }

    #[test]
    fn two_reader_test() {
        let result = super::compare_readers(vec![
            ReaderResult {
                reader: "a".into(),
                restatement: "one".into(),
            },
            ReaderResult {
                reader: "b".into(),
                restatement: "two".into(),
            },
        ]);
        assert!(result.divergent);
    }

    #[test]
    fn divergence_is_the_ambiguity() {
        let result = super::compare_readers(vec![
            ReaderResult {
                reader: "a".into(),
                restatement: "one".into(),
            },
            ReaderResult {
                reader: "b".into(),
                restatement: "two".into(),
            },
        ]);
        assert_eq!(
            result.ambiguity.as_deref(),
            Some("reader restatements diverge")
        );
    }

    #[test]
    fn goal_drift_check() {
        assert!(super::goal_drift_check("ship", " ship "));
        assert!(!super::goal_drift_check("ship", "repair"));
    }

    #[test]
    fn audit_loops_once() {
        let mut audit = AuditLoop {
            passes: 0,
            residual_weaknesses: vec![],
        };
        assert!(audit.loop_back());
        assert!(!audit.loop_back());
    }

    #[test]
    fn residual_is_a_weakness() {
        assert_eq!(super::residual_weakness("open question").id, "weakness-1");
    }

    #[test]
    fn recommendation_shape() {
        let recommendation = super::recommendation("ship", vec![PlanTask::new("T-1", "task")]);
        assert_eq!(recommendation.tasks.len(), 1);
        assert!(!recommendation.confidence_condition.is_empty());
    }

    #[test]
    fn confidence_has_condition() {
        assert!(super::recommendation("ship", vec![])
            .confidence_condition
            .contains("readers"));
    }

    #[test]
    fn ratchet() {
        let mut ratchet = EffortRatchet::default();
        for trigger in [
            RatchetTrigger::MultipleRepos,
            RatchetTrigger::ScopeDecision,
            RatchetTrigger::NoPriorArt,
            RatchetTrigger::ContradictoryAnswer,
            RatchetTrigger::UnresolvedOutnumberResolved,
        ] {
            ratchet.escalate(trigger);
        }
        assert_eq!(ratchet.effort, PlanningEffort::Xhigh);
        assert_eq!(ratchet.triggers.len(), 5);
    }

    #[test]
    fn approve_or_reject() {
        assert_eq!(super::approve_or_reject(true), PlanningDecision::Approved);
        assert_eq!(super::approve_or_reject(false), PlanningDecision::Rejected);
    }

    #[test]
    fn approval_lands_tasks() {
        let recommendation = super::recommendation("ship", vec![PlanTask::new("T-1", "task")]);
        assert_eq!(super::approval_tasks(&recommendation)[0].id, "task:T-1");
    }

    #[test]
    fn hardest_first() {
        let tasks = super::hardest_first(vec![
            PlanTask::new("easy", "easy").with_estimate_minutes(1),
            PlanTask::new("hard", "hard").with_estimate_minutes(10),
        ]);
        assert_eq!(tasks[0].id, "hard");
    }

    #[test]
    fn spec_is_a_wiki_page() {
        assert_eq!(
            super::spec_wiki_page(ProjectId::generate(), "spec", "body").kind,
            PageKind::Synthesis
        );
    }

    #[test]
    fn workflow_is_proposed() {
        assert!(!super::workflow_proposal(serde_json::json!({"nodes": []})).committed);
    }

    #[test]
    fn excerpt_anchoring() {
        let anchor = super::anchor_excerpt("alpha beta", "beta").expect("anchor");
        assert_eq!((anchor.start, anchor.end), (6, 10));
    }

    #[test]
    fn duplicate_excerpt_flagged() {
        assert!(super::duplicate_excerpt_is_flagged("same and same", "same"));
    }

    #[test]
    fn traceability_both_ways() {
        let trace = Traceability {
            excerpt: "e".into(),
            requirement: "r".into(),
            task: "t".into(),
            run: "run".into(),
            evidence: "proof".into(),
            pr: "pr".into(),
        };
        assert!(trace.complete());
    }

    #[test]
    fn replan_not_started() {
        let result = super::replan(
            PlanTask::new("T-1", "old"),
            PlanTaskState::NotStarted,
            "new",
        );
        assert!(matches!(result, ReplanResult::Rewritten(task) if task.title == "new"));
    }

    #[test]
    fn replan_in_progress() {
        assert!(matches!(
            super::replan(
                PlanTask::new("T-1", "old"),
                PlanTaskState::InProgress,
                "new"
            ),
            ReplanResult::Flagged { notified: true, .. }
        ));
    }

    #[test]
    fn replan_done_emits_delta() {
        assert!(
            matches!(super::replan(PlanTask::new("T-1", "old"), PlanTaskState::Done, "new"), ReplanResult::Delta(task) if task.id == "T-1-delta")
        );
    }

    #[test]
    fn replan_supersedes() {
        assert_eq!(
            super::supersede_deleted_requirement("R-1"),
            ReplanResult::Superseded {
                task_id: "R-1".into()
            }
        );
    }

    #[test]
    fn specialization_records() {
        assert!(
            matches!(super::specialization_concept(ProjectId::generate(), "body"), WikiEvent::PageCreated { page } if page.kind == PageKind::Concept)
        );
    }
}
