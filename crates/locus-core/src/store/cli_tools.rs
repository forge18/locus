//! Persistence for the host-approved CLI tool catalog.

use anyhow::{bail, Context, Result};
use uuid::Uuid;

use crate::store::Store;

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct CliToolRow {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub category: String,
    pub enabled: bool,
    pub source: String,
    pub binary_sha256: Option<String>,
    pub install_command: String,
    pub verify_command: String,
    pub documentation_url: Option<String>,
    pub last_rebuilt_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliToolUpload {
    pub name: String,
    pub version: String,
    pub category: String,
    pub binary_sha256: String,
    pub install_command: String,
    pub verify_command: String,
    pub documentation_url: Option<String>,
}

impl Store {
    /// Load the admitted catalog in stable category/name order.
    pub async fn cli_tools(&self) -> Result<Vec<CliToolRow>> {
        sqlx::query_as::<_, CliToolRow>(
            "SELECT id, name, version, category, enabled, source, binary_sha256,
                    install_command, verify_command, documentation_url,
                    last_rebuilt_at::text AS last_rebuilt_at
             FROM core.cli_tools
             ORDER BY category, name",
        )
        .fetch_all(self.pool())
        .await
        .context("load CLI tool catalog")
    }

    /// Change image eligibility without admitting a new executable.
    pub async fn set_cli_tool_enabled(&self, id: Uuid, enabled: bool) -> Result<CliToolRow> {
        let row = sqlx::query_as::<_, CliToolRow>(
            "UPDATE core.cli_tools
             SET enabled = $2, updated_at = now()
             WHERE id = $1
             RETURNING id, name, version, category, enabled, source, binary_sha256,
                       install_command, verify_command, documentation_url,
                       last_rebuilt_at::text AS last_rebuilt_at",
        )
        .bind(id)
        .bind(enabled)
        .fetch_optional(self.pool())
        .await
        .context("update CLI tool enablement")?
        .ok_or_else(|| anyhow::anyhow!("CLI tool `{id}` does not exist"))?;
        Ok(row)
    }

    /// Persist a tool only after the host-side signed admission boundary succeeds.
    pub async fn persist_uploaded_cli_tool(&self, upload: &CliToolUpload) -> Result<CliToolRow> {
        if upload.name.trim().is_empty() || upload.version.trim().is_empty() {
            bail!("CLI tool name and version are required")
        }
        if upload.install_command.trim().is_empty() || upload.verify_command.trim().is_empty() {
            bail!("CLI tool install and verify commands are required")
        }
        if !matches!(
            upload.category.as_str(),
            "source-control" | "search-files" | "rust" | "database" | "web-network"
        ) {
            bail!("unknown CLI tool category `{}`", upload.category)
        }
        let row = sqlx::query_as::<_, CliToolRow>(
            "INSERT INTO core.cli_tools
                (id, name, version, category, source, binary_sha256,
                 install_command, verify_command, documentation_url)
             VALUES ($1, $2, $3, $4, 'uploaded', $5, $6, $7, $8)
             RETURNING id, name, version, category, enabled, source, binary_sha256,
                       install_command, verify_command, documentation_url,
                       last_rebuilt_at::text AS last_rebuilt_at",
        )
        .bind(Uuid::new_v4())
        .bind(upload.name.trim())
        .bind(upload.version.trim())
        .bind(&upload.category)
        .bind(&upload.binary_sha256)
        .bind(upload.install_command.trim())
        .bind(upload.verify_command.trim())
        .bind(upload.documentation_url.as_deref())
        .fetch_one(self.pool())
        .await
        .context("persist uploaded CLI tool")?;
        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::CliToolUpload;
    use crate::store::Store;
    use crate::testkit::postgres::{start_postgres_named, test_backup_config, NoopMigrationBackup};

    #[tokio::test]
    async fn catalog_and_enablement_round_trip() {
        let (container, _cleanup) = start_postgres_named("locus-cli-tool-store-test").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect CLI tool store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("migrate CLI tool store");

        let tools = store.cli_tools().await.expect("list CLI tools");
        assert_eq!(tools.len(), 8);
        let updated = store
            .set_cli_tool_enabled(tools[0].id, false)
            .await
            .expect("disable CLI tool");
        assert!(!updated.enabled);
        let uploaded = store
            .persist_uploaded_cli_tool(&CliToolUpload {
                name: "custom-tool".into(),
                version: "1.0.0".into(),
                category: "rust".into(),
                binary_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
                install_command: "install custom-tool".into(),
                verify_command: "custom-tool --version".into(),
                documentation_url: None,
            })
            .await
            .expect("persist signed CLI tool metadata");
        assert_eq!(uploaded.source, "uploaded");
    }
}
