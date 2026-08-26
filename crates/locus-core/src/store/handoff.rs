//! Durable ownership-transfer writes for handoff artifacts and session/run links.

use crate::{
    services::{artifact::ArtifactContent, handoff::HandoffRecord},
    store::Store,
};
use anyhow::{bail, Result};
use sqlx::query;

impl Store {
    /// Atomically persist the payload, link both runs, close the predecessor session, and mark the
    /// successor's `handed_off_from` edge. Artifact bodies are the serialized payload only; source
    /// artifacts remain references in `payload.artifacts`.
    pub async fn save_handoff(&self, record: &HandoffRecord) -> Result<()> {
        let artifact = record.artifact_row(record.project_id)?;
        let (body, blob_path, media_type, sha256) = match &artifact.content {
            ArtifactContent::Text(body) => (Some(body.as_str()), None, None, None),
            ArtifactContent::Blob {
                path,
                media_type,
                sha256,
            } => (
                None,
                Some(path.to_string_lossy().into_owned()),
                Some(media_type.as_str()),
                Some(sha256.as_str()),
            ),
        };
        let mut transaction = self.pool().begin().await?;
        query(
            "INSERT INTO agents.artifacts
             (id, run_id, kind, body, blob_path, media_type, sha256, derived_representation, summary)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(artifact.id)
        .bind(artifact.run_id)
        .bind(artifact.kind.database_name())
        .bind(body)
        .bind(blob_path)
        .bind(media_type)
        .bind(sha256)
        .bind(&artifact.derived_cache)
        .bind(&artifact.summary)
        .execute(&mut *transaction)
        .await?;

        query(
            "INSERT INTO agents.run_edges (parent_run_id, child_run_id, edge_type)
             VALUES ($1, $2, 'handed_off')",
        )
        .bind(record.predecessor_run_id)
        .bind(record.successor_run_id)
        .execute(&mut *transaction)
        .await?;

        let predecessor = query(
            "UPDATE agents.sessions
             SET status = 'closed', closed_at = now()
             WHERE id = $1 AND status = 'active'",
        )
        .bind(record.predecessor_session_id)
        .execute(&mut *transaction)
        .await?;
        if predecessor.rows_affected() != 1 {
            bail!("handoff predecessor session is not active")
        }

        let successor = query(
            "UPDATE agents.sessions
             SET handed_off_from = $1
             WHERE id = $2 AND status = 'active'",
        )
        .bind(record.predecessor_session_id)
        .bind(record.successor_session_id)
        .execute(&mut *transaction)
        .await?;
        if successor.rows_affected() != 1 {
            bail!("handoff successor session is not active")
        }
        transaction.commit().await?;
        Ok(())
    }
}
