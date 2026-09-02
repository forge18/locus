//! Persistence for Locus-authored extension files and their immutable revisions.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use uuid::Uuid;

use crate::store::Store;

const EXTENSION_TYPES: [&str; 8] = [
    "agents",
    "skills",
    "rules",
    "context",
    "commands",
    "hooks",
    "output-styles",
    "linters",
];

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct ExtensionRow {
    pub id: Uuid,
    pub extension_type: String,
    pub name: String,
    pub version: i32,
    pub frontmatter: Value,
    pub body: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct ExtensionRevisionRow {
    pub id: Uuid,
    pub extension_id: Uuid,
    pub version: i32,
    pub frontmatter: Value,
    pub body: String,
    pub created_at: String,
}

pub fn is_extension_type(extension_type: &str) -> bool {
    EXTENSION_TYPES.contains(&extension_type)
}

impl Store {
    /// Load the current authored files in stable recent-edit order.
    pub async fn extensions(&self, extension_type: &str) -> Result<Vec<ExtensionRow>> {
        validate_extension_type(extension_type)?;
        sqlx::query_as::<_, ExtensionRow>(
            "SELECT id, extension_type, name, version, frontmatter, body,
                    updated_at::text AS updated_at
             FROM core.extensions
             WHERE extension_type = $1
             ORDER BY updated_at DESC, name, id",
        )
        .bind(extension_type)
        .fetch_all(self.pool())
        .await
        .context("load extensions")
    }

    /// Load an extension's immutable revisions newest first.
    pub async fn extension_history(&self, extension_id: Uuid) -> Result<Vec<ExtensionRevisionRow>> {
        sqlx::query_as::<_, ExtensionRevisionRow>(
            "SELECT id, extension_id, version, frontmatter, body,
                    created_at::text AS created_at
             FROM core.extension_revisions
             WHERE extension_id = $1
             ORDER BY version DESC",
        )
        .bind(extension_id)
        .fetch_all(self.pool())
        .await
        .context("load extension history")
    }

    /// Save a current extension and append its revision atomically.
    pub async fn persist_extension(
        &self,
        id: Option<Uuid>,
        extension_type: &str,
        name: &str,
        frontmatter: &Value,
        body: &str,
    ) -> Result<ExtensionRow> {
        validate_extension_type(extension_type)?;
        if name.trim().is_empty() {
            bail!("extension name is required")
        }
        if !frontmatter.is_object() {
            bail!("extension frontmatter must be a JSON object")
        }

        let mut transaction = self.pool().begin().await.context("begin extension save")?;
        let row = if let Some(id) = id {
            sqlx::query_as::<_, ExtensionRow>(
                "UPDATE core.extensions
                 SET extension_type = $2, name = $3, version = version + 1,
                     frontmatter = $4, body = $5, updated_at = now()
                 WHERE id = $1
                 RETURNING id, extension_type, name, version, frontmatter, body,
                           updated_at::text AS updated_at",
            )
            .bind(id)
            .bind(extension_type)
            .bind(name.trim())
            .bind(frontmatter)
            .bind(body)
            .fetch_optional(&mut *transaction)
            .await
            .context("update extension")?
            .ok_or_else(|| anyhow::anyhow!("extension `{id}` does not exist"))?
        } else {
            sqlx::query_as::<_, ExtensionRow>(
                "INSERT INTO core.extensions
                    (id, extension_type, name, frontmatter, body)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (extension_type, name) DO UPDATE SET
                    version = core.extensions.version + 1,
                    frontmatter = EXCLUDED.frontmatter,
                    body = EXCLUDED.body,
                    updated_at = now()
                 RETURNING id, extension_type, name, version, frontmatter, body,
                           updated_at::text AS updated_at",
            )
            .bind(Uuid::new_v4())
            .bind(extension_type)
            .bind(name.trim())
            .bind(frontmatter)
            .bind(body)
            .fetch_one(&mut *transaction)
            .await
            .context("insert extension")?
        };

        sqlx::query(
            "INSERT INTO core.extension_revisions
                (id, extension_id, version, frontmatter, body)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(row.id)
        .bind(row.version)
        .bind(&row.frontmatter)
        .bind(&row.body)
        .execute(&mut *transaction)
        .await
        .context("append extension revision")?;
        transaction
            .commit()
            .await
            .context("commit extension save")?;
        Ok(row)
    }
}

fn validate_extension_type(extension_type: &str) -> Result<()> {
    if is_extension_type(extension_type) {
        Ok(())
    } else {
        bail!("unknown extension type `{extension_type}`")
    }
}

#[cfg(test)]
mod tests {
    use super::is_extension_type;
    use crate::store::Store;
    use crate::testkit::postgres::{start_postgres_named, test_backup_config, NoopMigrationBackup};
    use serde_json::json;

    #[test]
    fn extension_types_are_closed() {
        assert!(is_extension_type("skills"));
        assert!(is_extension_type("output-styles"));
        assert!(!is_extension_type("harnesses"));
        assert!(!is_extension_type("unknown"));
    }

    #[tokio::test]
    async fn persistence_round_trips_current_row_and_history() {
        let (container, _cleanup) = start_postgres_named("locus-extension-store-test").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect extension store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("migrate extension store");

        let first = store
            .persist_extension(
                None,
                "skills",
                "verify-loop",
                &json!({"budget_tokens": "12000"}),
                "Run verification.",
            )
            .await
            .expect("save first extension revision");
        let second = store
            .persist_extension(
                Some(first.id),
                "skills",
                "verify-loop",
                &json!({"budget_tokens": "9000"}),
                "Run updated verification.",
            )
            .await
            .expect("save second extension revision");

        assert_eq!(second.version, 2);
        assert_eq!(
            store
                .extensions("skills")
                .await
                .expect("list extensions")
                .len(),
            1
        );
        let history = store
            .extension_history(first.id)
            .await
            .expect("load extension history");
        assert_eq!(
            history.iter().map(|row| row.version).collect::<Vec<_>>(),
            [2, 1]
        );
    }
}
