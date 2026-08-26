//! Durable ACP session-control projections.

use crate::{
    ids::RunId,
    runtime::controls::{Checkpoint, PermissionPosture},
    services::telemetry::PermissionGate,
};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use sqlx::{query, query_scalar};

use super::Store;

impl Store {
    /// Pins a run's permission posture while it is still queued.
    pub async fn set_run_permission_posture(
        &self,
        run_id: RunId,
        posture: PermissionPosture,
    ) -> Result<()> {
        let updated = query(
            "UPDATE agents.runs
             SET permission_posture = $2
             WHERE id = $1 AND status = 'queued'",
        )
        .bind(run_id)
        .bind(posture.as_str())
        .execute(self.pool())
        .await
        .context("persist run permission posture")?;
        if updated.rows_affected() == 0 {
            bail!("run is missing or has already started; permission posture is immutable")
        }
        Ok(())
    }

    pub async fn run_permission_posture(&self, run_id: RunId) -> Result<PermissionPosture> {
        let value: String =
            query_scalar("SELECT permission_posture FROM agents.runs WHERE id = $1")
                .bind(run_id)
                .fetch_one(self.pool())
                .await
                .context("read run permission posture")?;
        PermissionPosture::parse(&value)
    }

    pub async fn persist_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        query(
            "INSERT INTO agents.checkpoints (id, run_id, ordinal, workspace)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(checkpoint.id)
        .bind(checkpoint.run_id)
        .bind(i64::try_from(checkpoint.ordinal).context("checkpoint ordinal exceeds BIGINT")?)
        .bind(serde_json::to_value(&checkpoint.workspace).context("serialize checkpoint")?)
        .execute(self.pool())
        .await
        .context("persist checkpoint")?;
        Ok(())
    }

    pub async fn persist_permission_gate(&self, gate: &PermissionGate) -> Result<()> {
        query(
            "INSERT INTO agents.permission_gates (run_id, seq, request_id, diff, raw)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (run_id, seq) DO UPDATE SET
                 request_id = EXCLUDED.request_id,
                 diff = EXCLUDED.diff,
                 raw = EXCLUDED.raw",
        )
        .bind(gate.run_id)
        .bind(i64::try_from(gate.seq).context("permission gate sequence exceeds BIGINT")?)
        .bind(&gate.request_id)
        .bind(&gate.diff)
        .bind(&gate.raw)
        .execute(self.pool())
        .await
        .context("persist permission gate")?;
        Ok(())
    }

    pub async fn resolve_permission_gate(
        &self,
        run_id: RunId,
        seq: u64,
        resolution: Value,
    ) -> Result<()> {
        let updated = query(
            "UPDATE agents.permission_gates
             SET resolved_at = now(), resolution = $3
             WHERE run_id = $1 AND seq = $2",
        )
        .bind(run_id)
        .bind(i64::try_from(seq).context("permission gate sequence exceeds BIGINT")?)
        .bind(resolution)
        .execute(self.pool())
        .await
        .context("resolve permission gate")?;
        if updated.rows_affected() == 0 {
            bail!("permission gate not found")
        }
        Ok(())
    }
}
