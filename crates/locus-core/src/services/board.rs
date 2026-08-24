//! Small board folds consumed by Manage and QA.

use crate::services::manage::{dwell_by_column, TaskCard, TaskColumn, TaskTransition};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskOutcome {
    pub landed: bool,
    pub abandoned: bool,
    pub still_open: bool,
    pub reworked: bool,
}

pub fn outcome(card: &TaskCard, verify_passed: Option<bool>, iterations: u32) -> TaskOutcome {
    let landed = card.column == TaskColumn::Done && verify_passed == Some(true);
    let abandoned = verify_passed == Some(false) && card.column != TaskColumn::Done;
    TaskOutcome {
        landed,
        abandoned,
        still_open: !landed && !abandoned,
        reworked: landed && iterations > 1,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanFold {
    pub cards: Vec<TaskCard>,
    pub dwell: crate::services::manage::DwellByColumn,
}

impl KanbanFold {
    pub fn from(cards: Vec<TaskCard>, transitions: &[TaskTransition]) -> Self {
        Self {
            cards,
            dwell: dwell_by_column(transitions),
        }
    }
}

pub fn task_outcomes(
    cards: &[TaskCard],
    verifies: &BTreeMap<crate::ids::TaskId, bool>,
    iterations: &BTreeMap<crate::ids::TaskId, u32>,
) -> Vec<TaskOutcome> {
    cards
        .iter()
        .map(|card| {
            outcome(
                card,
                verifies.get(&card.id).copied(),
                iterations.get(&card.id).copied().unwrap_or(0),
            )
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod board {
    use super::*;
    use crate::{
        ids::{ProjectId, TaskId},
        services::manage::TaskCard,
    };
    use uuid::Uuid;
    fn card(column: TaskColumn) -> TaskCard {
        TaskCard {
            id: TaskId::new(Uuid::new_v4()),
            project_id: ProjectId::new(Uuid::new_v4()),
            title: "task".into(),
            column,
            blocked: false,
            gate: None,
            workflow: None,
            running_tokens: None,
            stuck_iterations: 0,
            verify_command: None,
            evidence_summary: None,
        }
    }
    #[test]
    fn outcomes() {
        let card = card(TaskColumn::Done);
        assert!(outcome(&card, Some(true), 1).landed);
    }
}
