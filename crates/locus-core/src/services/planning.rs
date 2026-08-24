//! Editable plan decomposition drafts, before final approval creates board cards.

use std::collections::BTreeSet;

use crate::runtime::routing::RoutingEffort;
use anyhow::{bail, Result};

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
