//! Provider credential references and the host-only OS-keychain boundary.
//!
//! A provider row can identify a keychain entry, but never contains the credential stored there.
//! The broker borrows a credential only while executing a host egress callback; it deliberately
//! does not expose a credential-returning API to persistence, events, or container launch code.

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::store::Store;

/// The stable locator for a credential held by the operating system's keychain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct KeychainReference(String);

impl KeychainReference {
    pub fn new(reference: impl Into<String>) -> Result<Self> {
        let reference = reference.into();
        if reference.trim().is_empty() {
            bail!("keychain reference must not be empty")
        }
        Ok(Self(reference))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The durable provider identity and OS-keychain locator.
///
/// Catalog and verification records are keyed by this reference, so no provider extension needs a
/// secret-bearing field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderReference {
    pub id: Uuid,
    pub identifier: String,
    pub keychain_reference: KeychainReference,
}

impl ProviderReference {
    pub fn new(
        id: Uuid,
        identifier: impl Into<String>,
        keychain_reference: KeychainReference,
    ) -> Result<Self> {
        let identifier = identifier.into();
        if identifier.trim().is_empty() {
            bail!("provider identifier must not be empty")
        }
        Ok(Self {
            id,
            identifier,
            keychain_reference,
        })
    }
}

/// Secret-free connection configuration stored beside a provider reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderConnectionConfig {
    authentication_method: String,
    base_url: Option<String>,
}

impl ProviderConnectionConfig {
    pub fn new(authentication_method: impl Into<String>, base_url: Option<String>) -> Result<Self> {
        let authentication_method = authentication_method.into();
        if authentication_method.trim().is_empty() || base_url.as_deref().is_some_and(|url| url.trim().is_empty()) {
            bail!("provider connection configuration must not contain empty values")
        }
        Ok(Self { authentication_method, base_url })
    }

    pub fn authentication_method(&self) -> &str { &self.authentication_method }
    pub fn base_url(&self) -> Option<&str> { self.base_url.as_deref() }
}

/// A curated model entry owned by one provider.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderModel {
    pub provider_id: Uuid,
    pub model_id: String,
    pub alias: Option<String>,
    pub selector_included: bool,
}

impl ProviderModel {
    pub fn new(provider_id: Uuid, model_id: impl Into<String>) -> Result<Self> {
        let model_id = model_id.into();
        if model_id.trim().is_empty() {
            bail!("provider model id must not be empty")
        }
        Ok(Self {
            provider_id,
            model_id,
            alias: None,
            selector_included: true,
        })
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Result<Self> {
        let alias = alias.into();
        if alias.trim().is_empty() {
            bail!("provider model alias must not be empty")
        }
        self.alias = Some(alias);
        Ok(self)
    }

    pub fn exclude_from_selector(mut self) -> Self {
        self.selector_included = false;
        self
    }
}

/// A model as presented to a selector. `label` is an alias whenever the provider supplied one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelSelectorOption {
    pub provider_id: Uuid,
    pub model_id: String,
    pub label: String,
}

/// Project the provider-curated catalog into selector-safe display entries.
pub fn selector_projection(models: &[ProviderModel]) -> Vec<ModelSelectorOption> {
    models
        .iter()
        .filter(|model| model.selector_included)
        .map(|model| ModelSelectorOption {
            provider_id: model.provider_id,
            model_id: model.model_id.clone(),
            label: model
                .alias
                .clone()
                .unwrap_or_else(|| model.model_id.clone()),
        })
        .collect()
}

/// The outcome metadata from the most recent host-side provider verification.
///
/// It deliberately records only a status, never a provider error, because error text can contain
/// credentials. Timestamps are RFC 3339 so the durable boundary is portable across keychain hosts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderVerificationMetadata {
    pub verified_at: String,
    pub model_count: u32,
    pub status: VerificationStatus,
}

impl ProviderVerificationMetadata {
    pub fn new(
        verified_at: impl Into<String>,
        model_count: u32,
        status: VerificationStatus,
    ) -> Result<Self> {
        let verified_at = verified_at.into();
        OffsetDateTime::parse(&verified_at, &Rfc3339)
            .map_err(|_| anyhow!("provider verification timestamp must be RFC 3339"))?;
        Ok(Self {
            verified_at,
            model_count,
            status,
        })
    }
}

/// Whether the last host-side provider verification succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Failed,
}

impl VerificationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }
}

