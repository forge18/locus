//! Task-centric orchestration ownership.
//!
//! A task owns one workflow execution and root session. Every run, reset, child
//! invocation, control, and evidence reference stays below that root.

use std::collections::BTreeMap;

use crate::{
    ids::{ProjectId, RunId, SessionId, TaskId},
    services::board::BoardTask,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSelection {
    pub task_id: TaskId,
    pub project_id: ProjectId,
    pub workflow_def_id: Option<Uuid>,
    pub confirmed: bool,
}

impl WorkflowSelection {
    pub fn default_for(
        task_id: TaskId,
        project_id: ProjectId,
        workflow_def_id: Option<Uuid>,
    ) -> Self {
        Self {
            task_id,
            project_id,
            workflow_def_id,
            confirmed: false,
        }
    }

    pub fn confirm(mut self) -> Result<Self, TaskError> {
        if self.workflow_def_id.is_none() {
            return Err(TaskError::WorkflowRequired {
                task_id: self.task_id,
            });
        }
        self.confirmed = true;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualTaskDraft {
    pub task: BoardTask,
    pub workflow: WorkflowSelection,
}

impl ManualTaskDraft {
    pub fn new(project_id: ProjectId, summary: impl Into<String>) -> Self {
        let task = BoardTask::new(project_id, TaskId::generate(), summary, None);
        Self {
            workflow: WorkflowSelection::default_for(task.id, project_id, None),
            task,
        }
    }

    pub fn confirm_workflow(mut self, workflow_def_id: Uuid) -> Result<Self, TaskError> {
        self.workflow.workflow_def_id = Some(workflow_def_id);
        self.workflow = self.workflow.confirm()?;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRootSession {
    pub task_id: TaskId,
    pub workflow_def_id: Uuid,
    pub session_id: SessionId,
    pub execution_id: Uuid,
}

impl TaskRootSession {
    pub fn new(
        task_id: Option<TaskId>,
        workflow_def_id: Option<Uuid>,
        session_id: SessionId,
        execution_id: Uuid,
    ) -> Result<Self, TaskError> {
        let task_id = task_id.ok_or(TaskError::RootSessionRequiresTask)?;
        let workflow_def_id = workflow_def_id.ok_or(TaskError::WorkflowRequired { task_id })?;
        Ok(Self {
            task_id,
            workflow_def_id,
            session_id,
            execution_id,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRunLink {
    pub task_id: TaskId,
    pub root_session_id: SessionId,
    pub run_id: RunId,
    pub parent_run_id: Option<RunId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskEvidenceLink {
    pub run_id: RunId,
    pub event_ids: Vec<Uuid>,
    pub artifact_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDetailSummary {
    pub task_id: TaskId,
    pub workflow_def_id: Option<Uuid>,
    pub root_session_id: Option<SessionId>,
    pub runs: Vec<TaskRunLink>,
    pub evidence: Vec<TaskEvidenceLink>,
    pub external_link: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskControl {
    Pause,
    Cancel,
    Handoff,
    NeedsAttention,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskControlReceipt {
    pub task_id: TaskId,
    pub run_id: RunId,
    pub action: TaskControl,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TaskError {
    #[error("task `{task_id}` requires a confirmed workflow before it can start")]
    WorkflowRequired { task_id: TaskId },
    #[error("an orchestration root session must have an owning task")]
    RootSessionRequiresTask,
    #[error("workflow `{workflow_def_id}` belongs to another project")]
    WorkflowProjectMismatch { workflow_def_id: Uuid },
    #[error("task `{task_id}` does not exist")]
    TaskNotFound { task_id: TaskId },
    #[error("task `{task_id}` is already registered")]
    DuplicateTask { task_id: TaskId },
    #[error("run `{run_id}` is not owned by task `{task_id}`")]
    RunNotOwned { task_id: TaskId, run_id: RunId },
    #[error("run `{run_id}` is already linked to task `{task_id}`")]
    DuplicateRun { task_id: TaskId, run_id: RunId },
}

#[derive(Clone, Debug)]
struct TaskRecord {
    task: BoardTask,
    workflow: WorkflowSelection,
    root: Option<TaskRootSession>,
    runs: BTreeMap<RunId, TaskRunLink>,
    evidence: Vec<TaskEvidenceLink>,
    external_link: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TaskOrchestrator {
    tasks: BTreeMap<TaskId, TaskRecord>,
}

impl TaskOrchestrator {
    pub fn register(
        &mut self,
        task: BoardTask,
        workflow: WorkflowSelection,
    ) -> Result<(), TaskError> {
        if self.tasks.contains_key(&task.id) {
            return Err(TaskError::DuplicateTask { task_id: task.id });
        }
        self.tasks.insert(
            task.id,
            TaskRecord {
                task,
                workflow,
                root: None,
                runs: BTreeMap::new(),
                evidence: Vec::new(),
                external_link: None,
            },
        );
        Ok(())
    }

    pub fn update_task(&mut self, task: BoardTask) -> Result<(), TaskError> {
        let record = self
            .tasks
            .get_mut(&task.id)
            .ok_or(TaskError::TaskNotFound { task_id: task.id })?;
        record.task = task;
        Ok(())
    }

    pub fn restore_task_state(
        &mut self,
        task: BoardTask,
        workflow: WorkflowSelection,
        root_session_id: Option<SessionId>,
        runs: Vec<TaskRunLink>,
        evidence: Vec<TaskEvidenceLink>,
        external_link: Option<String>,
    ) -> Result<(), TaskError> {
        if !workflow.confirmed {
            return Err(TaskError::WorkflowRequired { task_id: task.id });
        }
        let workflow_def_id = workflow
            .workflow_def_id
            .ok_or(TaskError::WorkflowRequired { task_id: task.id })?;
        self.register(task.clone(), workflow)?;
        let record = self
            .tasks
            .get_mut(&task.id)
            .ok_or(TaskError::TaskNotFound { task_id: task.id })?;
        record.root = root_session_id.map(|session_id| TaskRootSession {
            task_id: task.id,
            workflow_def_id,
            session_id,
            execution_id: Uuid::nil(),
        });
        for run in runs {
            if run.task_id != task.id {
                return Err(TaskError::RunNotOwned {
                    task_id: task.id,
                    run_id: run.run_id,
                });
            }
            if record.runs.insert(run.run_id, run.clone()).is_some() {
                return Err(TaskError::DuplicateRun {
                    task_id: task.id,
                    run_id: run.run_id,
                });
            }
        }
        for item in evidence {
            if !record.runs.contains_key(&item.run_id) {
                return Err(TaskError::RunNotOwned {
                    task_id: task.id,
                    run_id: item.run_id,
                });
            }
            record.evidence.push(item);
        }
        record.external_link = external_link;
        Ok(())
    }

    pub fn select_workflow(
        &mut self,
        task_id: TaskId,
        workflow_def_id: Uuid,
        workflow_project_id: ProjectId,
    ) -> Result<(), TaskError> {
        let record = self
            .tasks
            .get_mut(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        if record.task.project_id != workflow_project_id {
            return Err(TaskError::WorkflowProjectMismatch { workflow_def_id });
        }
        record.workflow.workflow_def_id = Some(workflow_def_id);
        record.workflow.confirmed = false;
        Ok(())
    }

    pub fn confirm_workflow(&mut self, task_id: TaskId) -> Result<(), TaskError> {
        let record = self
            .tasks
            .get_mut(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        record.workflow = record.workflow.clone().confirm()?;
        Ok(())
    }

    pub fn start_task(&mut self, task_id: TaskId) -> Result<TaskRootSession, TaskError> {
        let record = self
            .tasks
            .get_mut(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        if !record.workflow.confirmed {
            return Err(TaskError::WorkflowRequired { task_id });
        }
        let root = TaskRootSession::new(
            Some(task_id),
            record.workflow.workflow_def_id,
            SessionId::generate(),
            Uuid::new_v4(),
        )?;
        record.root = Some(root.clone());
        Ok(root)
    }

    pub fn link_reset_run(&mut self, task_id: TaskId, run_id: RunId) -> Result<(), TaskError> {
        self.link_run(task_id, run_id, None)
    }

    pub fn link_child_run(
        &mut self,
        task_id: TaskId,
        parent_run_id: RunId,
        run_id: RunId,
    ) -> Result<(), TaskError> {
        let record = self
            .tasks
            .get(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        if !record.runs.contains_key(&parent_run_id) {
            return Err(TaskError::RunNotOwned {
                task_id,
                run_id: parent_run_id,
            });
        }
        self.link_run(task_id, run_id, Some(parent_run_id))
    }

    fn link_run(
        &mut self,
        task_id: TaskId,
        run_id: RunId,
        parent_run_id: Option<RunId>,
    ) -> Result<(), TaskError> {
        let record = self
            .tasks
            .get_mut(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        let root = record
            .root
            .as_ref()
            .ok_or(TaskError::WorkflowRequired { task_id })?;
        if record.runs.contains_key(&run_id) {
            return Err(TaskError::DuplicateRun { task_id, run_id });
        }
        record.runs.insert(
            run_id,
            TaskRunLink {
                task_id,
                root_session_id: root.session_id,
                run_id,
                parent_run_id,
            },
        );
        Ok(())
    }

    pub fn add_evidence(
        &mut self,
        task_id: TaskId,
        evidence: TaskEvidenceLink,
    ) -> Result<(), TaskError> {
        let record = self
            .tasks
            .get_mut(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        if !record.runs.contains_key(&evidence.run_id) {
            return Err(TaskError::RunNotOwned {
                task_id,
                run_id: evidence.run_id,
            });
        }
        record.evidence.push(evidence);
        Ok(())
    }

    pub fn set_external_link(
        &mut self,
        task_id: TaskId,
        external_link: impl Into<String>,
    ) -> Result<(), TaskError> {
        self.tasks
            .get_mut(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?
            .external_link = Some(external_link.into());
        Ok(())
    }

    pub fn detail(&self, task_id: TaskId) -> Result<TaskDetailSummary, TaskError> {
        let record = self
            .tasks
            .get(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        Ok(TaskDetailSummary {
            task_id,
            workflow_def_id: record.workflow.workflow_def_id,
            root_session_id: record.root.as_ref().map(|root| root.session_id),
            runs: record.runs.values().cloned().collect(),
            evidence: record.evidence.clone(),
            external_link: record.external_link.clone(),
        })
    }

    pub fn control(
        &self,
        task_id: TaskId,
        run_id: RunId,
        action: TaskControl,
    ) -> Result<TaskControlReceipt, TaskError> {
        let record = self
            .tasks
            .get(&task_id)
            .ok_or(TaskError::TaskNotFound { task_id })?;
        if !record.runs.contains_key(&run_id) {
            return Err(TaskError::RunNotOwned { task_id, run_id });
        }
        Ok(TaskControlReceipt {
            task_id,
            run_id,
            action,
        })
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod task {
    use super::*;

    fn registered() -> (TaskOrchestrator, TaskId, Uuid) {
        let project = ProjectId::generate();
        let task = BoardTask::new(
            project,
            TaskId::generate(),
            "task",
            Some("cargo test".into()),
        );
        let task_id = task.id;
        let workflow = Uuid::new_v4();
        let selection = WorkflowSelection::default_for(task_id, project, Some(workflow));
        let mut orchestrator = TaskOrchestrator::default();
        orchestrator
            .register(task, selection)
            .expect("register task");
        (orchestrator, task_id, workflow)
    }

    #[test]
    fn workflow_session_link() {
        let (mut orchestrator, task_id, workflow) = registered();
        orchestrator.confirm_workflow(task_id).expect("confirm");
        let root = orchestrator.start_task(task_id).expect("start");
        assert_eq!(root.task_id, task_id);
        assert_eq!(root.workflow_def_id, workflow);
    }

    #[test]
    fn root_session_requires_task() {
        assert_eq!(
            TaskRootSession::new(
                None,
                Some(Uuid::new_v4()),
                SessionId::generate(),
                Uuid::new_v4()
            ),
            Err(TaskError::RootSessionRequiresTask)
        );
    }

    #[test]
    fn workflow_selection() {
        let project = ProjectId::generate();
        let draft = ManualTaskDraft::new(project, "draft");
        assert!(!draft.workflow.confirmed);
        let confirmed = draft
            .confirm_workflow(Uuid::new_v4())
            .expect("confirm default");
        assert!(confirmed.workflow.confirmed);
    }

    #[test]
    fn start_creates_root_session() {
        let (mut orchestrator, task_id, _) = registered();
        assert!(matches!(
            orchestrator.start_task(task_id),
            Err(TaskError::WorkflowRequired { .. })
        ));
        orchestrator.confirm_workflow(task_id).expect("confirm");
        assert!(orchestrator.start_task(task_id).is_ok());
    }

    #[test]
    fn reset_stays_with_task() {
        let (mut orchestrator, task_id, _) = registered();
        orchestrator.confirm_workflow(task_id).expect("confirm");
        let root = orchestrator.start_task(task_id).expect("start");
        let run = RunId::generate();
        orchestrator
            .link_reset_run(task_id, run)
            .expect("link reset");
        assert_eq!(
            orchestrator.detail(task_id).unwrap().root_session_id,
            Some(root.session_id)
        );
        assert_eq!(
            orchestrator.detail(task_id).unwrap().runs[0].task_id,
            task_id
        );
    }

    #[test]
    fn child_runs_stay_with_task() {
        let (mut orchestrator, task_id, _) = registered();
        orchestrator.confirm_workflow(task_id).expect("confirm");
        orchestrator.start_task(task_id).expect("start");
        let parent = RunId::generate();
        let child = RunId::generate();
        orchestrator
            .link_reset_run(task_id, parent)
            .expect("parent");
        orchestrator
            .link_child_run(task_id, parent, child)
            .expect("child");
        assert_eq!(orchestrator.detail(task_id).unwrap().runs.len(), 2);
    }

    #[test]
    fn detail_summary() {
        let (mut orchestrator, task_id, workflow) = registered();
        orchestrator.confirm_workflow(task_id).expect("confirm");
        orchestrator.start_task(task_id).expect("start");
        let run = RunId::generate();
        orchestrator.link_reset_run(task_id, run).expect("run");
        orchestrator
            .add_evidence(
                task_id,
                TaskEvidenceLink {
                    run_id: run,
                    event_ids: vec![Uuid::new_v4()],
                    artifact_ids: vec![],
                },
            )
            .expect("evidence");
        orchestrator
            .set_external_link(task_id, "https://tracker/task/1")
            .expect("link");
        let detail = orchestrator.detail(task_id).expect("detail");
        assert_eq!(detail.workflow_def_id, Some(workflow));
        assert_eq!(detail.evidence.len(), 1);
        assert!(detail.external_link.is_some());
    }

    #[test]
    fn controls_are_task_scoped() {
        let (mut orchestrator, task_id, _) = registered();
        orchestrator.confirm_workflow(task_id).expect("confirm");
        orchestrator.start_task(task_id).expect("start");
        let run = RunId::generate();
        orchestrator.link_reset_run(task_id, run).expect("run");
        assert!(orchestrator
            .control(task_id, run, TaskControl::Pause)
            .is_ok());
        assert!(matches!(
            orchestrator.control(task_id, RunId::generate(), TaskControl::Cancel),
            Err(TaskError::RunNotOwned { .. })
        ));
    }
}
