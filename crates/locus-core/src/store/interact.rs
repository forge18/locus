//! Durable Interact session transitions.

use crate::{
    ids::{ProjectId, SessionId, TaskId},
    services::interact::{InteractSession, InteractState},
    store::Store,
};
use anyhow::{bail, Context, Result};
use sqlx::{query, Row};

impl Store {
    pub async fn interact_session(&self, session_id: SessionId) -> Result<InteractSession> {
        let row = query("SELECT id, project_id, branch, board_task_id, interact_state FROM agents.sessions WHERE id = $1")
            .bind(session_id).fetch_one(self.pool()).await.context("read Interact session")?;
        Ok(InteractSession {
            id: row.try_get("id")?,
            project_id: row.try_get("project_id")?,
            repo: String::new(),
            branch: row.try_get("branch")?,
            board_task_id: row.try_get("board_task_id")?,
            container_id: None,
            state: match row.try_get::<&str, _>("interact_state")? {
                "promoted" => InteractState::Promoted,
                "discarded" => InteractState::Discarded,
                _ => InteractState::Open,
            },
        })
    }

    pub async fn promote_interact_session(
        &self,
        session_id: SessionId,
        task_id: TaskId,
    ) -> Result<()> {
        let updated = query("UPDATE agents.sessions SET board_task_id = $2, interact_state = 'promoted' WHERE id = $1 AND interact_state = 'open' AND board_task_id IS NULL")
            .bind(session_id).bind(task_id).execute(self.pool()).await.context("promote Interact session")?;
        if updated.rows_affected() != 1 {
            bail!("only an open Interact session can be promoted");
        }
        Ok(())
    }

    pub async fn discard_interact_session(&self, session_id: SessionId) -> Result<String> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin discard Interact session")?;
        let row = query("UPDATE agents.sessions SET interact_state = 'discarded', status = 'closed', closed_at = now() WHERE id = $1 AND interact_state = 'open' RETURNING branch")
            .bind(session_id).fetch_optional(&mut *tx).await.context("discard Interact session")?;
        let Some(row) = row else {
            bail!("only an open Interact session can be discarded");
        };
        query("UPDATE agents.runs SET status = 'cancelled', cancel_reason = 'interact session discarded' WHERE session_id = $1 AND status IN ('queued', 'running', 'paused')")
            .bind(session_id).execute(&mut *tx).await.context("cancel discarded Interact runs")?;
        let branch: String = row.try_get("branch")?;
        tx.commit()
            .await
            .context("commit discard Interact session")?;
        Ok(branch)
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