impl Store {
    /// Persist only the provider identity and OS-keychain locator.
    pub async fn persist_provider_reference(&self, provider: &ProviderReference) -> Result<()> {
        sqlx::query(
            "INSERT INTO core.providers (id, identifier, keychain_reference)
             VALUES ($1, $2, $3)
             ON CONFLICT (id) DO UPDATE SET
                 identifier = EXCLUDED.identifier,
                 keychain_reference = EXCLUDED.keychain_reference,
                 updated_at = now()",
        )
        .bind(provider.id)
        .bind(&provider.identifier)
        .bind(provider.keychain_reference.as_str())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Replace the curated model catalog for one provider atomically.
    pub async fn persist_provider_models(
        &self,
        provider_id: Uuid,
        models: &[ProviderModel],
    ) -> Result<()> {
        if models.iter().any(|model| model.provider_id != provider_id) {
            bail!("provider model belongs to a different provider")
        }
        let mut model_ids = std::collections::HashSet::new();
        if models
            .iter()
            .any(|model| !model_ids.insert(&model.model_id))
        {
            bail!("provider model catalog contains duplicate model ids")
        }

        let mut transaction = self.pool().begin().await?;
        sqlx::query("DELETE FROM core.provider_models WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&mut *transaction)
            .await?;
        for model in models {
            sqlx::query(
                "INSERT INTO core.provider_models (provider_id, model_id, alias, selector_included)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(model.provider_id)
            .bind(&model.model_id)
            .bind(&model.alias)
            .bind(model.selector_included)
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    /// Persist the secret-free result of the latest host-side verification.
    pub async fn persist_provider_verification_metadata(
        &self,
        provider_id: Uuid,
        metadata: &ProviderVerificationMetadata,
    ) -> Result<()> {
        let updated = sqlx::query(
            "UPDATE core.providers
             SET verification_at = $2::text::timestamptz,
                 verification_model_count = $3,
                 verification_status = $4,
                 updated_at = now()
             WHERE id = $1",
        )
        .bind(provider_id)
        .bind(&metadata.verified_at)
        .bind(i32::try_from(metadata.model_count).map_err(|_| anyhow!("model count is too large"))?)
        .bind(metadata.status.as_str())
        .execute(self.pool())
        .await?;
        if updated.rows_affected() != 1 {
            bail!("provider `{provider_id}` does not exist")
        }
        Ok(())
    }

    /// Load one provider's visible catalog in deterministic model-id order for a selector.
    pub async fn provider_selector_projection(
        &self,
        provider_id: Uuid,
    ) -> Result<Vec<ModelSelectorOption>> {
        use sqlx::Row;

        let rows = sqlx::query(
            "SELECT provider_id, model_id, alias, selector_included
             FROM core.provider_models
             WHERE provider_id = $1
             ORDER BY model_id",
        )
        .bind(provider_id)
        .fetch_all(self.pool())
        .await?;
        let models = rows
            .into_iter()
            .map(|row| {
                Ok(ProviderModel {
                    provider_id: row.try_get("provider_id")?,
                    model_id: row.try_get("model_id")?,
                    alias: row.try_get("alias")?,
                    selector_included: row.try_get("selector_included")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(selector_projection(&models))
    }
}

/// Platform adapters implement this trait with macOS Keychain, Windows Credential Manager, or a
/// Linux keyring. The durable reference contract above is independent of that implementation.
pub trait OsKeychain: Send + Sync {
    fn read_secret(&self, reference: &KeychainReference) -> Result<String>;
    fn write_secret(&self, reference: &KeychainReference, secret: &str) -> Result<()>;
    fn delete_secret(&self, reference: &KeychainReference) -> Result<()>;
}

/// The host-only operating-system keychain adapter. References are account names; values never
/// leave this adapter except through `ProviderBroker::with_secret` at host egress.
pub struct KeyringKeychain;

impl KeyringKeychain {
    const SERVICE: &'static str = "locus";

    pub fn entry_name(reference: &KeychainReference) -> &str {
        reference.as_str()
    }

    fn entry(reference: &KeychainReference) -> Result<keyring::Entry> {
        keyring::Entry::new(Self::SERVICE, Self::entry_name(reference))
            .map_err(|_| anyhow!("keychain entry unavailable"))
    }
}

impl OsKeychain for KeyringKeychain {
    fn read_secret(&self, reference: &KeychainReference) -> Result<String> {
        Self::entry(reference)?
            .get_password()
            .map_err(|_| anyhow!("keychain credential resolution failed"))
    }

    fn write_secret(&self, reference: &KeychainReference, secret: &str) -> Result<()> {
        Self::entry(reference)?
            .set_password(secret)
            .map_err(|_| anyhow!("keychain credential write failed"))
    }

    fn delete_secret(&self, reference: &KeychainReference) -> Result<()> {
        Self::entry(reference)?
            .delete_credential()
            .map_err(|_| anyhow!("keychain credential deletion failed"))
    }
}

/// The host-only boundary for provider authentication at outbound egress.
pub struct ProviderBroker<K> {
    keychain: K,
}

impl<K> ProviderBroker<K>
where
    K: OsKeychain,
{
    pub fn new(keychain: K) -> Self {
        Self { keychain }
    }

    /// Resolve a secret only while issuing a host egress request.
    ///
    /// Keychain and egress errors intentionally omit their underlying detail: either source may
    /// include the credential, and errors are observable outside this boundary.
    pub fn verify_connection(
        &self,
        provider: &ProviderReference,
        verified_at: impl Into<String>,
        probe: impl FnOnce(&str) -> Result<u32>,
    ) -> Result<ProviderVerificationMetadata> {
        let verified_at = verified_at.into();
        let secret = self
            .keychain
            .read_secret(&provider.keychain_reference)
            .map_err(|_| anyhow!("provider credential resolution failed"))?;
        if secret.is_empty() {
            bail!("provider credential resolution failed")
        }
        let model_count = probe(&secret).map_err(|error| anyhow!(redact(&error.to_string(), &secret)))?;
        ProviderVerificationMetadata::new(verified_at, model_count, VerificationStatus::Verified)
    }

    pub fn with_host_egress(
        &self,
        provider: &ProviderReference,
        egress: impl FnOnce(&str) -> Result<()>,
    ) -> Result<()> {
        let secret = self
            .keychain
            .read_secret(&provider.keychain_reference)
            .map_err(|_| anyhow!("provider credential resolution failed"))?;
        if secret.is_empty() {
            bail!("provider credential resolution failed")
        }

        egress(&secret).map_err(|error| anyhow!(redact(&error.to_string(), &secret)))
    }
}

fn redact(message: &str, secret: &str) -> String {
    message.replace(secret, "[REDACTED]")
}

#[cfg(test)]
#[test]
fn selector_aliases() {
    let provider_id = Uuid::nil();
    let aliased = ProviderModel::new(provider_id, "claude-opus-4-6")
        .unwrap()
        .with_alias("Opus")
        .unwrap();
    let raw = ProviderModel::new(provider_id, "claude-sonnet-4-6").unwrap();
    let hidden = ProviderModel::new(provider_id, "retired-model")
        .unwrap()
        .exclude_from_selector();

    assert_eq!(
        selector_projection(&[aliased, raw, hidden]),
        vec![
            ModelSelectorOption {
                provider_id,
                model_id: "claude-opus-4-6".into(),
                label: "Opus".into(),
            },
            ModelSelectorOption {
                provider_id,
                model_id: "claude-sonnet-4-6".into(),
                label: "claude-sonnet-4-6".into(),
            },
        ],
        "selectors use provider-owned aliases and omit curated exclusions"
    );
}

#[cfg(test)]
const SECRET: &str = "test-provider-secret";

#[cfg(test)]
struct TestKeychain;

#[cfg(test)]
impl OsKeychain for TestKeychain {
    fn read_secret(&self, _: &KeychainReference) -> Result<String> {
        Ok(SECRET.into())
    }

    fn write_secret(&self, _: &KeychainReference, _: &str) -> Result<()> {
        Ok(())
    }
    fn delete_secret(&self, _: &KeychainReference) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
struct LeakyKeychain;

#[cfg(test)]
impl OsKeychain for LeakyKeychain {
    fn read_secret(&self, _: &KeychainReference) -> Result<String> {
        anyhow::bail!("keychain failed for {SECRET}")
    }

    fn write_secret(&self, _: &KeychainReference, _: &str) -> Result<()> {
        Ok(())
    }
    fn delete_secret(&self, _: &KeychainReference) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
struct DockerCleanup {
    container_name: String,
    volume_name: String,
}

#[cfg(test)]
impl Drop for DockerCleanup {
    fn drop(&mut self) {
        use std::process::{Command, Stdio};

        let _ = Command::new("docker")
            .args(["rm", "--force", &self.container_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = Command::new("docker")
            .args(["volume", "rm", "--force", &self.volume_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[cfg(test)]
fn unused_port() -> u16 {
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
    listener.local_addr().expect("read the local port").port()
}

#[cfg(test)]
struct NoopMigrationBackup;

#[cfg(test)]
impl crate::backup::MigrationBackup for NoopMigrationBackup {
    fn create_retained(&self, _: &crate::backup::RetainedBackupConfig) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
fn test_backup_config() -> crate::backup::RetainedBackupConfig {
    crate::backup::RetainedBackupConfig::new(
        "postgres://locus@localhost/locus",
        "/var/lib/locus/artifacts",
        "/var/lib/locus/backups",
    )
}

#[cfg(test)]
#[test]
fn reference_schema() {
    let migration = include_str!("../../../migrations/0012_provider_references.up.sql");
    assert!(migration.contains("CREATE TABLE core.providers"));
    assert!(migration.contains("keychain_reference"));
    assert!(!migration.contains("secret"));
}

#[cfg(test)]
#[tokio::test]
async fn never_persists_secret() {
    use sqlx::query_scalar;

    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-provider-test-{suffix}");
    let volume_name = format!("locus-provider-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container = crate::store::PostgresContainer::new(crate::store::PostgresConfig::for_test(
        container_name,
        volume_name,
        port,
    ));
    container
        .start()
        .await
        .expect("start provider test database");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect provider test database");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &test_backup_config(),
        )
        .await
        .expect("migrate provider test database");

    let reference = KeychainReference::new("os-keychain://locus/anthropic").unwrap();
    let provider = ProviderReference::new(Uuid::nil(), "anthropic", reference).unwrap();
    store
        .persist_provider_reference(&provider)
        .await
        .expect("persist provider reference");
    let row: String =
        query_scalar("SELECT row_to_json(providers)::text FROM core.providers providers")
            .fetch_one(store.pool())
            .await
            .expect("read persisted provider row");
    let event = serde_json::json!({"provider": serde_json::to_value(&provider).unwrap()});

    assert!(!row.contains(SECRET));
    assert!(!event.to_string().contains(SECRET));
    assert!(!format!("{provider:?}").contains(SECRET));

    let broker = ProviderBroker::new(TestKeychain);
    let egress_error = broker
        .with_host_egress(&provider, |secret| {
            anyhow::bail!("upstream rejected {secret}")
        })
        .unwrap_err();
    assert!(!egress_error.to_string().contains(SECRET));

    let keychain_error = ProviderBroker::new(LeakyKeychain)
        .with_host_egress(&provider, |_| Ok(()))
        .unwrap_err();
    assert!(!keychain_error.to_string().contains(SECRET));
}

#[cfg(test)]
#[tokio::test]
async fn catalog_and_verification_metadata_persist() {
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-provider-catalog-test-{suffix}");
    let volume_name = format!("locus-provider-catalog-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container = crate::store::PostgresContainer::new(crate::store::PostgresConfig::for_test(
        container_name,
        volume_name,
        port,
    ));
    container
        .start()
        .await
        .expect("start provider catalog test database");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &test_backup_config(),
        )
        .await
        .expect("migrate provider catalog test database");

    let provider_id = Uuid::new_v4();
    let provider = ProviderReference::new(
        provider_id,
        "anthropic",
        KeychainReference::new("os-keychain://locus/anthropic").unwrap(),
    )
    .unwrap();
    store
        .persist_provider_reference(&provider)
        .await
        .expect("persist provider reference");
    store
        .persist_provider_models(
            provider_id,
            &[
                ProviderModel::new(provider_id, "claude-opus-4-6")
                    .unwrap()
                    .with_alias("Opus")
                    .unwrap(),
                ProviderModel::new(provider_id, "retired-model")
                    .unwrap()
                    .exclude_from_selector(),
            ],
        )
        .await
        .expect("persist provider model catalog");
    store
        .persist_provider_verification_metadata(
            provider_id,
            &ProviderVerificationMetadata::new(
                "2026-03-13T12:00:00Z",
                2,
                VerificationStatus::Failed,
            )
            .unwrap(),
        )
        .await
        .expect("persist provider verification metadata");

    assert_eq!(
        store
            .provider_selector_projection(provider_id)
            .await
            .expect("load provider selector projection"),
        vec![ModelSelectorOption {
            provider_id,
            model_id: "claude-opus-4-6".into(),
            label: "Opus".into(),
        }]
    );
    let metadata: (i32, String) = sqlx::query_as(
        "SELECT verification_model_count, verification_status
         FROM core.providers
         WHERE id = $1",
    )
    .bind(provider_id)
    .fetch_one(store.pool())
    .await
    .expect("load persisted verification metadata");
    assert_eq!(metadata, (2, "failed".into()));
}
