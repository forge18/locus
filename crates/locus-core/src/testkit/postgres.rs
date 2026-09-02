//! One isolated Postgres container per test, and the fixtures every DB test needs.
//!
//! These were copied into ten modules before this existed — `DockerCleanup` and
//! `unused_port` ten times each, `NoopMigrationBackup` eight. A fixture duplicated per
//! module drifts per module, and a container left running is a port conflict in the next
//! test run.

use std::{
    fs::{File, OpenOptions},
    net::TcpListener,
    os::fd::AsRawFd,
    process::{Command, Stdio},
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use anyhow::Result;

use crate::store::{
    backup::{MigrationBackup, RetainedBackupConfig},
    PostgresConfig, PostgresContainer,
};

/// Docker Desktop can fail concurrent PostgreSQL initialization, so container tests in
/// one process run one at a time. This lives here rather than on `PostgresContainer`:
/// serialization is a property of the test harness, not of the type under test.
/// Async-aware: the guard is held across the container start `await`, which a
/// `std::sync::Mutex` cannot do without risking a deadlock on a multi-threaded runtime.
static TEST_POSTGRES_LOCK: OnceLock<Arc<tokio::sync::Mutex<()>>> = OnceLock::new();

/// Cargo starts separate test binaries concurrently, so the in-process lock alone cannot
/// serialize containers across those processes. The advisory file lock closes that gap.
struct ProcessPostgresLock(File);

impl ProcessPostgresLock {
    async fn acquire() -> Self {
        let path = std::env::temp_dir().join("locus-postgres-test.lock");
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .expect("create the Postgres test lock");
        loop {
            let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
            if result == 0 {
                return Self(file);
            }
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EAGAIN) {
                panic!("acquire the Postgres test lock: {error}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
}

impl Drop for ProcessPostgresLock {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) };
    }
}

/// The credential proxy listens on a fixed host port (PLAN.md §Credentials), so two tests
/// that each build their own proxy collide when run in parallel. Hold this for the length
/// of any test that starts one.
static FIXED_PORT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Serializes tests that start a Postgres container themselves rather than through
/// [`start_postgres`].
pub async fn serialize_postgres() -> tokio::sync::OwnedMutexGuard<()> {
    TEST_POSTGRES_LOCK
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
        .lock_owned()
        .await
}

/// Serializes tests that bind the credential proxy's fixed host port.
pub fn serialize_fixed_port() -> MutexGuard<'static, ()> {
    FIXED_PORT_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Removes the container and its volume when the test ends, pass or fail, and holds the
/// serialization guard for the length of the test.
pub struct DockerCleanup {
    container_name: String,
    volume_name: String,
    _process_serialized: ProcessPostgresLock,
    _serialized: tokio::sync::OwnedMutexGuard<()>,
}

impl DockerCleanup {
    /// For a test that drives the container lifecycle itself and cannot use
    /// [`start_postgres`].
    pub async fn new(container_name: impl Into<String>, volume_name: impl Into<String>) -> Self {
        Self {
            container_name: container_name.into(),
            volume_name: volume_name.into(),
            _process_serialized: ProcessPostgresLock::acquire().await,
            _serialized: serialize_postgres().await,
        }
    }
}

impl Drop for DockerCleanup {
    fn drop(&mut self) {
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

/// A port the OS says is free right now. Racy by nature, which is why the container name
/// carries it too — two concurrent runs get different names, not a collision.
pub fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
    listener.local_addr().expect("read the local port").port()
}

/// Migrations are gated on a retained backup. Tests exercise the schema, not the gate.
pub struct NoopMigrationBackup;

impl MigrationBackup for NoopMigrationBackup {
    fn create_retained(&self, _: &RetainedBackupConfig) -> Result<()> {
        Ok(())
    }
}

pub fn test_backup_config() -> RetainedBackupConfig {
    RetainedBackupConfig::new(
        "postgres://locus@localhost/locus",
        "/var/lib/locus/artifacts",
        "/var/lib/locus/backups",
    )
}

/// Start an isolated pgvector container. Hold the returned guard for the whole test.
pub async fn start_postgres() -> (PostgresContainer, DockerCleanup) {
    start_postgres_named("locus-postgres-test").await
}

/// As [`start_postgres`], with a prefix that names the test in `docker ps`.
pub async fn start_postgres_named(prefix: &str) -> (PostgresContainer, DockerCleanup) {
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("{prefix}-{suffix}");
    let volume_name = format!("{prefix}-data-{suffix}");
    let cleanup = DockerCleanup::new(container_name.clone(), volume_name.clone()).await;
    let container =
        PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
    container
        .start()
        .await
        .expect("start the isolated pgvector container");
    (container, cleanup)
}
