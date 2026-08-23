//! Editable plan decomposition drafts, before final approval creates board cards.

use anyhow::{bail, Result};

/// The ordered durable states of a planning conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanningStage {
    Inputs,
    Orient,
    Converse,
    Synthesise,
    Audit,
    Recommend,
    Override,
    Decompose,
    Approve,
}

impl PlanningStage {
    pub const ALL: [Self; 9] = [
        Self::Inputs, Self::Orient, Self::Converse, Self::Synthesise, Self::Audit,
        Self::Recommend, Self::Override, Self::Decompose, Self::Approve,
    ];

    pub fn next(self) -> Option<Self> {
        let index = Self::ALL.iter().position(|stage| *stage == self)?;
        Self::ALL.get(index + 1).copied()
    }
}

/// A stable requirement id and editable specification block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub id: String,
    pub body: String,
}

impl Requirement {
    pub fn new(id: impl Into<String>, body: impl Into<String>) -> Self {
        Self { id: id.into(), body: body.into() }
    }
}

/// A draft specification that retains requirement identities across edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableSpec {
    requirements: Vec<Requirement>,
}

impl EditableSpec {
    pub fn new(requirements: impl IntoIterator<Item = Requirement>) -> Result<Self> {
        let spec = Self { requirements: requirements.into_iter().collect() };
        if spec.requirements.iter().any(|requirement| requirement.id.trim().is_empty() || requirement.body.trim().is_empty())
            || spec.requirements.iter().enumerate().any(|(index, requirement)| spec.requirements[..index].iter().any(|other| other.id == requirement.id))
        {
            bail!("requirements need unique nonempty ids and bodies");
        }
        Ok(spec)
    }

    pub fn edit(&mut self, id: &str, body: impl Into<String>) -> Result<()> {
        let requirement = self.requirements.iter_mut().find(|requirement| requirement.id == id)
            .ok_or_else(|| anyhow::anyhow!("requirement `{id}` does not exist"))?;
        let body = body.into();
        if body.trim().is_empty() { bail!("requirement body must not be empty"); }
        requirement.body = body;
        Ok(())
    }

    pub fn requirement(&self, id: &str) -> Option<&Requirement> {
        self.requirements.iter().find(|requirement| requirement.id == id)
    }
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

/// A task in an approved plan that can become its own board-card draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTask {
    pub id: String,
    pub title: String,
    pub dependencies: Vec<String>,
}

impl PlanTask {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            dependencies: Vec::new(),
        }
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
