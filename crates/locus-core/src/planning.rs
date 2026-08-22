//! Editable plan decomposition drafts, before final approval creates board cards.

use anyhow::{bail, Result};

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
}

impl PlanTask {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
        }
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

/// An editable spec/task-to-card mapping. It deliberately contains no board-card id.
/// Task 20 owns final approval, persistence, and board-card creation.
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
