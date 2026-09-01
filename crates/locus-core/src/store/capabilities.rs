//! Persistence for project capability policy revisions and run snapshots.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use sqlx::{query, query_scalar, Row};

use crate::{
    ids::{ProjectId, RunId},
    services::capabilities::CapabilityPolicies,
    store::Store,
};

impl Store {
    pub async fn project_capability_policy_revision(
        &self,
        project_id: ProjectId,
    ) -> Result<i32> {
        query(
            "INSERT INTO core.project_capability_policies (project_id)
             VALUES ($1)
             ON CONFLICT (project_id) DO NOTHING",
        )
        .bind(project_id)
        .execute(self.pool())
        .await
        .context("ensure project capability policy")?;
        query_scalar(
            "SELECT revision FROM core.project_capability_policies
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(self.pool())
        .await
        .context("read project capability policy revision")
    }

    pub async fn project_capability_policies(
        &self,
        project_id: ProjectId,
    ) -> Result<(i32, CapabilityPolicies)> {
        let revision = self.project_capability_policy_revision(project_id).await?;
        let row = query(
            "SELECT policies FROM core.project_capability_policies
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(self.pool())
        .await
        .context("read project capability policies")?;
        let policies = serde_json::from_value(row.try_get("policies")?)
            .context("decode project capability policies")?;
        Ok((revision, policies))
    }

    pub async fn set_project_capability_policy(
        &self,
        project_id: ProjectId,
        policies: Value,
    ) -> Result<i32> {
        if !policies.is_object() {
            bail!("project capability policies must be a JSON object");
        }
        let revision = query_scalar::<_, i32>(
            "INSERT INTO core.project_capability_policies
                (project_id, revision, policies)
             VALUES ($1, 1, $2)
             ON CONFLICT (project_id) DO UPDATE SET
                 revision = core.project_capability_policies.revision + 1,
                 policies = EXCLUDED.policies,
                 updated_at = now()
             RETURNING revision",
        )
        .bind(project_id)
        .bind(policies)
        .fetch_one(self.pool())
        .await
        .context("save project capability policy")?;
        Ok(revision)
    }

    pub async fn save_project_capability_policies(
        &self,
        project_id: ProjectId,
        policies: CapabilityPolicies,
    ) -> Result<i32> {
        let settings = self
            .project_settings(project_id)
            .await?
            .with_capability_policies(policies.clone());
        self.set_project_settings(project_id, &settings).await?;
        self.set_project_capability_policy(project_id, serde_json::to_value(policies)?)
            .await
    }

    pub async fn record_run_capability_snapshot(
        &self,
        run_id: RunId,
        policy_revision: i32,
        snapshot: Value,
    ) -> Result<()> {
        if !snapshot.is_object() {
            bail!("run capability snapshot must be a JSON object");
        }
        query(
            "UPDATE agents.runs
             SET capability_policy_revision = $2, capability_snapshot = $3
             WHERE id = $1",
        )
        .bind(run_id)
        .bind(policy_revision)
        .bind(snapshot)
        .execute(self.pool())
        .await
        .context("record run capability snapshot")?;
        Ok(())
    }
}
