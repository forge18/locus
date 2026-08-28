//! The board projection and its fixed-column transition rules.
//!
//! The projector consumes domain events; callers never mutate a `board.tasks` row directly.

use crate::{
    ids::{AgentDefId, ArtifactId, EventId, ProjectId, RunId, SessionId, TaskId},
    services::manage::{dwell_by_column, TaskCard, TaskColumn, TaskTransition},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardEvidenceLink {
    pub run_id: Option<RunId>,
    pub event_ids: Vec<EventId>,
    pub artifact_ids: Vec<ArtifactId>,
}

impl BoardEvidenceLink {
    pub fn proves_done(&self) -> bool {
        self.run_id.is_some() && (!self.event_ids.is_empty() || !self.artifact_ids.is_empty())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardComment {
    pub author: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardIssue {
    pub repository: String,
    pub number: u64,
    pub title: String,
    pub url: String,
}

/// The complete board-card contract. Runtime state is projected onto this shape from events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardTask {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub repo_id: Option<Uuid>,
    pub session_id: Option<SessionId>,
    pub summary: String,
    pub description: String,
    pub column: TaskColumn,
    pub blocked: bool,
    pub blocked_reason: Option<String>,
    pub blocked_clear_condition: Option<String>,
    pub assigned_agent: Option<AgentDefId>,
    pub blocked_by: BTreeSet<TaskId>,
    pub verify_command: Option<String>,
    pub evidence: Vec<BoardEvidenceLink>,
    pub comments: Vec<BoardComment>,
    pub external_issue: Option<BoardIssue>,
    /// Full provider identity and imported snapshot; `external_issue` is legacy display data.
    pub external_work_item: Option<crate::work_item::WorkItemSnapshot>,
}

impl BoardTask {
    pub fn new(
        project_id: ProjectId,
        id: TaskId,
        summary: impl Into<String>,
        verify_command: Option<String>,
    ) -> Self {
        Self {
            id,
            project_id,
            repo_id: None,
            session_id: None,
            summary: summary.into(),
            description: String::new(),
            column: TaskColumn::Ready,
            blocked: false,
            blocked_reason: None,
            blocked_clear_condition: None,
            assigned_agent: None,
            blocked_by: BTreeSet::new(),
            verify_command,
            evidence: Vec::new(),
            comments: Vec::new(),
            external_issue: None,
            external_work_item: None,
        }
    }

    pub fn block(&mut self, reason: impl Into<String>, clear_condition: impl Into<String>) {
        self.blocked = true;
        self.blocked_reason = Some(reason.into());
        self.blocked_clear_condition = Some(clear_condition.into());
    }

    pub fn transition(
        &self,
        to: TaskColumn,
        actor: BoardActor,
        evidence: Vec<BoardEvidenceLink>,
    ) -> Result<BoardEvent, BoardError> {
        if to == TaskColumn::Done
            && matches!(actor, BoardActor::Agent { .. })
            && !evidence.iter().any(BoardEvidenceLink::proves_done)
        {
            return Err(BoardError::AgentDoneNeedsEvidence { task_id: self.id });
        }
        Ok(BoardEvent::Moved {
            task_id: self.id,
            from: self.column,
            to,
            actor,
            evidence,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BoardActor {
    Human,
    Agent { run_id: RunId },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BoardError {
    #[error("board task `{task_id}` does not exist")]
    TaskNotFound { task_id: TaskId },
    #[error("agent cannot move task `{task_id}` to Done without run and event evidence")]
    AgentDoneNeedsEvidence { task_id: TaskId },
    #[error("blocked status is derived from dependencies and cannot be cleared manually")]
    ManualBlockedClear,
    #[error("board task `{task_id}` already exists")]
    DuplicateTask { task_id: TaskId },
    #[error("board dependency cannot point from task `{task_id}` to itself")]
    SelfDependency { task_id: TaskId },
    #[error("board task `{task_id}` has no active session for an approval inbox item")]
    ApprovalWithoutSession { task_id: TaskId },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BoardEvent {
    Created {
        task: Box<BoardTask>,
    },
    Moved {
        task_id: TaskId,
        from: TaskColumn,
        to: TaskColumn,
        actor: BoardActor,
        evidence: Vec<BoardEvidenceLink>,
    },
    Assigned {
        task_id: TaskId,
        agent: AgentDefId,
        actor: BoardActor,
    },
    Commented {
        task_id: TaskId,
        comment: BoardComment,
        actor: BoardActor,
    },
    /// This is the only dependency event. Its workflow node id is the source of truth;
    /// there is no hand-drawn edge operation in the board API.
    WorkflowDependency {
        task_id: TaskId,
        blocked_by: TaskId,
        workflow_node_id: String,
    },
    RunCompleted {
        task_id: TaskId,
        run_id: RunId,
        passed: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardApprovalInboxItem {
    pub task_id: TaskId,
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub title: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BoardProjection {
    tasks: BTreeMap<TaskId, BoardTask>,
}

/// Turn validated workflow Task-node edges into board events. The mapping from
/// workflow node handles to persisted task ids is supplied by the task creator;
/// there is deliberately no UI/manual-edge input here.
pub fn workflow_dependency_events(
    graph: &crate::services::workflow::graph::WorkflowGraph,
    task_ids: &BTreeMap<String, TaskId>,
) -> Vec<BoardEvent> {
    crate::services::workflow::graph::blocked_by_edges(graph)
        .into_iter()
        .filter_map(|dependency| {
            Some(BoardEvent::WorkflowDependency {
                task_id: *task_ids.get(&dependency.task_id)?,
                blocked_by: *task_ids.get(&dependency.blocked_by)?,
                workflow_node_id: dependency.blocked_by,
            })
        })
        .collect()
}

impl BoardProjection {
    pub fn from_events(events: impl IntoIterator<Item = BoardEvent>) -> Result<Self, BoardError> {
        let mut projection = Self::default();
        for event in events {
            projection.apply(event)?;
        }
        Ok(projection)
    }

    pub fn apply(&mut self, event: BoardEvent) -> Result<(), BoardError> {
        match event {
            BoardEvent::Created { task } => {
                let task = *task;
                if self.tasks.contains_key(&task.id) {
                    return Err(BoardError::DuplicateTask { task_id: task.id });
                }
                self.tasks.insert(task.id, task);
            }
            BoardEvent::Moved {
                task_id,
                from,
                to,
                actor,
                evidence,
            } => {
                let task = self
                    .tasks
                    .get(&task_id)
                    .ok_or(BoardError::TaskNotFound { task_id })?;
                if task.column != from {
                    return Err(BoardError::TaskNotFound { task_id });
                }
                let event = task.transition(to, actor, evidence)?;
                let BoardEvent::Moved { evidence, .. } = event else {
                    unreachable!("task transition always produces a move")
                };
                let task = self.tasks.get_mut(&task_id).expect("checked above");
                task.column = to;
                task.evidence.extend(evidence);
            }
            BoardEvent::Assigned {
                task_id,
                agent,
                actor: _,
            } => {
                self.tasks
                    .get_mut(&task_id)
                    .ok_or(BoardError::TaskNotFound { task_id })?
                    .assigned_agent = Some(agent);
            }
            BoardEvent::Commented {
                task_id,
                comment,
                actor: _,
            } => {
                self.tasks
                    .get_mut(&task_id)
                    .ok_or(BoardError::TaskNotFound { task_id })?
                    .comments
                    .push(comment);
            }
            BoardEvent::WorkflowDependency {
                task_id,
                blocked_by,
                workflow_node_id: _,
            } => {
                if task_id == blocked_by {
                    return Err(BoardError::SelfDependency { task_id });
                }
                if !self.tasks.contains_key(&blocked_by) {
                    return Err(BoardError::TaskNotFound {
                        task_id: blocked_by,
                    });
                }
                let task = self
                    .tasks
                    .get_mut(&task_id)
                    .ok_or(BoardError::TaskNotFound { task_id })?;
                task.blocked_by.insert(blocked_by);
                task.block("dependency", "predecessor completes");
            }
            BoardEvent::RunCompleted {
                task_id,
                run_id: _,
                passed,
            } => {
                if !self.tasks.contains_key(&task_id) {
                    return Err(BoardError::TaskNotFound { task_id });
                }
                if passed {
                    for dependent in self.tasks.values_mut() {
                        if dependent.blocked_by.remove(&task_id) && dependent.blocked_by.is_empty()
                        {
                            dependent.blocked = false;
                            dependent.blocked_reason = None;
                            dependent.blocked_clear_condition = None;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn task(&self, task_id: TaskId) -> Option<&BoardTask> {
        self.tasks.get(&task_id)
    }

    pub fn tasks(&self) -> impl Iterator<Item = &BoardTask> {
        self.tasks.values()
    }

    pub fn pending_approvals(&self) -> Result<Vec<BoardApprovalInboxItem>, BoardError> {
        self.tasks
            .values()
            .filter(|task| task.column == TaskColumn::PendingApproval)
            .map(|task| {
                Ok(BoardApprovalInboxItem {
                    task_id: task.id,
                    project_id: task.project_id,
                    session_id: task
                        .session_id
                        .ok_or(BoardError::ApprovalWithoutSession { task_id: task.id })?,
                    title: task.summary.clone(),
                })
            })
            .collect()
    }

    pub fn clear_blocked_manually(&self, _task_id: TaskId) -> Result<(), BoardError> {
        Err(BoardError::ManualBlockedClear)
    }

    pub fn next_unblocked(&self, agent: AgentDefId) -> Option<TaskId> {
        self.tasks
            .values()
            .find(|task| !task.blocked && task.assigned_agent == Some(agent))
            .map(|task| task.id)
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod board {
    use super::*;
    use crate::{
        ids::{AgentDefId, EventId, ProjectId, RunId, SessionId, TaskId},
        services::manage::TaskCard,
    };
    use serde_json::json;
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
    fn board_task(column: TaskColumn) -> BoardTask {
        let mut task = BoardTask::new(
            ProjectId::generate(),
            TaskId::generate(),
            "task",
            Some("cargo test".into()),
        );
        task.column = column;
        task
    }

    fn evidence() -> BoardEvidenceLink {
        BoardEvidenceLink {
            run_id: Some(RunId::generate()),
            event_ids: vec![EventId::generate()],
            artifact_ids: vec![],
        }
    }

    fn created_projection() -> (BoardProjection, BoardTask, BoardTask) {
        let predecessor = board_task(TaskColumn::Done);
        let dependent = board_task(TaskColumn::Testing);
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(predecessor.clone()),
            })
            .expect("create predecessor");
        projection
            .apply(BoardEvent::Created {
                task: Box::new(dependent.clone()),
            })
            .expect("create dependent");
        (projection, predecessor, dependent)
    }

    #[test]
    fn outcomes() {
        let card = card(TaskColumn::Done);
        assert!(outcome(&card, Some(true), 1).landed);
    }

    #[test]
    fn six_fixed_columns() {
        assert_eq!(TaskColumn::ALL.len(), 6);
        assert_eq!(TaskColumn::InProgress.as_str(), "in_progress");
    }

    #[test]
    fn columns_are_closed() {
        assert_eq!(
            serde_json::from_str::<TaskColumn>("\"building\"").is_err(),
            true
        );
        assert_eq!(
            serde_json::to_value(TaskColumn::Done).expect("column json"),
            json!("done")
        );
    }

    #[test]
    fn task_shape() {
        let task = board_task(TaskColumn::Ready);
        assert!(task.verify_command.is_some());
        assert!(task.blocked_by.is_empty());
        assert!(task.evidence.is_empty());
        assert!(task.external_issue.is_none());
    }

    #[test]
    fn blocked_is_a_status() {
        let mut task = board_task(TaskColumn::Reviewing);
        task.block("waiting on predecessor", "predecessor completes");
        assert_eq!(task.column, TaskColumn::Reviewing);
        assert!(task.blocked);
    }

    #[test]
    fn blockable_anywhere() {
        for column in TaskColumn::ALL {
            let mut task = board_task(column);
            task.block("dependency", "predecessor completes");
            assert!(task.blocked);
            assert_eq!(task.column, column);
        }
    }

    #[test]
    fn transitions() {
        let task = board_task(TaskColumn::Ready);
        let event = task
            .transition(TaskColumn::InProgress, BoardActor::Human, vec![])
            .expect("human transition");
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(task.clone()),
            })
            .expect("create");
        projection.apply(event).expect("move");
        assert_eq!(
            projection.task(task.id).expect("task").column,
            TaskColumn::InProgress
        );
    }

    #[test]
    fn evidence_links() {
        assert!(evidence().proves_done());
    }

    #[test]
    fn agent_done_needs_evidence() {
        let task = board_task(TaskColumn::Reviewing);
        assert!(matches!(
            task.transition(
                TaskColumn::Done,
                BoardActor::Agent {
                    run_id: RunId::generate()
                },
                vec![]
            ),
            Err(BoardError::AgentDoneNeedsEvidence { .. })
        ));
    }

    #[test]
    fn human_is_unrestricted() {
        let task = board_task(TaskColumn::Reviewing);
        let event = task
            .transition(TaskColumn::Done, BoardActor::Human, vec![])
            .expect("human can move without evidence");
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(task),
            })
            .expect("create");
        projection.apply(event).expect("move");
    }

    #[test]
    fn edges_from_graph() {
        let (mut projection, predecessor, dependent) = created_projection();
        let graph: crate::services::workflow::graph::WorkflowGraph = serde_json::from_value(json!({
            "version": 1,
            "nodes": [
                {"id": "predecessor", "kind": "Task", "position": {"x": 0, "y": 0}},
                {"id": "dependent", "kind": "Task", "position": {"x": 10, "y": 0}}
            ],
            "edges": [{"id": "edge", "source": "predecessor", "sourceHandle": "out", "target": "dependent", "targetHandle": "in"}]
        })).expect("workflow graph");
        let task_ids = BTreeMap::from([
            ("predecessor".into(), predecessor.id),
            ("dependent".into(), dependent.id),
        ]);
        for event in workflow_dependency_events(&graph, &task_ids) {
            projection.apply(event).expect("workflow edge");
        }
        assert!(projection.task(dependent.id).expect("dependent").blocked);
    }

    #[test]
    fn no_manual_edges() {
        let event = BoardEvent::WorkflowDependency {
            task_id: TaskId::generate(),
            blocked_by: TaskId::generate(),
            workflow_node_id: "workflow-node".into(),
        };
        assert!(matches!(event, BoardEvent::WorkflowDependency { .. }));
    }

    #[test]
    fn auto_unblock() {
        let (mut projection, predecessor, dependent) = created_projection();
        projection
            .apply(BoardEvent::WorkflowDependency {
                task_id: dependent.id,
                blocked_by: predecessor.id,
                workflow_node_id: "task-edge".into(),
            })
            .expect("dependency");
        projection
            .apply(BoardEvent::RunCompleted {
                task_id: predecessor.id,
                run_id: RunId::generate(),
                passed: true,
            })
            .expect("completion");
        assert!(!projection.task(dependent.id).expect("dependent").blocked);
    }

    #[test]
    fn unblock_does_not_move() {
        let (mut projection, predecessor, dependent) = created_projection();
        projection
            .apply(BoardEvent::WorkflowDependency {
                task_id: dependent.id,
                blocked_by: predecessor.id,
                workflow_node_id: "task-edge".into(),
            })
            .expect("dependency");
        projection
            .apply(BoardEvent::RunCompleted {
                task_id: predecessor.id,
                run_id: RunId::generate(),
                passed: true,
            })
            .expect("completion");
        assert_eq!(
            projection.task(dependent.id).expect("dependent").column,
            TaskColumn::Testing
        );
    }

    #[test]
    fn no_manual_unblock() {
        let projection = BoardProjection::default();
        assert_eq!(
            projection.clear_blocked_manually(TaskId::generate()),
            Err(BoardError::ManualBlockedClear)
        );
    }

    #[test]
    fn picked_up_automatically() {
        let (mut projection, predecessor, dependent) = created_projection();
        let agent = AgentDefId::generate();
        projection
            .apply(BoardEvent::Assigned {
                task_id: dependent.id,
                agent,
                actor: BoardActor::Human,
            })
            .expect("assign dependent");
        projection
            .apply(BoardEvent::WorkflowDependency {
                task_id: dependent.id,
                blocked_by: predecessor.id,
                workflow_node_id: "task-edge".into(),
            })
            .expect("dependency");
        projection
            .apply(BoardEvent::RunCompleted {
                task_id: predecessor.id,
                run_id: RunId::generate(),
                passed: true,
            })
            .expect("completion");
        assert_eq!(projection.next_unblocked(agent), Some(dependent.id));
    }

    #[test]
    fn approval_is_an_inbox_item() {
        let mut task = board_task(TaskColumn::PendingApproval);
        task.session_id = Some(SessionId::generate());
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(task),
            })
            .expect("create");
        assert_eq!(
            projection
                .pending_approvals()
                .expect("approval items")
                .len(),
            1
        );
    }
}
