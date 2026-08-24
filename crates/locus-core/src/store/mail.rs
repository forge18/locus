//! Durable adapters for mail threads, deliveries, and waits.

use anyhow::{bail, Context, Result};
use sqlx::query;
use uuid::Uuid;
use crate::{ids::RunId, store::Store};

impl Store {
    pub async fn create_mail_thread(&self, id: Uuid, project_id: Uuid, subject: &str) -> Result<()> {
        if subject.trim().is_empty() { bail!("mail subject is required"); }
        query("INSERT INTO mail.threads (id, project_id, subject) VALUES ($1, $2, $3)").bind(id).bind(project_id).bind(subject).execute(self.pool()).await.context("create mail thread")?;
        Ok(())
    }

    pub async fn append_mail_message(&self, id: Uuid, thread_id: Uuid, sender_kind: &str, sender_run: Option<RunId>, body: &str) -> Result<()> {
        if body.trim().is_empty() { bail!("mail body is required"); }
        query("INSERT INTO mail.messages (id, thread_id, sender_kind, sender_run_id, body) VALUES ($1, $2, $3, $4, $5)").bind(id).bind(thread_id).bind(sender_kind).bind(sender_run.map(|run| run.as_uuid())).bind(body).execute(self.pool()).await.context("append mail message")?;
        Ok(())
    }

    pub async fn create_human_mail_delivery(&self, id: Uuid, message_id: Uuid) -> Result<()> {
        query("INSERT INTO mail.deliveries (id, message_id, recipient_kind, status) VALUES ($1, $2, 'human', 'pending')").bind(id).bind(message_id).execute(self.pool()).await.context("create human mail delivery")?;
        Ok(())
    }

    pub async fn set_mail_delivery_status(&self, delivery_id: Uuid, status: &str) -> Result<()> {
        query("UPDATE mail.deliveries SET status = $2, updated_at = now() WHERE id = $1").bind(delivery_id).bind(status).execute(self.pool()).await.context("update mail delivery")?;
        Ok(())
    }

    pub async fn start_mail_wait(&self, id: Uuid, run_id: RunId, reason: &str, detail: serde_json::Value) -> Result<()> {
        query("INSERT INTO mail.waits (id, run_id, reason, detail) VALUES ($1, $2, $3, $4)").bind(id).bind(run_id.as_uuid()).bind(reason).bind(detail).execute(self.pool()).await.context("start mail wait")?;
        Ok(())
    }

    pub async fn end_mail_wait(&self, run_id: RunId) -> Result<()> {
        query("UPDATE mail.waits SET ended_at = now() WHERE run_id = $1 AND ended_at IS NULL").bind(run_id.as_uuid()).execute(self.pool()).await.context("end mail wait")?;
        Ok(())
    }
}
