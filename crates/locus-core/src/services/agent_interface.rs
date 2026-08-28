//! The canonical view model shared by the Agent Pane and session projections.
//!
//! This is a projection only: ACP events, plans, controls, and research remain owned by their
//! existing services. The panel receives one session/run/task/workflow/permission snapshot.

use crate::{
    ids::{ProjectId, RunId, SessionId, TaskId},
    runtime::{
        controls::{ActivePlan, ContextView, PermissionPosture},
        session::{Run, RunStatus, Session},
    },
    services::task::TaskDetailSummary,
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPaneLiveStatus {
    Working,
    Waiting,
    Idle,
    Done,
}

impl AgentPaneLiveStatus {
    pub const fn derive(
        run_status: &RunStatus,
        blocker_pending: bool,
        elicitation_pending: bool,
    ) -> Self {
        if blocker_pending || elicitation_pending {
            return Self::Waiting;
        }
        match run_status {
            RunStatus::Running => Self::Working,
            RunStatus::Completed => Self::Done,
            RunStatus::Queued
            | RunStatus::Paused
            | RunStatus::Stopped
            | RunStatus::Aborted
            | RunStatus::Cancelled => Self::Idle,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AgentPaneViewModel {
    pub session_id: SessionId,
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub task_id: Option<TaskId>,
    pub workflow_def_id: Option<Uuid>,
    pub permission_posture: PermissionPosture,
    pub live_status: AgentPaneLiveStatus,
    pub context: ContextView,
    pub active_plan: Option<ActivePlan>,
    pub run_event_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPaneInputs<'a> {
    pub task: Option<&'a TaskDetailSummary>,
    pub context: ContextView,
    pub active_plan: Option<ActivePlan>,
    pub blocker_pending: bool,
    pub elicitation_pending: bool,
}

impl<'a> AgentPaneInputs<'a> {
    pub fn new(context: ContextView) -> Self {
        Self {
            task: None,
            context,
            active_plan: None,
            blocker_pending: false,
            elicitation_pending: false,
        }
    }

    pub fn with_task(mut self, task: &'a TaskDetailSummary) -> Self {
        self.task = Some(task);
        self
    }

    pub fn with_plan(mut self, active_plan: ActivePlan) -> Self {
        self.active_plan = Some(active_plan);
        self
    }

    pub fn with_blocker(mut self) -> Self {
        self.blocker_pending = true;
        self
    }

    pub fn with_elicitation(mut self) -> Self {
        self.elicitation_pending = true;
        self
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AgentPaneViewError {
    #[error("run does not belong to the selected session")]
    RunSessionMismatch,
    #[error("task detail does not belong to the selected session task")]
    TaskMismatch,
    #[error("context view does not belong to the selected session")]
    ContextSessionMismatch,
}

impl AgentPaneViewModel {
    pub fn from_parts(
        session: &Session,
        run: &Run,
        inputs: AgentPaneInputs<'_>,
    ) -> Result<Self, AgentPaneViewError> {
        let AgentPaneInputs {
            task,
            context,
            active_plan,
            blocker_pending,
            elicitation_pending,
        } = inputs;
        if run.session_id != session.id {
            return Err(AgentPaneViewError::RunSessionMismatch);
        }
        if context
            .session_id
            .is_some_and(|context_session_id| context_session_id != session.id)
        {
            return Err(AgentPaneViewError::ContextSessionMismatch);
        }
        if let Some(task) = task {
            let belongs_to_session = session.board_task_id == Some(task.task_id)
                && task
                    .root_session_id
                    .is_none_or(|session_id| session_id == session.id);
            if !belongs_to_session {
                return Err(AgentPaneViewError::TaskMismatch);
            }
        }
        Ok(Self {
            session_id: session.id,
            run_id: run.id,
            project_id: session.project_id,
            task_id: session.board_task_id,
            workflow_def_id: task.and_then(|detail| detail.workflow_def_id),
            // The run is the immutable source of the dispatch posture; a stale UI value
            // cannot change how the pane interprets a permission request.
            permission_posture: run.permission_posture,
            live_status: AgentPaneLiveStatus::derive(
                &run.status,
                blocker_pending,
                elicitation_pending,
            ),
            context,
            active_plan,
            run_event_count: run.events.len(),
        })
    }
}

#[cfg(test)]
mod agent_interface {
    use super::*;
    use crate::{
        ids::{AgentDefId, ProjectId},
        runtime::session::{RunStatus, SessionStatus},
    };

    #[test]
    fn view_model() -> Result<(), AgentPaneViewError> {
        let task_id = TaskId::generate();
        let session = Session {
            id: SessionId::generate(),
            project_id: ProjectId::generate(),
            agent_def_id: AgentDefId::generate(),
            name: "build task".into(),
            branch: "agent/build-task".into(),
            board_task_id: Some(task_id),
            memory_base: serde_json::json!({"catalog": "ready"}),
            pane_state: serde_json::json!({}),
            status: SessionStatus::Active,
            handed_off_from: None,
        };
        let run = Run {
            id: RunId::generate(),
            session_id: session.id,
            resolved_model_id: "model".into(),
            status: RunStatus::Running,
            permission_posture: PermissionPosture::Gated,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        let task = TaskDetailSummary {
            task_id,
            workflow_def_id: Some(Uuid::new_v4()),
            root_session_id: Some(session.id),
            runs: vec![],
            evidence: vec![],
            external_link: None,
        };
        let context = ContextView::from_session(session.id, &session.memory_base, 3);
        let view = AgentPaneViewModel::from_parts(
            &session,
            &run,
            AgentPaneInputs::new(context).with_task(&task),
        )?;
        assert_eq!(view.project_id, session.project_id);
        assert_eq!(view.task_id, session.board_task_id);
        assert_eq!(view.permission_posture, PermissionPosture::Gated);
        assert_eq!(view.live_status, AgentPaneLiveStatus::Working);
        assert_eq!(view.run_event_count, 0);
        Ok(())
    }

    #[test]
    fn pending_human_actions_override_run_progress() -> Result<(), AgentPaneViewError> {
        let session = Session {
            id: SessionId::generate(),
            project_id: ProjectId::generate(),
            agent_def_id: AgentDefId::generate(),
            name: "waiting session".into(),
            branch: "agent/waiting-session".into(),
            board_task_id: None,
            memory_base: serde_json::json!({}),
            pane_state: serde_json::json!({}),
            status: SessionStatus::Active,
            handed_off_from: None,
        };
        let run = Run {
            id: RunId::generate(),
            session_id: session.id,
            resolved_model_id: "model".into(),
            status: RunStatus::Running,
            permission_posture: PermissionPosture::Bypass,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        let context = ContextView::from_session(session.id, &session.memory_base, 0);
        let waiting = AgentPaneViewModel::from_parts(
            &session,
            &run,
            AgentPaneInputs::new(context.clone()).with_blocker(),
        )?;
        let done = AgentPaneViewModel::from_parts(
            &session,
            &Run {
                status: RunStatus::Completed,
                ..run
            },
            AgentPaneInputs::new(context),
        )?;
        assert_eq!(waiting.live_status, AgentPaneLiveStatus::Waiting);
        assert_eq!(done.live_status, AgentPaneLiveStatus::Done);
        Ok(())
    }

    #[test]
    fn task_detail_must_match_the_session_owner() {
        let session = Session {
            id: SessionId::generate(),
            project_id: ProjectId::generate(),
            agent_def_id: AgentDefId::generate(),
            name: "owned task".into(),
            branch: "agent/owned-task".into(),
            board_task_id: Some(TaskId::generate()),
            memory_base: serde_json::json!({}),
            pane_state: serde_json::json!({}),
            status: SessionStatus::Active,
            handed_off_from: None,
        };
        let run = Run {
            id: RunId::generate(),
            session_id: session.id,
            resolved_model_id: "model".into(),
            status: RunStatus::Queued,
            permission_posture: PermissionPosture::Bypass,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        let task = TaskDetailSummary {
            task_id: TaskId::generate(),
            workflow_def_id: None,
            root_session_id: Some(session.id),
            runs: vec![],
            evidence: vec![],
            external_link: None,
        };
        let error = AgentPaneViewModel::from_parts(
            &session,
            &run,
            AgentPaneInputs::new(ContextView::from_session(
                session.id,
                &session.memory_base,
                0,
            ))
            .with_task(&task),
        )
        .expect_err("foreign task must not enter the pane");
        assert_eq!(error, AgentPaneViewError::TaskMismatch);
    }
}
