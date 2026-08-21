//! In-process event delivery for the Locus core.

use anyhow::{bail, Context, Result};
use sqlx::{postgres::PgListener, query, PgPool};
use tokio::sync::broadcast;

const POSTGRES_CHANNEL: &str = "locus_events";
const POSTGRES_NOTIFY_PAYLOAD_CAP_BYTES: usize = 8_000;

/// Delivers event ids to Locus processes through Postgres LISTEN/NOTIFY.
pub struct PostgresBus {
    pool: PgPool,
}

impl PostgresBus {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Opens a dedicated connection that receives notifications from other processes.
    pub async fn subscribe(&self) -> Result<PostgresSubscription> {
        let mut listener = PgListener::connect_with(&self.pool)
            .await
            .context("connect Postgres event listener")?;
        listener
            .listen(POSTGRES_CHANNEL)
            .await
            .context("listen for Postgres events")?;

        Ok(PostgresSubscription { listener })
    }

    /// Publishes an event id; the event payload remains in Postgres for subscribers to fetch.
    pub async fn publish(&self, event_id: &str) -> Result<()> {
        if event_id.len() > POSTGRES_NOTIFY_PAYLOAD_CAP_BYTES {
            bail!(
                "Postgres NOTIFY payload exceeds 8000-byte cap: {} bytes",
                event_id.len()
            );
        }

        query("SELECT pg_notify($1, $2)")
            .bind(POSTGRES_CHANNEL)
            .bind(event_id)
            .execute(&self.pool)
            .await
            .context("notify Postgres event subscribers")?;

        Ok(())
    }
}

/// A dedicated Postgres notification subscription.
pub struct PostgresSubscription {
    listener: PgListener,
}

impl PostgresSubscription {
    /// Waits for the next event id published by another process.
    pub async fn recv(&mut self) -> Result<String> {
        Ok(self
            .listener
            .recv()
            .await
            .context("receive Postgres event notification")?
            .payload()
            .to_owned())
    }
}

/// Broadcasts events to all subscribers in this process.
#[derive(Clone)]
pub struct InProcessBus<T> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone> InProcessBus<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    /// Sends an event to current subscribers, returning their count.
    pub fn publish(&self, event: T) -> usize {
        self.sender.send(event).unwrap_or(0)
    }
}

#[cfg(test)]
mod notify_across_processes {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
        time::Duration,
    };

    use tokio::time::timeout;

    use super::PostgresBus;
    use crate::store::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn notify_across_processes() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the isolated pgvector container");

        let listener_store = Store::connect(&container.database_url())
            .await
            .expect("connect listener store");
        let publisher_store = Store::connect(&container.database_url())
            .await
            .expect("connect publisher store");
        let listener_bus = PostgresBus::new(listener_store.pool().clone());
        let publisher_bus = PostgresBus::new(publisher_store.pool().clone());
        let mut subscription = listener_bus.subscribe().await.expect("listen for events");

        publisher_bus
            .publish("00000000-0000-0000-0000-000000000001")
            .await
            .expect("notify other process");
        let notification = timeout(Duration::from_secs(5), subscription.recv())
            .await
            .expect("notification arrives before timeout")
            .expect("receive notification");

        assert_eq!(notification, "00000000-0000-0000-0000-000000000001");
    }
}

#[cfg(test)]
mod notify_payload_cap {
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;

    use super::PostgresBus;

    #[tokio::test]
    async fn rejects_payloads_over_8000_bytes_before_notifying() {
        let bus = PostgresBus::new(
            PgPoolOptions::new()
                .acquire_timeout(Duration::from_millis(100))
                .connect_lazy("postgres://locus@127.0.0.1:1/locus")
                .expect("create a disconnected test pool"),
        );
        let payload = "x".repeat(8_001);

        let error = bus
            .publish(&payload)
            .await
            .expect_err("payloads over the PostgreSQL cap are rejected");

        assert!(
            error
                .to_string()
                .contains("Postgres NOTIFY payload exceeds 8000-byte cap"),
            "unexpected error: {error:#}"
        );
    }
}

#[cfg(test)]
mod in_process {
    use super::InProcessBus;

    #[tokio::test]
    async fn broadcasts_to_every_subscriber() {
        let bus = InProcessBus::new(4);
        let mut first = bus.subscribe();
        let mut second = bus.subscribe();

        assert_eq!(bus.publish("run.completed"), 2);
        assert_eq!(first.recv().await.expect("first event"), "run.completed");
        assert_eq!(second.recv().await.expect("second event"), "run.completed");
    }
}
