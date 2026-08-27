//! Durable adapters for external work-item snapshots and completion delivery.

use crate::{
    ids::TaskId,
    store::Store,
    work_item::{
        CompletionDelivery, WorkItemProviderConfig, WorkItemProviderId, WorkItemSnapshot,
    },
};
use anyhow::{anyhow, Context, Result};
use sqlx::{query, query_as};

impl Store {
    pub async fn save_external_work_item_provider(
        &self,
        config: &WorkItemProviderConfig,
    ) -> Result<()> {
        query(
            "INSERT INTO board.external_work_item_providers
             (plugin_id, host, provider_project)
             VALUES ($1, $2, $3)
             ON CONFLICT (plugin_id) DO UPDATE
             SET host = EXCLUDED.host,
                 provider_project = EXCLUDED.provider_project,
                 configured_at = now()",
        )
        .bind(config.plugin_id.as_str())
        .bind(&config.host)
        .bind(&config.project)
        .execute(self.pool())
        .await
        .context("save external work-item provider")?;
        Ok(())
    }

    pub async fn load_external_work_item_providers(&self) -> Result<Vec<WorkItemProviderConfig>> {
        let rows = query_as::<_, (String, String, String)>(
            "SELECT plugin_id, host, provider_project
             FROM board.external_work_item_providers
             ORDER BY plugin_id",
        )
        .fetch_all(self.pool())
        .await
        .context("load external work-item providers")?;

        rows.into_iter()
            .map(|(plugin_id, host, project)| {
                let plugin_id = WorkItemProviderId::new(plugin_id)
                    .map_err(|error| anyhow!("invalid external provider id: {error}"))?;
                WorkItemProviderConfig::new(plugin_id.as_str(), host, project)
                    .map_err(|error| anyhow!("invalid external provider configuration: {error}"))
            })
            .collect()
    }

    pub async fn persist_external_work_item(
        &self,
        task_id: TaskId,
        snapshot: &WorkItemSnapshot,
    ) -> Result<bool> {
        let inserted = query(
            "INSERT INTO board.external_work_items
             (task_id, plugin_id, host, provider_project, native_id, url, title, body, labels, source_status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (plugin_id, host, provider_project, native_id) DO NOTHING",
        )
        .bind(task_id)
        .bind(snapshot.identity.plugin_id.as_str())
        .bind(&snapshot.identity.host)
        .bind(&snapshot.identity.project)
        .bind(&snapshot.identity.native_id)
        .bind(&snapshot.url)
        .bind(&snapshot.title)
        .bind(&snapshot.body)
        .bind(serde_json::to_value(&snapshot.labels)?)
        .bind(&snapshot.status)
        .execute(self.pool())
        .await
        .context("persist external work-item snapshot")?;
        Ok(inserted.rows_affected() == 1)
    }

    pub async fn enqueue_external_completion(
        &self,
        delivery: &CompletionDelivery,
        snapshot: &WorkItemSnapshot,
    ) -> Result<bool> {
        let inserted = query(
            "INSERT INTO board.external_completion_outbox
             (id, task_id, plugin_id, host, provider_project, native_id, comment, evidence, attempts)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (task_id) DO NOTHING",
        )
        .bind(delivery.event.id)
        .bind(delivery.event.task_id)
        .bind(snapshot.identity.plugin_id.as_str())
        .bind(&snapshot.identity.host)
        .bind(&snapshot.identity.project)
        .bind(&snapshot.identity.native_id)
        .bind(&delivery.event.comment)
        .bind(serde_json::to_value(&delivery.event.evidence)?)
        .bind(delivery.attempts as i32)
        .execute(self.pool())
        .await
        .context("enqueue external completion")?;
        Ok(inserted.rows_affected() == 1)
    }

    pub async fn record_external_completion_attempt(
        &self,
        event_id: uuid::Uuid,
        attempts: i32,
        commented: bool,
        resolved: bool,
        error: Option<&str>,
    ) -> Result<()> {
        query(
            "UPDATE board.external_completion_outbox
             SET attempts = $2,
                 commented_at = CASE WHEN $3 THEN COALESCE(commented_at, now()) ELSE commented_at END,
                 resolved_at = CASE WHEN $4 THEN COALESCE(resolved_at, now()) ELSE resolved_at END,
                 last_error = $5
             WHERE id = $1",
        )
        .bind(event_id)
        .bind(attempts)
        .bind(commented)
        .bind(resolved)
        .bind(error)
        .execute(self.pool())
        .await
        .context("record external completion attempt")?;
        Ok(())
    }
}
