//! Provider credential references and the host-only OS-keychain boundary.
//!
//! A provider row can identify a keychain entry, but never contains the credential stored there.
//! The broker borrows a credential only while executing a host egress callback; it deliberately
//! does not expose a credential-returning API to persistence, events, or container launch code.

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
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

/// The complete durable provider record for this task.
///
/// Authentication configuration, verification state, and model catalogs intentionally arrive in
/// later tasks. Keeping this record narrow prevents them from inventing a secret-bearing field.
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
}

/// Platform adapters implement this trait with macOS Keychain, Windows Credential Manager, or a
/// Linux keyring. The durable reference contract above is independent of that implementation.
pub trait OsKeychain: Send + Sync {
    fn read_secret(&self, reference: &KeychainReference) -> Result<String>;
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
    pub fn with_secret<T>(
        &self,
        provider: &ProviderReference,
        egress: impl FnOnce(&str) -> Result<T>,
    ) -> Result<T> {
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
const SECRET: &str = "test-provider-secret";

#[cfg(test)]
struct TestKeychain;

#[cfg(test)]
impl OsKeychain for TestKeychain {
    fn read_secret(&self, _: &KeychainReference) -> Result<String> {
        Ok(SECRET.into())
    }
}

#[cfg(test)]
struct LeakyKeychain;

#[cfg(test)]
impl OsKeychain for LeakyKeychain {
    fn read_secret(&self, _: &KeychainReference) -> Result<String> {
        anyhow::bail!("keychain failed for {SECRET}")
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
        .with_secret::<()>(&provider, |secret| {
            anyhow::bail!("upstream rejected {secret}")
        })
        .unwrap_err();
    assert!(!egress_error.to_string().contains(SECRET));

    let keychain_error = ProviderBroker::new(LeakyKeychain)
        .with_secret::<()>(&provider, |_| Ok(()))
        .unwrap_err();
    assert!(!keychain_error.to_string().contains(SECRET));
}
