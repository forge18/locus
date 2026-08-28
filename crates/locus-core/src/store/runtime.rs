//! Persistence for the machine-selected runtime recorded on each agent run.

use crate::{ids::RunId, runtime::backend::RuntimeBackend, store::Store};
use anyhow::{bail, Context, Result};

impl Store {
    pub async fn record_runtime_backend(
        &self,
        run_id: RunId,
        backend: RuntimeBackend,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE agents.runs
             SET runtime_backend = $2
             WHERE id = $1",
        )
        .bind(run_id)
        .bind(backend.as_str())
        .execute(self.pool())
        .await
        .context("record run runtime backend")?;
        if result.rows_affected() != 1 {
            bail!("run `{run_id}` does not exist")
        }
        Ok(())
    }

    pub async fn run_runtime_backend(&self, run_id: RunId) -> Result<RuntimeBackend> {
        let value = sqlx::query_scalar::<_, String>(
            "SELECT runtime_backend FROM agents.runs WHERE id = $1",
        )
        .bind(run_id)
        .fetch_optional(self.pool())
        .await
        .context("read run runtime backend")?
        .ok_or_else(|| anyhow::anyhow!("run `{run_id}` does not exist"))?;
        value
            .parse()
            .map_err(|error| anyhow::anyhow!("invalid runtime backend on run `{run_id}`: {error}"))
    }
}
