//! Durable Interact session queries and terminal transitions.

use crate::{
    ids::{ProjectId, RunId, SessionId, TaskId},
    services::interact::{InteractSession, InteractState},
    store::Store,
};
use anyhow::{bail, Context, Result};
use sqlx::{query, query_scalar, Row};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct InteractSessionRow {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub project: String,
    pub name: String,
    pub agent: String,
    pub harness: String,
    pub branch: String,
    pub status: String,
    pub state: InteractState,
    pub board_task_id: Option<TaskId>,
    pub run_id: Option<RunId>,
    pub run_status: Option<String>,
    pub model: Option<String>,
    pub permission_posture: String,
    pub created_at: Option<String>,
    pub repo: Option<String>,
    pub workspace_remote: Option<String>,
    pub container_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InteractDiscardTarget {
    pub branch: String,
    pub workspace_remote: Option<String>,
    pub container_id: Option<String>,
}

fn state(value: &str) -> InteractState {
    match value {
        "promoted" => InteractState::Promoted,
        "discarded" => InteractState::Discarded,
        _ => InteractState::Open,
    }
}

fn session_row(row: &sqlx::postgres::PgRow) -> Result<InteractSessionRow> {
    Ok(InteractSessionRow {
        id: row.try_get::<Uuid, _>("id")?.into(),
        project_id: row.try_get::<Uuid, _>("project_id")?.into(),
        project: row.try_get("project")?,
        name: row.try_get("name")?,
        agent: row.try_get("agent")?,
        harness: row
            .try_get::<Option<String>, _>("harness")?
            .unwrap_or_else(|| "unknown".into()),
        branch: row.try_get("branch")?,
        status: row.try_get("status")?,
        state: state(row.try_get("interact_state")?),
        board_task_id: row
            .try_get::<Option<Uuid>, _>("board_task_id")?
            .map(Into::into),
        run_id: row.try_get::<Option<Uuid>, _>("run_id")?.map(Into::into),
        run_status: row.try_get("run_status")?,
        model: row.try_get("model")?,
        permission_posture: row
            .try_get::<Option<String>, _>("permission_posture")?
            .unwrap_or_else(|| "bypass".into()),
        created_at: row.try_get("created_at")?,
        repo: row.try_get("repo")?,
        workspace_remote: row.try_get("workspace_remote")?,
        container_id: row.try_get("container_id")?,
    })
}

const INTERACT_SESSION_QUERY: &str = "SELECT s.id, s.project_id, p.name AS project, s.name,
       ad.name AS agent, ad.frontmatter ->> 'harness' AS harness, s.branch, s.status,
       s.interact_state, s.board_task_id, latest.id AS run_id, latest.status AS run_status,
       latest.resolved_model_id AS model, latest.permission_posture,
       s.created_at::text AS created_at, repo.name AS repo, remote.bare_path AS workspace_remote,
       latest.container_id
FROM agents.sessions s
JOIN core.projects p ON p.id = s.project_id
JOIN agents.agent_defs ad ON ad.id = s.agent_def_id
LEFT JOIN LATERAL (
    SELECT r.id, r.status, r.resolved_model_id, r.permission_posture, r.container_id
    FROM agents.runs r
    WHERE r.session_id = s.id
    ORDER BY r.created_at DESC
    LIMIT 1
) latest ON TRUE
LEFT JOIN LATERAL (
    SELECT repos.name, repos.id
    FROM core.repos repos
    WHERE repos.project_id = s.project_id
      AND (s.repo_id IS NULL OR repos.id = s.repo_id)
    ORDER BY repos.name, repos.id
    LIMIT 1
) repo ON TRUE
LEFT JOIN LATERAL (
    SELECT remotes.bare_path
    FROM core.local_remotes remotes
    WHERE remotes.repo_id = repo.id
    ORDER BY remotes.bare_path
    LIMIT 1
) remote ON TRUE
WHERE s.branch LIKE 'interact/%'
  AND ($1::uuid IS NULL OR s.project_id = $1)
  AND ($2::uuid IS NULL OR s.id = $2)
ORDER BY s.created_at DESC";

impl Store {
    pub async fn interact_sessions(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<InteractSessionRow>> {
        let rows = query(INTERACT_SESSION_QUERY)
            .bind(project_id)
            .bind(Option::<Uuid>::None)
            .fetch_all(self.pool())
            .await
            .context("list Interact sessions")?;
        rows.iter().map(session_row).collect()
    }

    pub async fn interact_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<InteractSessionRow>> {
        self.read_interact_session(None, session_id).await
    }

    pub async fn interact_session_for_project(
        &self,
        project_id: ProjectId,
        session_id: SessionId,
    ) -> Result<Option<InteractSessionRow>> {
        self.read_interact_session(Some(project_id), session_id)
            .await
    }

    async fn read_interact_session(
        &self,
        project_id: Option<ProjectId>,
        session_id: SessionId,
    ) -> Result<Option<InteractSessionRow>> {
        let row = query(INTERACT_SESSION_QUERY)
            .bind(project_id)
            .bind(session_id)
            .fetch_optional(self.pool())
            .await
            .context("read Interact session")?;
        row.as_ref().map(session_row).transpose()
    }

    /// Create an open, board-less session and queue its first ACP run.
    pub async fn active_interact_run(
        &self,
        project_id: ProjectId,
        session_id: SessionId,
    ) -> Result<Option<RunId>> {
        query_scalar::<_, Uuid>(
            "SELECT run.id
             FROM agents.runs run
             JOIN agents.sessions session ON session.id = run.session_id
             WHERE run.session_id = $1 AND session.project_id = $2
               AND session.branch LIKE 'interact/%' AND run.status = 'running'
             ORDER BY run.created_at DESC
             LIMIT 1",
        )
        .bind(session_id)
        .bind(project_id)
        .fetch_optional(self.pool())
        .await
        .map(|run_id| run_id.map(Into::into))
        .context("read active Interact run")
    }

    /// Create an open, board-less session and queue its first ACP run.
    pub async fn create_interact_session(
        &self,
        project_id: ProjectId,
        repo_id: Option<Uuid>,
        name: &str,
        model: &str,
    ) -> Result<SessionId> {
        if name.trim().is_empty() {
            bail!("Interact session name must not be empty")
        }
        if model.trim().is_empty() {
            bail!("Interact session model must not be empty")
        }
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin Interact session")?;
        let agent_def_id: Uuid =
            query("SELECT id FROM agents.agent_defs ORDER BY name, version DESC, id DESC LIMIT 1")
                .fetch_optional(&mut *tx)
                .await
                .context("find default Interact agent")?
                .ok_or_else(|| anyhow::anyhow!("no agent definition is configured"))?
                .try_get("id")?;
        let repo_id = match repo_id {
            Some(repo_id) => {
                let owned = query("SELECT id FROM core.repos WHERE id = $1 AND project_id = $2")
                    .bind(repo_id)
                    .bind(project_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .context("check Interact repository ownership")?;
                let owned: Option<Uuid> =
                    owned.map(|row| row.try_get::<Uuid, _>("id")).transpose()?;
                owned.ok_or_else(|| anyhow::anyhow!("repository was not found in the project"))?
            }
            None => {
                query("SELECT id FROM core.repos WHERE project_id = $1 ORDER BY name, id LIMIT 1")
                    .bind(project_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .context("find default Interact repository")?
                    .ok_or_else(|| anyhow::anyhow!("no repository is configured for the project"))?
                    .try_get("id")?
            }
        };
        let session_id = SessionId::generate();
        let run_id = RunId::generate();
        let branch = format!("interact/{session_id}");
        query(
            "INSERT INTO agents.sessions
                (id, project_id, repo_id, agent_def_id, name, branch, interact_state)
             VALUES ($1, $2, $3, $4, $5, $6, 'open')",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(repo_id)
        .bind(agent_def_id)
        .bind(name)
        .bind(&branch)
        .execute(&mut *tx)
        .await
        .context("create Interact session")?;
        query(
            "INSERT INTO agents.runs
                (id, session_id, agent_def_id, resolved_model_id, status)
             VALUES ($1, $2, $3, $4, 'queued')",
        )
        .bind(run_id)
        .bind(session_id)
        .bind(agent_def_id)
        .bind(model)
        .execute(&mut *tx)
        .await
        .context("create Interact run")?;
        query(
            "INSERT INTO agents.dispatch_queue
                (run_id, plan_order, manual_order, unblocks_count, estimate_minutes)
             VALUES ($1, 0, 0, 0, 0)",
        )
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .context("queue Interact run")?;
        tx.commit().await.context("commit Interact session")?;
        Ok(session_id)
    }

    /// Attach an existing board task, or create a minimal ready task when no id is supplied.
    pub async fn promote_interact_session(
        &self,
        project_id: ProjectId,
        session_id: SessionId,
        task_id: Option<TaskId>,
    ) -> Result<TaskId> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin Interact promotion")?;
        let session = query(
            "SELECT project_id, agent_def_id, name
             FROM agents.sessions
             WHERE id = $1 AND project_id = $2 AND branch LIKE 'interact/%'
               AND interact_state = 'open' AND board_task_id IS NULL
             FOR UPDATE",
        )
        .bind(session_id)
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await
        .context("read promotable Interact session")?
        .ok_or_else(|| anyhow::anyhow!("only an open Interact session can be promoted"))?;
        let session_project_id: Uuid = session.try_get("project_id")?;
        let agent_def_id: Uuid = session.try_get("agent_def_id")?;
        let name: String = session.try_get("name")?;
        let task_id = match task_id {
            Some(task_id) => {
                let owner = query(
                    "SELECT session_id FROM board.tasks
                     WHERE id = $1 AND project_id = $2
                     FOR UPDATE",
                )
                .bind(task_id)
                .bind(session_project_id)
                .fetch_optional(&mut *tx)
                .await
                .context("check Interact task ownership")?
                .ok_or_else(|| anyhow::anyhow!("board task was not found in the session project"))?
                .try_get::<Option<Uuid>, _>("session_id")?;
                let session_uuid: Uuid = session_id.into();
                if owner.is_some_and(|owner| owner != session_uuid) {
                    bail!("board task is already attached to another session")
                }
                task_id
            }
            None => {
                let task_id = TaskId::generate();
                query(
                    "INSERT INTO board.tasks
                        (id, project_id, summary, assigned_agent_def_id, session_id)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(task_id)
                .bind(session_project_id)
                .bind(name)
                .bind(agent_def_id)
                .bind(session_id)
                .execute(&mut *tx)
                .await
                .context("create promoted Interact task")?;
                task_id
            }
        };
        let attached = query(
            "UPDATE board.tasks SET session_id = $2
             WHERE id = $1 AND (session_id IS NULL OR session_id = $2)",
        )
        .bind(task_id)
        .bind(session_id)
        .execute(&mut *tx)
        .await
        .context("attach promoted Interact task")?;
        if attached.rows_affected() != 1 {
            bail!("board task was attached concurrently")
        }
        query(
            "UPDATE agents.sessions
             SET board_task_id = $2, interact_state = 'promoted'
             WHERE id = $1",
        )
        .bind(session_id)
        .bind(task_id)
        .execute(&mut *tx)
        .await
        .context("promote Interact session")?;
        tx.commit().await.context("commit Interact promotion")?;
        Ok(task_id)
    }

    /// Mark the session terminal and return the host resources that must be destroyed.
    pub async fn discard_interact_session(
        &self,
        project_id: ProjectId,
        session_id: SessionId,
    ) -> Result<InteractDiscardTarget> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin discard Interact session")?;
        let state_row = query(
            "SELECT interact_state FROM agents.sessions
             WHERE id = $1 AND project_id = $2 AND branch LIKE 'interact/%' FOR UPDATE",
        )
        .bind(session_id)
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await
        .context("lock discard Interact session")?;
        if state_row
            .as_ref()
            .and_then(|row| row.try_get::<String, _>("interact_state").ok())
            .as_deref()
            != Some("open")
        {
            bail!("only an open Interact session can be discarded")
        }
        let row = query(INTERACT_SESSION_QUERY)
            .bind(project_id)
            .bind(session_id)
            .fetch_one(&mut *tx)
            .await
            .context("read discard Interact session")?;
        let target = InteractDiscardTarget {
            branch: row.try_get("branch")?,
            workspace_remote: row.try_get("workspace_remote")?,
            container_id: row.try_get("container_id")?,
        };
        query(
            "UPDATE agents.sessions
             SET interact_state = 'discarded', status = 'closed', closed_at = now()
             WHERE id = $1 AND project_id = $2",
        )
        .bind(session_id)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .context("discard Interact session")?;
        query(
            "UPDATE agents.runs
             SET status = 'cancelled', cancel_reason = 'interact session discarded', ended_at = now()
             WHERE session_id = $1 AND status IN ('queued', 'running', 'paused')",
        )
        .bind(session_id)
        .execute(&mut *tx)
        .await
        .context("cancel discarded Interact runs")?;
        query(
            "DELETE FROM agents.dispatch_queue
             WHERE run_id IN (SELECT id FROM agents.runs WHERE session_id = $1)",
        )
        .bind(session_id)
        .execute(&mut *tx)
        .await
        .context("remove discarded Interact runs from queue")?;
        tx.commit()
            .await
            .context("commit discard Interact session")?;
        Ok(target)
    }

    pub async fn commit_interact_session(
        &self,
        project_id: ProjectId,
        session_id: SessionId,
    ) -> Result<InteractSessionRow> {
        self.interact_session_for_project(project_id, session_id)
            .await?
            .filter(|session| session.state == InteractState::Open)
            .ok_or_else(|| anyhow::anyhow!("only an open Interact session can be committed"))
    }

    pub async fn open_interact_session(
        &self,
        session_id: SessionId,
        project_id: ProjectId,
        agent_def_id: uuid::Uuid,
        name: &str,
        repo: &str,
    ) -> Result<InteractSession> {
        let branch = format!("interact/{session_id}");
        query("INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch, interact_state) VALUES ($1, $2, $3, $4, $5, 'open')")
            .bind(session_id).bind(project_id).bind(agent_def_id).bind(name).bind(&branch).execute(self.pool()).await.context("open Interact session")?;
        Ok(InteractSession::open(session_id, project_id, repo))
    }
}
