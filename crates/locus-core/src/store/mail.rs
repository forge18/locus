//! Durable adapters for mail threads, deliveries, and waits.

use crate::{ids::ProjectId, ids::RunId, store::Store};
use anyhow::{bail, Context, Result};
use sqlx::{query, query_as, query_scalar};
use uuid::Uuid;

/// One pending human delivery with its message, thread subject, and project —
/// the Inbox list's wire shape.
#[derive(Debug, sqlx::FromRow)]
pub struct InboxDeliveryRow {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub subject: String,
    pub body: String,
    pub sender_kind: String,
    pub project: String,
    pub created_at: Option<String>,
}

/// Deliveries resolved today (drained), newest first — the RESOLVED TODAY list.
#[derive(Debug, sqlx::FromRow)]
pub struct ResolvedDeliveryRow {
    pub id: Uuid,
    pub subject: String,
    pub body: String,
    pub project: String,
    pub resolved_at: Option<String>,
}

impl Store {
    /// Every human-addressed delivery still pending, newest first. A project id
    /// scopes the read; `None` is the cross-project Inbox.
    pub async fn pending_human_inbox(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<InboxDeliveryRow>> {
        query_as(
            "SELECT d.id, m.thread_id, t.subject, m.body, m.sender_kind,
                    p.name AS project, d.created_at::text AS created_at
             FROM mail.deliveries d
             JOIN mail.messages m ON m.id = d.message_id
             JOIN mail.threads t ON t.id = m.thread_id
             JOIN core.projects p ON p.id = t.project_id
             WHERE d.recipient_kind = 'human' AND d.status = 'pending'
                AND ($1::uuid IS NULL OR t.project_id = $1)
             ORDER BY d.created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("list pending human deliveries")
    }

    pub async fn resolved_today(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<ResolvedDeliveryRow>> {
        query_as(
            "SELECT d.id, t.subject, m.body, p.name AS project,
                    d.updated_at::text AS resolved_at
             FROM mail.deliveries d
             JOIN mail.messages m ON m.id = d.message_id
             JOIN mail.threads t ON t.id = m.thread_id
             JOIN core.projects p ON p.id = t.project_id
             WHERE d.recipient_kind = 'human' AND d.status = 'drained'
                AND d.updated_at >= date_trunc('day', now())
                AND ($1::uuid IS NULL OR t.project_id = $1)
             ORDER BY d.updated_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("list resolved deliveries")
    }

    /// Deliveries drained today — the throughput half of the Inbox pill.
    pub async fn resolved_today_count(&self) -> Result<i64> {
        query_scalar(
            "SELECT COUNT(*)
             FROM mail.deliveries
             WHERE recipient_kind = 'human' AND status = 'drained'
                AND updated_at >= date_trunc('day', now())",
        )
        .fetch_one(&self.pool)
        .await
        .context("count resolved deliveries")
    }

    /// The thread a delivery belongs to, for appending the decision reply.
    pub async fn mail_thread_of_delivery(&self, delivery_id: Uuid) -> Result<Option<Uuid>> {
        query_scalar("SELECT thread_id FROM mail.deliveries d JOIN mail.messages m ON m.id = d.message_id WHERE d.id = $1")
            .bind(delivery_id)
            .fetch_optional(&self.pool)
            .await
            .context("read delivery thread")
    }

    /// Human-addressed deliveries still pending — the Inbox pill's count. Agent
    /// deliveries never appear here: the Inbox is what waits on a person.
    pub async fn pending_human_delivery_count(&self) -> Result<i64> {
        query_scalar(
            "SELECT COUNT(*)
             FROM mail.deliveries
             WHERE recipient_kind = 'human' AND status = 'pending'",
        )
        .fetch_one(&self.pool)
        .await
        .context("count pending human deliveries")
    }


    pub async fn create_mail_thread(
        &self,
        id: Uuid,
        project_id: Uuid,
        subject: &str,
    ) -> Result<()> {
        if subject.trim().is_empty() {
            bail!("mail subject is required");
        }
        query("INSERT INTO mail.threads (id, project_id, subject) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(project_id)
            .bind(subject)
            .execute(self.pool())
            .await
            .context("create mail thread")?;
        Ok(())
    }

    pub async fn append_mail_message(
        &self,
        id: Uuid,
        thread_id: Uuid,
        sender_kind: &str,
        sender_run: Option<RunId>,
        body: &str,
    ) -> Result<()> {
        if body.trim().is_empty() {
            bail!("mail body is required");
        }
        query("INSERT INTO mail.messages (id, thread_id, sender_kind, sender_run_id, body) VALUES ($1, $2, $3, $4, $5)").bind(id).bind(thread_id).bind(sender_kind).bind(sender_run.map(|run| run.as_uuid())).bind(body).execute(self.pool()).await.context("append mail message")?;
        Ok(())
    }

    pub async fn create_human_mail_delivery(&self, id: Uuid, message_id: Uuid) -> Result<()> {
        query("INSERT INTO mail.deliveries (id, message_id, recipient_kind, status) VALUES ($1, $2, 'human', 'pending')").bind(id).bind(message_id).execute(self.pool()).await.context("create human mail delivery")?;
        Ok(())
    }

    pub async fn set_mail_delivery_status(&self, delivery_id: Uuid, status: &str) -> Result<()> {
        query("UPDATE mail.deliveries SET status = $2, updated_at = now() WHERE id = $1")
            .bind(delivery_id)
            .bind(status)
            .execute(self.pool())
            .await
            .context("update mail delivery")?;
        Ok(())
    }

    pub async fn start_mail_wait(
        &self,
        id: Uuid,
        run_id: RunId,
        reason: &str,
        detail: serde_json::Value,
    ) -> Result<()> {
        query("INSERT INTO mail.waits (id, run_id, reason, detail) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(run_id.as_uuid())
            .bind(reason)
            .bind(detail)
            .execute(self.pool())
            .await
            .context("start mail wait")?;
        Ok(())
    }

    pub async fn end_mail_wait(&self, run_id: RunId) -> Result<()> {
        query("UPDATE mail.waits SET ended_at = now() WHERE run_id = $1 AND ended_at IS NULL")
            .bind(run_id.as_uuid())
            .execute(self.pool())
            .await
            .context("end mail wait")?;
        Ok(())
    }
}
