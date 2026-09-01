//! Durable Planning Workspace wrapper projections.

use crate::ids::{PlanningWorkspaceId, ProjectId};
use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sqlx::{query, query_as, query_scalar, Row};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct PlanningWorkspaceRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub scope: String,
    pub lifecycle: String,
    pub current_revision: i32,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PlanningWorkspaceRevisionRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub revision: i32,
    pub state: Value,
    pub frozen_at: Option<String>,
    pub approved_at: Option<String>,
}

fn valid_scope(scope: &str) -> bool {
    matches!(scope, "amendment" | "feature" | "project")
}

fn valid_checkpoint_lifecycle(lifecycle: &str) -> bool {
    matches!(lifecycle, "draft" | "in_progress" | "ready_for_approval")
}

impl crate::store::Store {
    pub async fn planning_workspaces(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<PlanningWorkspaceRow>> {
        query_as(
            "SELECT id, project_id, scope, lifecycle, current_revision,
                    updated_at::text AS updated_at
             FROM core.planning_workspaces
             WHERE ($1::uuid IS NULL OR project_id = $1)
             ORDER BY updated_at DESC, id",
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list planning workspaces")
    }

    pub async fn planning_workspace(
        &self,
        project_id: ProjectId,
        workspace_id: PlanningWorkspaceId,
    ) -> Result<Option<PlanningWorkspaceRow>> {
        query_as(
            "SELECT id, project_id, scope, lifecycle, current_revision,
                    updated_at::text AS updated_at
             FROM core.planning_workspaces
             WHERE id = $1 AND project_id = $2",
        )
        .bind(workspace_id.as_uuid())
        .bind(project_id)
        .fetch_optional(self.pool())
        .await
        .context("read planning workspace")
    }

    pub async fn create_planning_workspace(
        &self,
        project_id: ProjectId,
        scope: &str,
        brief: &str,
    ) -> Result<PlanningWorkspaceId> {
        if !valid_scope(scope) {
            bail!("planning workspace scope must be amendment, feature, or project");
        }
        if brief.trim().is_empty() {
            bail!("planning workspace brief must not be empty");
        }
        let workspace_id = PlanningWorkspaceId::generate();
        let revision_id = Uuid::new_v4();
        let mut tx = self.pool().begin().await.context("begin planning workspace")?;
        query(
            "INSERT INTO core.planning_workspaces (id, project_id, scope)
             VALUES ($1, $2, $3)",
        )
        .bind(workspace_id)
        .bind(project_id)
        .bind(scope)
        .execute(&mut *tx)
        .await
        .context("create planning workspace")?;
        query(
            "INSERT INTO core.planning_workspace_revisions
                (id, workspace_id, revision, state)
             VALUES ($1, $2, 1, $3)",
        )
        .bind(revision_id)
        .bind(workspace_id)
        .bind(json!({ "brief": brief }))
        .execute(&mut *tx)
        .await
        .context("create planning workspace revision")?;
        query(
            "INSERT INTO core.planning_workspace_events
                (id, workspace_id, revision_id, kind, payload)
             VALUES ($1, $2, $3, 'created', $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(revision_id)
        .bind(json!({ "scope": scope }))
        .execute(&mut *tx)
        .await
        .context("record planning workspace creation")?;
        tx.commit().await.context("commit planning workspace")?;
        Ok(workspace_id)
    }

    pub async fn planning_workspace_revisions(
        &self,
        project_id: ProjectId,
        workspace_id: PlanningWorkspaceId,
    ) -> Result<Vec<PlanningWorkspaceRevisionRow>> {
        query_as(
            "SELECT r.id, r.workspace_id, r.revision, r.state,
                    r.frozen_at::text AS frozen_at, r.approved_at::text AS approved_at
             FROM core.planning_workspace_revisions r
             JOIN core.planning_workspaces w ON w.id = r.workspace_id
             WHERE r.workspace_id = $1 AND w.project_id = $2
             ORDER BY r.revision",
        )
        .bind(workspace_id.as_uuid())
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list planning workspace revisions")
    }

    pub async fn save_planning_workspace_checkpoint(
        &self,
        project_id: ProjectId,
        workspace_id: PlanningWorkspaceId,
        expected_revision: i32,
        lifecycle: &str,
        state: Value,
    ) -> Result<i32> {
        if !valid_checkpoint_lifecycle(lifecycle) {
            bail!("checkpoint lifecycle must be draft, in_progress, or ready_for_approval");
        }
        let mut tx = self.pool().begin().await.context("begin workspace checkpoint")?;
        let row = query(
            "SELECT lifecycle, current_revision
             FROM core.planning_workspaces
             WHERE id = $1 AND project_id = $2
             FOR UPDATE",
        )
        .bind(workspace_id.as_uuid())
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await
        .context("lock planning workspace checkpoint")?
        .ok_or_else(|| anyhow::anyhow!("planning workspace was not found in the project"))?;
        let current_lifecycle: String = row.try_get("lifecycle")?;
        let current_revision: i32 = row.try_get("current_revision")?;
        if current_lifecycle == "approved" || current_lifecycle == "deleted" {
            bail!("planning workspace is terminal");
        }
        if current_revision != expected_revision {
            bail!(
                "planning workspace revision conflict: expected {expected_revision}, current {current_revision}"
            );
        }
        let next_revision = current_revision + 1;
        let revision_id = Uuid::new_v4();
        query(
            "INSERT INTO core.planning_workspace_revisions
                (id, workspace_id, revision, state, frozen_at)
             VALUES ($1, $2, $3, $4, CASE WHEN $5 = 'ready_for_approval' THEN now() END)",
        )
        .bind(revision_id)
        .bind(workspace_id)
        .bind(next_revision)
        .bind(state)
        .bind(lifecycle)
        .execute(&mut *tx)
        .await
        .context("save workspace revision")?;
        query(
            "UPDATE core.planning_workspaces
             SET lifecycle = $3, current_revision = $2, updated_at = now()
             WHERE id = $1",
        )
        .bind(workspace_id)
        .bind(next_revision)
        .bind(lifecycle)
        .execute(&mut *tx)
        .await
        .context("advance planning workspace")?;
        query(
            "INSERT INTO core.planning_workspace_events
                (id, workspace_id, revision_id, kind, payload)
             VALUES ($1, $2, $3, 'checkpoint_saved', $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(revision_id)
        .bind(json!({ "lifecycle": lifecycle, "revision": next_revision }))
        .execute(&mut *tx)
        .await
        .context("record workspace checkpoint")?;
        tx.commit().await.context("commit workspace checkpoint")?;
        Ok(next_revision)
    }

    pub async fn approve_planning_workspace(
        &self,
        project_id: ProjectId,
        workspace_id: PlanningWorkspaceId,
        expected_revision: i32,
    ) -> Result<Vec<Uuid>> {
        let mut tx = self.pool().begin().await.context("begin workspace approval")?;
        let workspace = query(
            "SELECT lifecycle, current_revision
             FROM core.planning_workspaces
             WHERE id = $1 AND project_id = $2
             FOR UPDATE",
        )
        .bind(workspace_id.as_uuid())
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await
        .context("lock planning workspace approval")?
        .ok_or_else(|| anyhow::anyhow!("planning workspace was not found in the project"))?;
        let lifecycle: String = workspace.try_get("lifecycle")?;
        let current_revision: i32 = workspace.try_get("current_revision")?;
        if lifecycle != "ready_for_approval" {
            bail!("planning workspace is not ready for approval");
        }
        if current_revision != expected_revision {
            bail!(
                "planning workspace revision conflict: expected {expected_revision}, current {current_revision}"
            );
        }
        let revision = query(
            "SELECT id, state FROM core.planning_workspace_revisions
             WHERE workspace_id = $1 AND revision = $2 AND frozen_at IS NOT NULL",
        )
        .bind(workspace_id.as_uuid())
        .bind(current_revision)
        .fetch_one(&mut *tx)
        .await
        .context("read frozen planning workspace revision")?;
        let revision_id: Uuid = revision.try_get("id")?;
        let state: Value = revision.try_get("state")?;
        if let Some(materialized) = query_scalar::<_, Value>(
            "SELECT board_task_ids FROM core.planning_workspace_materializations
             WHERE workspace_id = $1 AND revision_id = $2",
        )
        .bind(workspace_id.as_uuid())
        .bind(revision_id)
        .fetch_optional(&mut *tx)
        .await
        .context("read existing workspace materialization")?
        {
            let ids = materialized
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("workspace materialization has an invalid task list"))?
                .iter()
                .map(|id| {
                    id.as_str()
                        .ok_or_else(|| anyhow::anyhow!("workspace materialization has an invalid task id"))?
                        .parse::<Uuid>()
                        .context("parse materialized board task id")
                })
                .collect::<Result<Vec<_>>>()?;
            tx.commit().await.context("commit existing workspace materialization")?;
            return Ok(ids);
        }
        let tasks = state
            .get("tasks")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("approved workspace has no task set"))?;
        if tasks.is_empty() {
            bail!("approved workspace must contain at least one task");
        }
        let mut task_ids = HashMap::new();
        let mut created_ids = Vec::with_capacity(tasks.len());
        for task in tasks {
            let key = task
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("workspace task id is required"))?;
            let summary = task
                .get("summary")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("workspace task summary is required"))?;
            let workflow_id = task
                .get("workflowId")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("workspace task workflowId is required"))?
                .parse::<Uuid>()
                .context("parse workspace task workflow id")?;
            if !query_scalar::<_, bool>(
                "SELECT EXISTS(
                     SELECT 1 FROM workflows.workflow_defs
                     WHERE id = $1 AND project_id = $2
                 )",
            )
            .bind(workflow_id)
            .bind(project_id)
            .fetch_one(&mut *tx)
            .await
            .context("validate workspace task workflow")?
            {
                bail!("workspace task workflow does not belong to the project");
            }
            let repo_id = task
                .get("repoId")
                .and_then(Value::as_str)
                .map(|repo| repo.parse::<Uuid>().context("parse workspace task repo id"))
                .transpose()?;
            if let Some(repo_id) = repo_id {
                if !query_scalar::<_, bool>(
                    "SELECT EXISTS(
                         SELECT 1 FROM core.repos
                         WHERE id = $1 AND project_id = $2
                     )",
                )
                .bind(repo_id)
                .bind(project_id)
                .fetch_one(&mut *tx)
                .await
                .context("validate workspace task repository")?
                {
                    bail!("workspace task repository does not belong to the project");
                }
            }
            let task_id = Uuid::new_v4();
            query(
                "INSERT INTO board.tasks
                    (id, project_id, repo_id, summary, description, column_name, workflow_def_id)
                 VALUES ($1, $2, $3, $4, $5, 'ready', $6)",
            )
            .bind(task_id)
            .bind(project_id)
            .bind(repo_id)
            .bind(summary.trim())
            .bind(task.get("description").and_then(Value::as_str).unwrap_or_default())
            .bind(workflow_id)
            .execute(&mut *tx)
            .await
            .context("materialize workspace board task")?;
            query(
                "INSERT INTO board.task_transitions
                    (id, task_id, from_column, to_column, actor_kind)
                 VALUES ($1, $2, NULL, 'ready', 'human')",
            )
            .bind(Uuid::new_v4())
            .bind(task_id)
            .execute(&mut *tx)
            .await
            .context("record workspace task creation")?;
            task_ids.insert(key.to_owned(), task_id);
            created_ids.push(task_id);
        }
        for task in tasks {
            let task_key = task.get("id").and_then(Value::as_str).unwrap_or_default();
            let task_id = task_ids
                .get(task_key)
                .ok_or_else(|| anyhow::anyhow!("workspace task dependency owner is missing"))?;
            for dependency in task
                .get("after")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let dependency_key = dependency
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("workspace task dependency id is invalid"))?;
                let dependency_id = task_ids
                    .get(dependency_key)
                    .ok_or_else(|| anyhow::anyhow!("workspace task dependency is missing"))?;
                query(
                    "INSERT INTO board.task_dependencies
                        (task_id, blocked_by_task_id, workflow_node_id)
                     VALUES ($1, $2, 'planning-workspace')",
                )
                .bind(task_id)
                .bind(dependency_id)
                .execute(&mut *tx)
                .await
                .context("materialize workspace task dependency")?;
            }
        }
        let task_ids_json = Value::Array(
            created_ids
                .iter()
                .map(|id| Value::String(id.to_string()))
                .collect(),
        );
        query(
            "INSERT INTO core.planning_workspace_materializations
                (id, workspace_id, revision_id, board_task_ids)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id.as_uuid())
        .bind(revision_id)
        .bind(&task_ids_json)
        .execute(&mut *tx)
        .await
        .context("record workspace materialization")?;
        query(
            "UPDATE core.planning_workspace_revisions
             SET approved_at = now() WHERE id = $1",
        )
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .context("approve workspace revision")?;
        query(
            "UPDATE core.planning_workspaces
             SET lifecycle = 'approved', approved_revision_id = $2, updated_at = now()
             WHERE id = $1",
        )
        .bind(workspace_id.as_uuid())
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .context("approve planning workspace")?;
        query(
            "INSERT INTO core.planning_workspace_events
                (id, workspace_id, revision_id, kind, payload)
             VALUES ($1, $2, $3, 'approved', $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id.as_uuid())
        .bind(revision_id)
        .bind(json!({ "board_task_ids": task_ids_json }))
        .execute(&mut *tx)
        .await
        .context("record workspace approval")?;
        tx.commit().await.context("commit planning workspace approval")?;
        Ok(created_ids)
    }

    pub async fn delete_planning_workspace(
        &self,
        project_id: ProjectId,
        workspace_id: PlanningWorkspaceId,
    ) -> Result<()> {
        let lifecycle: Option<String> = sqlx::query_scalar(
            "SELECT lifecycle FROM core.planning_workspaces
             WHERE id = $1 AND project_id = $2",
        )
        .bind(workspace_id.as_uuid())
        .bind(project_id)
        .fetch_optional(self.pool())
        .await
        .context("check planning workspace deletion")?;
        let Some(lifecycle) = lifecycle else {
            bail!("planning workspace was not found in the project");
        };
        if !matches!(lifecycle.as_str(), "draft" | "in_progress") {
            bail!("planning workspace is not deletable in its current lifecycle");
        }
        query(
            "DELETE FROM core.planning_workspaces
             WHERE id = $1 AND project_id = $2 AND lifecycle IN ('draft', 'in_progress')",
        )
        .bind(workspace_id.as_uuid())
        .bind(project_id)
        .execute(self.pool())
        .await
        .context("delete planning workspace")?;
        Ok(())
    }
}
