//! Persistence for run artifacts and their comments (`agents.artifacts`).
//!
//! Moved out of `services/artifact.rs` so every query in the crate lives under `store/`.

use crate::ids::{ArtifactId, ProjectId, RunId, SessionId};
use std::path::PathBuf;

use anyhow::{bail, Result};
use sqlx::query;
use sqlx::{query_as, FromRow};
use uuid::Uuid;

use crate::{
    services::artifact::{
        ArtifactComment, ArtifactContent, ArtifactKind, ArtifactRow, ResearchProvenance,
        SessionResearchFeed,
    },
    store::Store,
};

impl Store {
    pub async fn session_research_feed(
        &self,
        session_id: SessionId,
    ) -> Result<SessionResearchFeed> {
        let mut feed = SessionResearchFeed::new(session_id);
        for (artifact, provenance) in self.finding_artifacts(session_id).await? {
            feed.add_finding(artifact, provenance)?;
        }
        Ok(feed)
    }

    pub async fn session_research_feed_with_seed(
        &self,
        session_id: SessionId,
        planning_session_id: Option<SessionId>,
    ) -> Result<SessionResearchFeed> {
        let mut feed = self.session_research_feed(session_id).await?;
        if let Some(planning_session_id) = planning_session_id {
            let findings = self
                .finding_artifacts(planning_session_id)
                .await?
                .into_iter()
                .map(|(artifact, _)| artifact);
            feed.seed_from_plan(findings)?;
        }
        Ok(feed)
    }

    async fn finding_artifacts(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<(ArtifactRow, ResearchProvenance)>> {
        let rows = query_as::<_, PersistedArtifactRow>(
            "SELECT a.id, s.project_id, a.run_id, a.kind, a.body, a.blob_path,
                    a.media_type, a.sha256, a.derived_representation, a.summary
             FROM agents.artifacts a
             JOIN agents.runs r ON r.id = a.run_id
             JOIN agents.sessions s ON s.id = r.session_id
             WHERE r.session_id = $1 AND a.kind = 'finding'
             ORDER BY a.created_at, a.id",
        )
        .bind(session_id)
        .fetch_all(self.pool())
        .await?;
        rows.into_iter()
            .map(|row| {
                let provenance = row
                    .derived_representation
                    .as_ref()
                    .and_then(|metadata| metadata.get("research_provenance"))
                    .and_then(serde_json::Value::as_str)
                    .and_then(ResearchProvenance::from_label)
                    .unwrap_or(ResearchProvenance::ThisRun);
                Ok((row.try_into()?, provenance))
            })
            .collect()
    }

    /// Persist the metadata and either text or blob reference for a reviewable artifact.
    pub async fn save_artifact(&self, artifact: &ArtifactRow) -> Result<()> {
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
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Load one artifact, deriving its project ownership through its persisted run and session.
    pub async fn artifact(&self, id: Uuid) -> Result<Option<ArtifactRow>> {
        query_as::<_, PersistedArtifactRow>(
            "SELECT a.id, s.project_id, a.run_id, a.kind, a.body, a.blob_path,
                    a.media_type, a.sha256, a.derived_representation, a.summary
             FROM agents.artifacts a
             JOIN agents.runs r ON r.id = a.run_id
             JOIN agents.sessions s ON s.id = r.session_id
             WHERE a.id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .map(TryInto::try_into)
        .transpose()
    }

    /// Persist human feedback; delivery to a live or future run remains a supervisor concern.
    pub async fn save_artifact_comment(
        &self,
        artifact_id: ArtifactId,
        parent_id: Option<Uuid>,
        body: impl AsRef<str>,
    ) -> Result<ArtifactComment> {
        let body = body.as_ref();
        if body.trim().is_empty() {
            bail!("artifact comment body must not be empty")
        }
        query_as::<_, PersistedArtifactComment>(
            "INSERT INTO agents.artifact_comments
             (id, artifact_id, parent_comment_id, author_kind, body)
             VALUES ($1, $2, $3, 'human', $4)
             RETURNING id, artifact_id, parent_comment_id, body",
        )
        .bind(Uuid::new_v4())
        .bind(artifact_id)
        .bind(parent_id)
        .bind(body)
        .fetch_one(self.pool())
        .await
        .map(Into::into)
        .map_err(Into::into)
    }

    pub async fn artifact_comments(&self, artifact_id: ArtifactId) -> Result<Vec<ArtifactComment>> {
        query_as::<_, PersistedArtifactComment>(
            "SELECT id, artifact_id, parent_comment_id, body
             FROM agents.artifact_comments
             WHERE artifact_id = $1
             ORDER BY created_at, id",
        )
        .bind(artifact_id)
        .fetch_all(self.pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(Into::into)
    }
}

#[derive(FromRow)]
struct PersistedArtifactRow {
    id: Uuid,
    project_id: ProjectId,
    run_id: RunId,
    kind: String,
    body: Option<String>,
    blob_path: Option<String>,
    media_type: Option<String>,
    sha256: Option<String>,
    derived_representation: Option<serde_json::Value>,
    summary: Option<String>,
}

#[derive(FromRow)]
struct PersistedArtifactComment {
    id: Uuid,
    artifact_id: ArtifactId,
    parent_comment_id: Option<Uuid>,
    body: String,
}

impl TryFrom<PersistedArtifactRow> for ArtifactRow {
    type Error = anyhow::Error;

    fn try_from(row: PersistedArtifactRow) -> Result<Self> {
        let content = match (row.body, row.blob_path, row.media_type, row.sha256) {
            (Some(body), None, None, None) => ArtifactContent::Text(body),
            (None, Some(path), Some(media_type), Some(sha256)) => ArtifactContent::Blob {
                path: PathBuf::from(path),
                media_type,
                sha256,
            },
            _ => bail!("artifact row {} has invalid content columns", row.id),
        };
        Ok(Self {
            id: row.id.into(),
            project_id: row.project_id,
            run_id: row.run_id,
            kind: ArtifactKind::from_database_name(&row.kind)?,
            content,
            derived_cache: row.derived_representation,
            summary: row.summary,
        })
    }
}

impl From<PersistedArtifactComment> for ArtifactComment {
    fn from(row: PersistedArtifactComment) -> Self {
        Self {
            id: row.id.into(),
            artifact_id: row.artifact_id,
            parent_id: row.parent_comment_id.map(Into::into),
            body: row.body,
        }
    }
}
