//! Durable installation-wide guardrail and dispatch policy settings.

use anyhow::{bail, Context, Result};
use sqlx::{query, query_as};

use crate::store::Store;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GuardrailDefaultsRow {
    pub max_iterations: i32,
    pub token_budget: Option<i64>,
    pub stuck_iterations: i32,
    pub kill_and_reassign: bool,
    pub change_lines_ceiling: Option<i32>,
    pub change_files_ceiling: Option<i32>,
    pub network_tier: String,
    pub block_system_changes: bool,
    pub autopilot: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DispatchPolicyRow {
    pub global_parallelism: i32,
    pub per_project_parallelism: i32,
    pub priority_method: String,
    pub tie_break: String,
}

#[derive(Debug, Clone)]
pub struct GuardrailSettings {
    pub defaults: GuardrailDefaultsRow,
    pub dispatch: DispatchPolicyRow,
}

impl Store {
    pub async fn guardrail_settings(&self) -> Result<GuardrailSettings> {
        let defaults = query_as::<_, GuardrailDefaultsRow>(
            "SELECT max_iterations, token_budget, stuck_iterations, kill_and_reassign,
                    change_lines_ceiling, change_files_ceiling, network_tier,
                    block_system_changes, autopilot
             FROM core.guardrail_defaults
             WHERE singleton",
        )
        .fetch_one(self.pool())
        .await
        .context("read guardrail defaults")?;
        let dispatch = query_as::<_, DispatchPolicyRow>(
            "SELECT global_parallelism, per_project_parallelism, priority_method, tie_break
             FROM core.dispatch_policy
             WHERE singleton",
        )
        .fetch_one(self.pool())
        .await
        .context("read dispatch policy")?;
        Ok(GuardrailSettings { defaults, dispatch })
    }

    pub async fn set_guardrail_settings(
        &self,
        defaults: &GuardrailDefaultsRow,
        dispatch: &DispatchPolicyRow,
    ) -> Result<GuardrailSettings> {
        if defaults.max_iterations <= 0 || defaults.stuck_iterations <= 0 {
            bail!("iteration guardrails must be greater than zero");
        }
        if defaults.token_budget.is_some_and(|budget| budget <= 0)
            || defaults
                .change_lines_ceiling
                .is_some_and(|ceiling| ceiling < 0)
            || defaults
                .change_files_ceiling
                .is_some_and(|ceiling| ceiling < 0)
        {
            bail!("guardrail limits must not be negative");
        }
        if dispatch.global_parallelism <= 0 || dispatch.per_project_parallelism <= 0 {
            bail!("parallelism limits must be greater than zero");
        }
        let mut transaction = self.pool().begin().await.context("begin guardrail update")?;
        query(
            "UPDATE core.guardrail_defaults
             SET max_iterations = $1, token_budget = $2, stuck_iterations = $3,
                 kill_and_reassign = $4, change_lines_ceiling = $5,
                 change_files_ceiling = $6, network_tier = $7,
                 block_system_changes = $8, autopilot = $9, updated_at = now()
             WHERE singleton",
        )
        .bind(defaults.max_iterations)
        .bind(defaults.token_budget)
        .bind(defaults.stuck_iterations)
        .bind(defaults.kill_and_reassign)
        .bind(defaults.change_lines_ceiling)
        .bind(defaults.change_files_ceiling)
        .bind(&defaults.network_tier)
        .bind(defaults.block_system_changes)
        .bind(defaults.autopilot)
        .execute(&mut *transaction)
        .await
        .context("update guardrail defaults")?;
        query(
            "UPDATE core.dispatch_policy
             SET global_parallelism = $1, per_project_parallelism = $2,
                 priority_method = $3, tie_break = $4
             WHERE singleton",
        )
        .bind(dispatch.global_parallelism)
        .bind(dispatch.per_project_parallelism)
        .bind(&dispatch.priority_method)
        .bind(&dispatch.tie_break)
        .execute(&mut *transaction)
        .await
        .context("update dispatch policy")?;
        transaction
            .commit()
            .await
            .context("commit guardrail update")?;
        self.guardrail_settings().await
    }
}
