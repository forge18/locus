//! The composition root: the one place the subsystems in PLAN.md §Process topology are
//! assembled.
//!
//! Before this, nothing built the graph. The desktop host hand-constructed three leaf
//! objects as Tauri state, never held a `Store`, and re-read and re-parsed the whole
//! harness registry from disk on every invoke. Each new consumer wired again, so the
//! desktop host and the socket router were free to drift.
//!
//! `Core` is built once and shared as `Arc<Core>`: Tauri manages it, `locusd` serves the
//! agent socket from it, and both see the same registry, the same collector, and the same
//! store.

use std::{
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::{Context, Result};

use crate::{
    harness::registry::{load_from_directory, HarnessRegistry},
    ipc::{EventChannel, PtyChannel},
    runtime::daemon::Daemon,
    services::telemetry::EventCollector,
    store::Store,
};

/// How much fan-out each in-process channel buffers before a slow subscriber lags.
const CHANNEL_CAPACITY: usize = 1_024;

/// Everything that outlives a window.
///
/// PLAN.md §Process topology: "`locusd` outlives the window. It runs as a background
/// service; closing the app detaches the UI and nothing else."
pub struct Core {
    registry: HarnessRegistry,
    collector: EventCollector,
    pty: PtyChannel,
    events: EventChannel,
    /// Set once, by [`Core::connect`]. `Core` is shared as `Arc`, so the store cannot be
    /// assigned through `&mut self`.
    store: OnceLock<Store>,
    daemon: Mutex<Daemon>,
}

impl Core {
    /// Assemble everything that does not need a database.
    ///
    /// The registry is loaded once here rather than per request: it is the harness
    /// contract, and re-parsing eleven TOMLs on every invoke is work the process already
    /// did at start.
    pub fn load(harnesses: impl AsRef<Path>) -> Result<Arc<Self>> {
        let registry =
            load_from_directory(harnesses.as_ref()).context("load the harness registry")?;
        Ok(Arc::new(Self {
            registry,
            collector: EventCollector::new(CHANNEL_CAPACITY),
            pty: PtyChannel::new(CHANNEL_CAPACITY),
            events: EventChannel::new(CHANNEL_CAPACITY),
            store: OnceLock::new(),
            daemon: Mutex::new(Daemon::default()),
        }))
    }

    /// Attach the store. Separate from [`Core::load`] because the desktop shell starts
    /// before Postgres is reachable, and the registry surfaces do not need it.
    pub async fn connect(&self, database_url: &str) -> Result<&Store> {
        if let Some(store) = self.store.get() {
            return Ok(store);
        }
        let store = Store::connect(database_url)
            .await
            .context("connect the Locus store")?;
        Ok(self.store.get_or_init(|| store))
    }

    pub fn registry(&self) -> &HarnessRegistry {
        &self.registry
    }

    pub fn collector(&self) -> &EventCollector {
        &self.collector
    }

    pub fn pty(&self) -> &PtyChannel {
        &self.pty
    }

    pub fn events(&self) -> &EventChannel {
        &self.events
    }

    /// The store, once [`Core::connect`] has run. `None` means the shell is up but
    /// Postgres is not, which is a state the UI is expected to render rather than crash on.
    pub fn store(&self) -> Option<&Store> {
        self.store.get()
    }

    pub fn daemon(&self) -> &Mutex<Daemon> {
        &self.daemon
    }
}
