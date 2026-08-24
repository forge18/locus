//! Durable adapters for immutable wiki pages and revisions.

use anyhow::{bail, Context, Result};
use sqlx::{query, Row};
use uuid::Uuid;
use crate::{ids::{ProjectId, RunId}, services::wiki::{PageKind, WikiPage}, store::Store};

fn kind_name(kind: PageKind) -> &'static str { match kind { PageKind::Decision => "decision", PageKind::Concept => "concept", PageKind::Entity => "entity", PageKind::Source => "source", PageKind::Synthesis => "synthesis" } }

impl Store {
    pub async fn create_wiki_page(&self, page: &WikiPage) -> Result<()> {
        let id: Uuid = page.id.parse().context("wiki page id")?;
        query("INSERT INTO wiki.pages (id, project_id, kind, slug, title) VALUES ($1, $2, $3, $4, $5)")
            .bind(id).bind(page.project_id).bind(kind_name(page.kind)).bind(&page.slug).bind(&page.title)
            .execute(self.pool()).await.context("create wiki page")?;
        Ok(())
    }

    pub async fn append_wiki_revision(&self, page_id: Uuid, revision_id: Uuid, revision: u32, body: &str, summary: &str, author_kind: &str, author_run: Option<RunId>) -> Result<()> {
        if body.trim().is_empty() { bail!("wiki revision body is required"); }
        query("INSERT INTO wiki.revisions (id, page_id, revision_number, body, summary, author_kind, author_run_id) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(revision_id).bind(page_id).bind(revision as i32).bind(body).bind(summary).bind(author_kind).bind(author_run.map(|run| run.as_uuid()))
            .execute(self.pool()).await.context("append wiki revision")?;
        Ok(())
    }

    pub async fn link_wiki_pages(&self, source_page_id: Uuid, source_revision_id: Uuid, target_page_id: Uuid) -> Result<()> {
        query("INSERT INTO wiki.links (source_page_id, source_revision_id, target_page_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(source_page_id).bind(source_revision_id).bind(target_page_id).execute(self.pool()).await.context("link wiki pages")?;
        Ok(())
    }

    pub async fn wiki_project_id(&self, page_id: Uuid) -> Result<ProjectId> {
        let project_id: ProjectId = query("SELECT project_id FROM wiki.pages WHERE id = $1").bind(page_id).fetch_one(self.pool()).await.context("read wiki page project")?.try_get("project_id")?;
        Ok(project_id)
    }
}
