//! Lightweight persisted search projections used by the desktop command palette.

use anyhow::{Context, Result};
use sqlx::{query_as, FromRow};

use crate::store::Store;

#[derive(Debug, FromRow)]
pub struct PaletteSearchRow {
    pub kind: String,
    pub project: String,
    pub label: String,
    pub locator: String,
    pub score: i32,
}

impl Store {
    pub async fn palette_search(&self, query: &str) -> Result<Vec<PaletteSearchRow>> {
        let pattern = format!("%{}%", query.trim());
        let mut rows = query_as::<_, PaletteSearchRow>(
            "SELECT 'task' AS kind, p.name AS project,
                    t.summary AS label,
                    'locus://' || t.project_id::text || '/task/' || t.id::text AS locator,
                    2 AS score
             FROM board.tasks t
             JOIN core.projects p ON p.id = t.project_id
             WHERE t.summary ILIKE $1 OR t.description ILIKE $1
             ORDER BY score DESC, p.name, t.summary, t.id",
        )
        .bind(&pattern)
        .fetch_all(self.pool())
        .await
        .context("search board tasks")?;

        rows.extend(
            query_as::<_, PaletteSearchRow>(
                "SELECT 'wiki' AS kind, p.name AS project,
                        p.title AS label,
                        'locus://' || p.project_id::text || '/page/' || p.slug AS locator,
                        3 AS score
                 FROM wiki.pages p
                 WHERE p.title ILIKE $1 OR p.slug ILIKE $1
                 ORDER BY score DESC, p.name, p.title, p.id",
            )
            .bind(&pattern)
            .fetch_all(self.pool())
            .await
            .context("search wiki pages")?,
        );
        rows.extend(
            query_as::<_, PaletteSearchRow>(
                "SELECT 'run' AS kind, p.name AS project,
                        ad.name || ' · ' || s.branch AS label,
                        'locus://' || s.project_id::text || '/session/' || s.id::text || '/run/' || r.id::text AS locator,
                        2 AS score
                 FROM agents.runs r
                 JOIN agents.sessions s ON s.id = r.session_id
                 JOIN core.projects p ON p.id = s.project_id
                 JOIN agents.agent_defs ad ON ad.id = s.agent_def_id
                 WHERE ad.name ILIKE $1 OR s.branch ILIKE $1 OR r.id::text ILIKE $1
                 ORDER BY score DESC, p.name, label, r.id",
            )
            .bind(&pattern)
            .fetch_all(self.pool())
            .await
            .context("search runs")?,
        );
        rows.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.project.cmp(&right.project))
                .then_with(|| left.label.cmp(&right.label))
        });
        Ok(rows)
    }
}
