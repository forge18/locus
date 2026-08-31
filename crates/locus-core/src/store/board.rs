//! Project-scoped board task projections and manual task creation.

use anyhow::{bail, Context, Result};
use sqlx::{query, query_as, query_scalar};
use uuid::Uuid;

use crate::{ids::ProjectId, store::Store};

#[derive(Debug, sqlx::FromRow)]
pub struct BoardTaskRow {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub repo_id: Option<Uuid>,
    pub summary: String,
    pub column_name: String,
    pub blocked: bool,
    pub assigned_agent: Option<String>,
    pub session_id: Option<Uuid>,
    pub verify_command: Option<String>,
    pub workflow_id: Option<Uuid>,
    pub child_run_ids: Vec<Uuid>,
    pub evidence_ids: Vec<Uuid>,
    pub external_link: Option<String>,
}

impl Store {
    /// Read board tasks and their persisted run/evidence links for one project.
    pub async fn board_tasks(&self, project_id: ProjectId) -> Result<Vec<BoardTaskRow>> {
        query_as(
            "SELECT t.id,
                    t.project_id,
                    t.repo_id,
                    t.summary,
                    t.column_name,
                    t.blocked,
                    CASE WHEN ad.id IS NULL THEN NULL ELSE ad.name || '@' || ad.version::text END AS assigned_agent,
                    t.session_id,
                    t.verify_command,
                    t.workflow_def_id AS workflow_id,
                    COALESCE(runs.child_run_ids, ARRAY[]::uuid[]) AS child_run_ids,
                    COALESCE(evidence.evidence_ids, ARRAY[]::uuid[]) AS evidence_ids,
                    external_item.url AS external_link
             FROM board.tasks t
             LEFT JOIN agents.agent_defs ad ON ad.id = t.assigned_agent_def_id
             LEFT JOIN LATERAL (
                 SELECT array_agg(task_runs.run_id ORDER BY task_runs.run_id) AS child_run_ids
                 FROM board.task_runs task_runs
                 WHERE task_runs.task_id = t.id
             ) runs ON TRUE
             LEFT JOIN LATERAL (
                 SELECT array_agg(task_evidence.id ORDER BY task_evidence.created_at, task_evidence.id) AS evidence_ids
                 FROM board.task_evidence task_evidence
                 WHERE task_evidence.task_id = t.id
             ) evidence ON TRUE
             LEFT JOIN board.external_work_items external_item ON external_item.task_id = t.id
             WHERE t.project_id = $1
             ORDER BY t.updated_at DESC, t.id",
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list board tasks")
    }

    pub async fn board_task(
        &self,
        project_id: ProjectId,
        task_id: Uuid,
    ) -> Result<Option<BoardTaskRow>> {
        Ok(self
            .board_tasks(project_id)
            .await?
            .into_iter()
            .find(|task| task.id == task_id))
    }

    /// Create a Ready task only after validating its project-owned repo and workflow.
    /// The workflow id is validated here; later run creation records the selected definition.
    pub async fn create_board_task(
        &self,
        project_id: ProjectId,
        repo_id: Option<Uuid>,
        summary: &str,
        workflow_def_id: Uuid,
    ) -> Result<BoardTaskRow> {
        if summary.trim().is_empty() {
            bail!("task summary is required");
        }
        if !query_scalar::<_, bool>(
            "SELECT EXISTS(
                 SELECT 1 FROM workflows.workflow_defs
                 WHERE id = $1 AND project_id = $2
             )",
        )
        .bind(workflow_def_id)
        .bind(project_id)
        .fetch_one(self.pool())
        .await
        .context("validate task workflow project")?
        {
            bail!("workflow definition does not belong to the active project");
        }
        if let Some(repo_id) = repo_id {
            if query_scalar::<_, Option<ProjectId>>(
                "SELECT project_id FROM core.repos WHERE id = $1",
            )
            .bind(repo_id)
            .fetch_one(self.pool())
            .await
            .context("validate task repo")?
                != Some(project_id)
            {
                bail!("repository does not belong to the active project");
            }
        }

        let id = Uuid::new_v4();
        query(
            "INSERT INTO board.tasks
                (id, project_id, repo_id, summary, column_name, workflow_def_id)
             VALUES ($1, $2, $3, $4, 'ready', $5)",
        )
        .bind(id)
        .bind(project_id)
        .bind(repo_id)
        .bind(summary.trim())
        .bind(workflow_def_id)
        .execute(self.pool())
        .await
        .context("create board task")?;
        query(
            "INSERT INTO board.task_transitions
                (id, task_id, from_column, to_column, actor_kind)
             VALUES ($1, $2, NULL, 'ready', 'human')",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .execute(self.pool())
        .await
        .context("record task creation transition")?;
        self.board_task(project_id, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("created task disappeared"))
    }
}
