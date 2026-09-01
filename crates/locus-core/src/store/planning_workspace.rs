//! Durable Planning Workspace wrapper projections.

use crate::ids::{PlanningWorkspaceId, ProjectId};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sqlx::{query, query_as, Row};
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
