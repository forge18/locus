//! Durable resolver snapshots for the local marketplace index.

use anyhow::{Context, Result};
use serde_json::Value;
use sqlx::query_as;
use uuid::Uuid;

use crate::services::market::{Manifest, ManifestIndex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestSnapshot {
    pub id: Uuid,
    pub name: String,
    pub manifest: Manifest,
    pub content_sha256: String,
}

#[derive(sqlx::FromRow)]
struct ManifestSnapshotRow {
    id: Uuid,
    name: String,
    manifest: Value,
    content_sha256: String,
}

impl TryFrom<ManifestSnapshotRow> for ManifestSnapshot {
    type Error = anyhow::Error;

    fn try_from(row: ManifestSnapshotRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            name: row.name,
            manifest: serde_json::from_value(row.manifest)
                .context("decode persisted marketplace manifest")?,
            content_sha256: row.content_sha256,
        })
    }
}

impl crate::store::Store {
    /// Persist one validated resolver snapshot idempotently. This does not install or trust a
    /// binary; signed admission remains the separate `ToolCatalog` boundary used by M8.
    pub async fn persist_market_manifest(&self, manifest: &Manifest) -> Result<ManifestSnapshot> {
        manifest
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let content_sha256 = manifest.content_sha256()?;
        let json = serde_json::to_value(manifest).context("encode marketplace manifest")?;
        let row = query_as::<_, ManifestSnapshotRow>(
            "INSERT INTO market.manifest_snapshots (id, name, manifest, content_sha256)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (name, content_sha256) DO UPDATE SET name = EXCLUDED.name
             RETURNING id, name, manifest, content_sha256",
        )
        .bind(Uuid::new_v4())
        .bind(&manifest.name)
        .bind(json)
        .bind(&content_sha256)
        .fetch_one(self.pool())
        .await?;
        row.try_into()
    }

    /// Snapshot every manifest in deterministic name order and return the resulting pins.
    pub async fn persist_market_index(
        &self,
        index: &ManifestIndex,
    ) -> Result<Vec<ManifestSnapshot>> {
        let mut snapshots = Vec::new();
        for manifest in index.manifests() {
            snapshots.push(self.persist_market_manifest(manifest).await?);
        }
        Ok(snapshots)
    }

    pub async fn market_manifest(&self, name: &str) -> Result<Option<ManifestSnapshot>> {
        let row = query_as::<_, ManifestSnapshotRow>(
            "SELECT id, name, manifest, content_sha256
             FROM market.manifest_snapshots
             WHERE name = $1
             ORDER BY content_sha256 ASC
             LIMIT 1",
        )
        .bind(name)
        .fetch_optional(self.pool())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    /// Resolve names from snapshots already materialized in the market schema. Unknown names
    /// fail at the same boundary as local resolution, so an agent can never reach an unindexed
    /// tool merely by knowing its executable name.
    pub async fn resolve_market_tools(&self, names: &[String]) -> Result<Vec<ManifestSnapshot>> {
        let mut requested = names.to_vec();
        requested.sort();
        requested.dedup();
        let rows = query_as::<_, ManifestSnapshotRow>(
            "SELECT DISTINCT ON (name) id, name, manifest, content_sha256
             FROM market.manifest_snapshots
             WHERE name = ANY($1::text[])
             ORDER BY name, content_sha256 ASC",
        )
        .bind(&requested)
        .fetch_all(self.pool())
        .await?;
        let mut snapshots = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<ManifestSnapshot>>>()?;
        let found: std::collections::BTreeSet<_> =
            snapshots.iter().map(|row| row.name.as_str()).collect();
        if let Some(missing) = requested.iter().find(|name| !found.contains(name.as_str())) {
            anyhow::bail!("tool `{missing}` is absent from the marketplace index")
        }
        snapshots.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(snapshots)
    }
}
