//! Durable adapters for curated memory revisions and retrieval feedback.

use crate::{
    ids::ProjectId,
    services::memory::{ConfidenceState, FactRevision},
    store::Store,
};
use anyhow::{Context, Result};
use sqlx::{query, query_as};
use uuid::Uuid;

fn confidence_name(state: ConfidenceState) -> &'static str {
    match state {
        ConfidenceState::Verified => "verified",
        ConfidenceState::Asserted => "asserted",
        ConfidenceState::Decaying => "decaying",
        ConfidenceState::Contradicted => "contradicted",
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct MemoryFactRow {
    pub id: Uuid,
    pub subject: String,
    pub confidence_state: String,
    pub score: Option<f64>,
    pub recall_count: i32,
}

impl Store {
    pub async fn memory_facts(&self, project_id: ProjectId) -> Result<Vec<MemoryFactRow>> {
        query_as(
            "SELECT id, subject, confidence_state,
                    CASE WHEN confidence_state = 'contradicted' THEN NULL ELSE confidence END AS score,
                    recall_count
             FROM memory.store
             WHERE project_id = $1
               AND invalidated_at IS NULL
               AND archived_at IS NULL
             ORDER BY updated_at DESC, id",
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list project memory facts")
    }

    pub async fn append_memory_revision(
        &self,
        fact_id: Uuid,
        revision: &FactRevision,
    ) -> Result<()> {
        let mut tx = self.pool().begin().await.context("begin memory revision")?;
        query("INSERT INTO memory.fact_revisions (fact_id, rev, value, written_by_run, curated_by, score, created_at) VALUES ($1, $2, $3, $4, $5, $6, to_timestamp($7))")
            .bind(fact_id).bind(revision.rev as i32).bind(&revision.value).bind(revision.written_by_run.map(|id| id.as_uuid())).bind(&revision.curated_by).bind(revision.score).bind(revision.written_at as f64)
            .execute(&mut *tx).await.context("append memory revision")?;
        query("UPDATE memory.store SET current_revision = $2, updated_at = now() WHERE id = $1")
            .bind(fact_id)
            .bind(revision.rev as i32)
            .execute(&mut *tx)
            .await
            .context("advance memory revision")?;
        tx.commit().await.context("commit memory revision")?;
        Ok(())
    }

    pub async fn set_memory_confidence(&self, fact_id: Uuid, state: ConfidenceState) -> Result<()> {
        query("UPDATE memory.store SET confidence_state = $2, updated_at = now() WHERE id = $1")
            .bind(fact_id)
            .bind(confidence_name(state))
            .execute(self.pool())
            .await
            .context("set memory confidence")?;
        Ok(())
    }

    pub async fn set_memory_confidence_for_project(
        &self,
        project_id: ProjectId,
        fact_id: Uuid,
        state: ConfidenceState,
    ) -> Result<bool> {
        let updated = query(
            "UPDATE memory.store
             SET confidence_state = $3, updated_at = now()
             WHERE project_id = $1 AND id = $2
             RETURNING id",
        )
        .bind(project_id)
        .bind(fact_id)
        .bind(confidence_name(state))
        .fetch_optional(self.pool())
        .await
        .context("set project memory confidence")?;
        Ok(updated.is_some())
    }

    pub async fn record_memory_feedback(
        &self,
        id: Uuid,
        project_id: Uuid,
        run_id: Option<Uuid>,
        fact_id: Option<Uuid>,
        useful: Option<bool>,
        changed_answer: Option<bool>,
    ) -> Result<()> {
        query("INSERT INTO memory.retrieval_feedback (id, project_id, run_id, fact_id, useful, changed_answer) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(id).bind(project_id).bind(run_id).bind(fact_id).bind(useful).bind(changed_answer).execute(self.pool()).await.context("record memory feedback")?;
        Ok(())
    }
}
