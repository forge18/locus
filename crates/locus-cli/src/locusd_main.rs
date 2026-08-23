//! `locusd` — the background service that outlives the window.
//!
//! PLAN.md §Process topology: "It runs as a background service; closing the app detaches
//! the UI and nothing else. Runs keep streaming into Postgres, schedules keep firing, and
//! reopening the window re-attaches to state that never stopped."
//!
//! Until now that was a struct with no process behind it. This is the process: it builds
//! the one `Core`, binds `/run/locus.sock`, and serves the agent CLI from the same graph
//! the desktop host reads.

use std::{collections::BTreeMap, env, path::PathBuf};

use anyhow::{Context, Result};
use locus_core::{
    core::Core,
    runtime::daemon::{
        bind_agent_socket, serve_agent_socket, AgentSocketError, AgentSocketRouter, AgentSocketVerb,
    },
};

const DEFAULT_HARNESS_REGISTRY: &str = "harnesses";
const DEFAULT_SOCKET_PATH: &str = "/run/locus.sock";

/// Refuses every verb, by name.
///
/// The routing table is the run supervisor's, and it does not exist yet. Answering with a
/// refusal that says so is honest; answering with a plausible empty result would not be.
struct UnroutedVerbs;

impl AgentSocketRouter for UnroutedVerbs {
    fn route(
        &self,
        _run_id: locus_core::ids::RunId,
        verb: AgentSocketVerb,
        _args: &[String],
    ) -> std::result::Result<serde_json::Value, AgentSocketError> {
        Err(AgentSocketError::unavailable(format!(
            "`{verb}` has no route yet: locusd is running, the verb is not wired"
        )))
    }
}

fn main() -> Result<()> {
    let registry = env::var("LOCUS_HARNESS_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_HARNESS_REGISTRY));
    let socket = env::var("LOCUS_SOCKET_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_SOCKET_PATH));

    let core = Core::load(&registry).context("load the harness registry")?;

    let runtime = tokio::runtime::Runtime::new().context("start the locusd runtime")?;
    runtime.block_on(async move {
        if let Ok(database_url) = env::var("DATABASE_URL") {
            core.connect(&database_url)
                .await
                .context("connect the store")?;
        }

        let listener = bind_agent_socket(&socket)?;
        println!(
            "locusd serving {} harnesses on {}",
            core.registry().len(),
            socket.display()
        );

        // Capabilities are minted per run by the supervisor; with no runs yet the map is
        // empty, so every request is refused at the capability check.
        let capabilities = BTreeMap::new();
        serve_agent_socket(&listener, &capabilities, &UnroutedVerbs).await
    })
}
