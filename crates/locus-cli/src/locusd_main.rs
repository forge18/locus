//! `locusd` — the background service that outlives the window.
//!
//! PLAN.md §Process topology: "It runs as a background service; closing the app detaches
//! the UI and nothing else. Runs keep streaming into Postgres, schedules keep firing, and
//! reopening the window re-attaches to state that never stopped."
//!
//! Until now that was a struct with no process behind it. This is the process: it builds
//! the one `Core`, binds `/run/locus.sock`, and serves the agent CLI from the same graph
//! the desktop host reads.

use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use locus_core::{
    core::Core,
    ids::{RunId, SessionId, TaskId},
    lsp::{parse_cli_request, LanguageCatalog},
    runtime::{
        container::{ContainerRuntime, DebugAdapterLaunch, DockerContainerRuntime},
        daemon::{
            agent_registration_root, bind_agent_socket, read_agent_registrations,
            serve_agent_socket_shared, AgentSocketCapabilities, AgentSocketError,
            AgentSocketRouter, AgentSocketVerb,
        },
        dap::{DapError, DebugSessionRegistry},
    },
    services::handoff::{HandoffContext, HandoffPayload, HandoffRegistry, HandoffTrigger},
};

const DEFAULT_HARNESS_REGISTRY: &str = "harnesses";
const DEFAULT_SOCKET_PATH: &str = "/run/locus.sock";

/// Routes LSP requests against the workspace owned by the connected run.
///
/// The capability map still supplies run ownership at the socket boundary. This router only
/// chooses the server from the frozen descriptor catalog and never accepts a repository installer
/// or a caller-selected executable.
struct LspRouter {
    catalog: LanguageCatalog,
}

impl LspRouter {
    fn is_lsp(verb: AgentSocketVerb) -> bool {
        matches!(
            verb,
            AgentSocketVerb::LspDef
                | AgentSocketVerb::LspRefs
                | AgentSocketVerb::LspHover
                | AgentSocketVerb::LspSymbols
                | AgentSocketVerb::LspDiagnostics
                | AgentSocketVerb::LspRename
        )
    }

    fn is_lsp_name(value: &str) -> bool {
        matches!(
            value,
            "lsp.def" | "lsp.refs" | "lsp.hover" | "lsp.symbols" | "lsp.diagnostics" | "lsp.rename"
        )
    }

    fn lease(&self, args: &[String]) -> std::result::Result<serde_json::Value, AgentSocketError> {
        let (verb, query_args) = args.split_first().ok_or_else(|| {
            AgentSocketError::unavailable("LSP lease requires the requested verb")
        })?;
        let request = parse_cli_request(verb, query_args)
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
        if request.path.is_absolute() {
            return Err(AgentSocketError::unavailable(
                "LSP lease paths must be relative to the authenticated workspace",
            ));
        }
        let descriptor = self
            .catalog
            .execution_descriptor_for_path(Path::new(&request.path))
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
        serde_json::to_value(descriptor)
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))
    }
}

impl AgentSocketRouter for LspRouter {
    fn authorize(
        &self,
        run_id: RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<(), AgentSocketError> {
        if verb == AgentSocketVerb::LspLease {
            if args.first().is_none_or(|value| !Self::is_lsp_name(value)) {
                return Err(AgentSocketError::unavailable(
                    "LSP lease must name a supported LSP verb",
                ));
            }
            let _ = run_id;
            return self.lease(args).map(|_| ());
        }
        if !Self::is_lsp(verb) {
            return Err(AgentSocketError::unavailable(format!(
                "`{verb}` is not routed by the LSP executor"
            )));
        }
        let _ = run_id;
        parse_cli_request(&verb.to_string(), args)
            .map(|_| ())
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))
    }

    fn route(
        &self,
        run_id: RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<serde_json::Value, AgentSocketError> {
        if verb == AgentSocketVerb::LspLease {
            return self.lease(args);
        }
        if !Self::is_lsp(verb) {
            return Err(AgentSocketError::unavailable(format!(
                "`{verb}` is not routed by the LSP executor"
            )));
        }
        let _ = run_id;
        Err(AgentSocketError::unavailable(
            "LSP requests must execute inside the authenticated run",
        ))
    }
}

struct DebugRouter {
    registry: DebugSessionRegistry,
    capabilities: AgentSocketCapabilities,
    /// The production router owns a shared host runtime that launches the adapter through the
    /// authenticated run container. Tests opt into the recording seam explicitly.
    container_runtime: Option<Arc<Mutex<Box<dyn ContainerRuntime>>>>,
    recording_for_tests: bool,
}

impl DebugRouter {
    fn is_debug(verb: AgentSocketVerb) -> bool {
        matches!(
            verb,
            AgentSocketVerb::DebugStart
                | AgentSocketVerb::DebugBreak
                | AgentSocketVerb::DebugStep
                | AgentSocketVerb::DebugRun
                | AgentSocketVerb::DebugNext
                | AgentSocketVerb::DebugFinish
                | AgentSocketVerb::DebugContinue
                | AgentSocketVerb::DebugStop
                | AgentSocketVerb::DebugStack
                | AgentSocketVerb::DebugVars
                | AgentSocketVerb::DebugEval
        )
    }

    fn option(
        args: &[String],
        name: &str,
    ) -> std::result::Result<Option<String>, AgentSocketError> {
        let mut value = None;
        let mut index = 0;
        while index < args.len() {
            if args[index] == name {
                let next = args.get(index + 1).ok_or_else(|| {
                    AgentSocketError::unavailable(format!("{name} requires a value"))
                })?;
                if next.starts_with("--") || next.trim().is_empty() {
                    return Err(AgentSocketError::unavailable(format!(
                        "{name} requires a value"
                    )));
                }
                if value.replace(next.clone()).is_some() {
                    return Err(AgentSocketError::unavailable(format!(
                        "duplicate option {name}"
                    )));
                }
                index += 2;
            } else {
                index += 1;
            }
        }
        Ok(value)
    }

    fn validate_tokens(
        args: &[String],
        positional: usize,
        allowed_options: &[&str],
    ) -> std::result::Result<(), AgentSocketError> {
        if args.len() < positional {
            return Err(AgentSocketError::unavailable("missing debug argument"));
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut index = positional;
        while index < args.len() {
            let option = args[index].as_str();
            if !allowed_options.contains(&option) || !seen.insert(option) {
                return Err(AgentSocketError::unavailable(format!(
                    "unknown or duplicate debug option: {option}"
                )));
            }
            let value = args.get(index + 1).ok_or_else(|| {
                AgentSocketError::unavailable(format!("{option} requires a value"))
            })?;
            if value.starts_with("--") || value.trim().is_empty() {
                return Err(AgentSocketError::unavailable(format!(
                    "{option} requires a value"
                )));
            }
            index += 2;
        }
        Ok(())
    }

    fn location(args: &[String]) -> std::result::Result<String, AgentSocketError> {
        let location = args
            .first()
            .ok_or_else(|| AgentSocketError::unavailable("debug break requires FILE:LINE"))?;
        let Some((file, line)) = location.rsplit_once(':') else {
            return Err(AgentSocketError::unavailable(
                "debug break location must be FILE:LINE",
            ));
        };
        if file.trim().is_empty() || line.parse::<u32>().map_or(true, |line| line == 0) {
            return Err(AgentSocketError::unavailable(
                "debug break location must be FILE:LINE",
            ));
        }
        Ok(location.clone())
    }

    fn authorize_debug(
        &self,
        run_id: RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<(), AgentSocketError> {
        match verb {
            AgentSocketVerb::DebugStart => {
                Self::validate_tokens(args, 0, &["--config"])?;
                let config_name = Self::option(args, "--config")?.ok_or_else(|| {
                    AgentSocketError::unavailable("debug start requires --config")
                })?;
                let config = self
                    .capabilities
                    .debug_config(run_id, &config_name)
                    .map_err(|error| AgentSocketError::unavailable(error.to_string()))?
                    .ok_or_else(|| {
                        AgentSocketError::unavailable(format!(
                            "debug config `{config_name}` is not available"
                        ))
                    })?;
                let adapters = self
                    .capabilities
                    .debug_adapters(run_id)
                    .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
                if !adapters.contains(config.adapter()) {
                    return Err(AgentSocketError::unavailable(format!(
                        "debug adapter plugin `{}` is not available",
                        config.adapter()
                    )));
                }
            }
            AgentSocketVerb::DebugBreak => {
                let _ = Self::location(args)?;
                Self::validate_tokens(args, 1, &["--if", "--log"])?;
                let _ = Self::option(args, "--if")?;
                let _ = Self::option(args, "--log")?;
            }
            AgentSocketVerb::DebugVars => {
                Self::validate_tokens(args, 0, &["--frame"])?;
                if let Some(frame) = Self::option(args, "--frame")? {
                    frame
                        .parse::<u32>()
                        .map_err(|_| AgentSocketError::unavailable("--frame must be an integer"))?;
                }
            }
            AgentSocketVerb::DebugEval => {
                if args.join(" ").trim().is_empty() {
                    return Err(AgentSocketError::unavailable(
                        "debug eval requires an expression",
                    ));
                }
            }
            AgentSocketVerb::DebugStep
            | AgentSocketVerb::DebugRun
            | AgentSocketVerb::DebugNext
            | AgentSocketVerb::DebugFinish
            | AgentSocketVerb::DebugContinue
            | AgentSocketVerb::DebugStop
            | AgentSocketVerb::DebugStack => {
                if !args.is_empty() {
                    return Err(AgentSocketError::unavailable(format!(
                        "{verb} does not accept arguments"
                    )));
                }
            }
            _ => return Err(AgentSocketError::unavailable("not a debug verb")),
        }
        Ok(())
    }

    fn route_debug(
        &self,
        run_id: RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<serde_json::Value, AgentSocketError> {
        let result = match verb {
            AgentSocketVerb::DebugStart => {
                let config_name = Self::option(args, "--config")?.ok_or_else(|| {
                    AgentSocketError::unavailable("debug start requires --config")
                })?;
                let config = self
                    .capabilities
                    .debug_config(run_id, &config_name)
                    .map_err(|error| AgentSocketError::unavailable(error.to_string()))?
                    .ok_or_else(|| {
                        AgentSocketError::unavailable(format!(
                            "debug config `{config_name}` is not available"
                        ))
                    })?;
                let adapters = self
                    .capabilities
                    .debug_adapters(run_id)
                    .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
                let snapshot = if let Some(runtime) = &self.container_runtime {
                    let launch = DebugAdapterLaunch::new(
                        format!("locus-agent-{run_id}"),
                        config.adapter_command(),
                    )
                    .map_err(|error| {
                        AgentSocketError::unavailable(format!(
                            "debug adapter runtime unavailable: {error}"
                        ))
                    })?;
                    let process = runtime
                        .lock()
                        .map_err(|_| {
                            AgentSocketError::unavailable(
                                "debug adapter runtime unavailable: container runtime lock is poisoned",
                            )
                        })?
                        .launch_debug_adapter(&launch)
                        .map_err(|error| {
                            AgentSocketError::unavailable(format!(
                                "debug adapter runtime unavailable: {error}"
                            ))
                        })?;
                    self.registry.start_with_process(
                        run_id,
                        config.adapter().to_owned(),
                        config.command().to_owned(),
                        adapters,
                        process,
                    )
                } else if self.recording_for_tests {
                    self.registry.start(
                        run_id,
                        config.adapter().to_owned(),
                        config.command().to_owned(),
                        adapters,
                    )
                } else {
                    Err(DapError::AdapterRuntimeUnavailable(
                        "locusd has no container runtime".into(),
                    ))
                };
                snapshot.map(|snapshot| {
                    serde_json::to_value(snapshot).expect("debug snapshot serializes")
                })
            }
            AgentSocketVerb::DebugBreak => {
                let location = Self::location(args)?;
                let condition = Self::option(args, "--if")?;
                let log_message = Self::option(args, "--log")?;
                self.registry
                    .set_breakpoint(run_id, &location, condition, log_message)
                    .map(|snapshot| {
                        serde_json::to_value(snapshot).expect("debug snapshot serializes")
                    })
            }
            AgentSocketVerb::DebugStop => self
                .registry
                .stop(run_id)
                .map(|snapshot| serde_json::to_value(snapshot).expect("debug snapshot serializes")),
            AgentSocketVerb::DebugStep => {
                self.registry.command(run_id, "step", serde_json::json!({}))
            }
            AgentSocketVerb::DebugRun => {
                self.registry.command(run_id, "run", serde_json::json!({}))
            }
            AgentSocketVerb::DebugNext => {
                self.registry.command(run_id, "next", serde_json::json!({}))
            }
            AgentSocketVerb::DebugFinish => {
                self.registry
                    .command(run_id, "finish", serde_json::json!({}))
            }
            AgentSocketVerb::DebugContinue => {
                self.registry
                    .command(run_id, "continue", serde_json::json!({}))
            }
            AgentSocketVerb::DebugStack => {
                self.registry
                    .command(run_id, "stack", serde_json::json!({}))
            }
            AgentSocketVerb::DebugVars => {
                let frame =
                    Self::option(args, "--frame")?.and_then(|value| value.parse::<u32>().ok());
                self.registry
                    .command(run_id, "vars", serde_json::json!({"frameId": frame}))
            }
            AgentSocketVerb::DebugEval => self.registry.command(
                run_id,
                "eval",
                serde_json::json!({"expression": args.join(" ")}),
            ),
            _ => Err(DapError::InvalidRun),
        };
        result.map_err(|error| AgentSocketError::unavailable(error.to_string()))
    }
}

struct HandoffRouter {
    registry: Arc<Mutex<HandoffRegistry>>,
    capabilities: AgentSocketCapabilities,
    successor_contexts: Arc<Mutex<BTreeMap<RunId, HandoffContext>>>,
}

impl HandoffRouter {
    fn parse(args: &[String]) -> std::result::Result<(String, String), AgentSocketError> {
        if args.len() < 3 || args[1] != "--why" {
            return Err(AgentSocketError::unavailable(
                "handoff requires <agent> --why <reason>",
            ));
        }
        let target = args[0].trim();
        let reason = args[2..].join(" ");
        if target.is_empty() || target.starts_with("--") {
            return Err(AgentSocketError::unavailable(
                "handoff requires a target agent",
            ));
        }
        if reason.trim().is_empty() {
            return Err(AgentSocketError::unavailable(
                "handoff reason must not be empty",
            ));
        }
        Ok((target.to_owned(), reason))
    }

    fn authorize(
        &self,
        _run_id: RunId,
        args: &[String],
    ) -> std::result::Result<(), AgentSocketError> {
        Self::parse(args).map(|_| ())
    }

    fn context(&self, run_id: RunId) -> std::result::Result<HandoffContext, AgentSocketError> {
        if let Some(context) = self
            .successor_contexts
            .lock()
            .map_err(|_| AgentSocketError::unavailable("handoff context lock is poisoned"))?
            .get(&run_id)
            .cloned()
        {
            return Ok(context);
        }
        self.capabilities
            .handoff_context(run_id)
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))?
            .ok_or_else(|| {
                AgentSocketError::unavailable(format!(
                    "handoff context for run {run_id} is unavailable"
                ))
            })
    }

    fn route(
        &self,
        run_id: RunId,
        args: &[String],
    ) -> std::result::Result<serde_json::Value, AgentSocketError> {
        let (target_agent, reason) = Self::parse(args)?;
        let predecessor = self.context(run_id)?;
        let successor = HandoffContext::new(
            predecessor.project_id,
            SessionId::generate(),
            RunId::generate(),
            predecessor.branch.clone(),
            predecessor.task.clone(),
        )
        .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
        let payload = HandoffPayload::new(
            format!("Continue task {}", predecessor.task),
            predecessor.branch.clone(),
            predecessor.task.clone(),
        )
        .and_then(|payload| payload.with_attempted(reason, None::<String>))
        .and_then(|payload| payload.with_open("successor must continue from this payload"))
        .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
        let record = self
            .registry
            .lock()
            .map_err(|_| AgentSocketError::unavailable("handoff registry lock is poisoned"))?
            .transfer(
                predecessor.clone(),
                successor.clone(),
                target_agent,
                HandoffTrigger::HumanReassignment,
                payload,
            )
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))?;
        self.successor_contexts
            .lock()
            .map_err(|_| AgentSocketError::unavailable("handoff context lock is poisoned"))?
            .insert(successor.run_id, successor);
        serde_json::to_value(record)
            .map_err(|error| AgentSocketError::unavailable(error.to_string()))
    }
}

struct DaemonRouter {
    lsp: LspRouter,
    debug: DebugRouter,
    handoff: HandoffRouter,
}

impl AgentSocketRouter for DaemonRouter {
    fn authorize(
        &self,
        run_id: locus_core::ids::RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<(), AgentSocketError> {
        if DebugRouter::is_debug(verb) {
            return self.debug.authorize_debug(run_id, verb, args);
        }
        if verb == AgentSocketVerb::Handoff {
            return self.handoff.authorize(run_id, args);
        }
        self.lsp.authorize(run_id, verb, args)
    }

    fn route(
        &self,
        run_id: locus_core::ids::RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<serde_json::Value, AgentSocketError> {
        if DebugRouter::is_debug(verb) {
            return self.debug.route_debug(run_id, verb, args);
        }
        if verb == AgentSocketVerb::Handoff {
            return self.handoff.route(run_id, args);
        }
        self.lsp.route(run_id, verb, args)
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
    let catalog = core.lsp().catalog().clone();

    let container_runtime = DockerContainerRuntime::connect()
        .ok()
        .map(|runtime| Arc::new(Mutex::new(Box::new(runtime) as Box<dyn ContainerRuntime>)));
    let runtime = tokio::runtime::Runtime::new().context("start the locusd runtime")?;
    runtime.block_on(async move {
        if let Ok(database_url) = env::var("DATABASE_URL") {
            core.connect(&database_url)
                .await
                .context("connect the store")?;
            let sync_core = core.clone();
            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(Duration::from_secs(1));
                let mut next_sync_at = BTreeMap::<TaskId, Instant>::new();
                loop {
                    ticker.tick().await;
                    let Some(store) = sync_core.store() else {
                        continue;
                    };
                    let intervals = match store.load_external_work_item_providers().await {
                        Ok(configs) => configs
                            .into_iter()
                            .map(|config| {
                                (
                                    (
                                        config.plugin_id.as_str().to_owned(),
                                        config.host,
                                        config.project,
                                    ),
                                    config.sync_interval_seconds,
                                )
                            })
                            .collect::<BTreeMap<_, _>>(),
                        Err(error) => {
                            eprintln!("external work-item sync configuration failed: {error}");
                            continue;
                        }
                    };
                    let items = match store.load_external_work_items().await {
                        Ok(items) => items,
                        Err(error) => {
                            eprintln!("external work-item sync discovery failed: {error}");
                            continue;
                        }
                    };
                    let now = Instant::now();
                    for item in items {
                        let identity = &item.snapshot.identity;
                        let interval = intervals
                            .get(&(
                                identity.plugin_id.as_str().to_owned(),
                                identity.host.clone(),
                                identity.project.clone(),
                            ))
                            .copied()
                            .unwrap_or(60)
                            .max(1);
                        if next_sync_at
                            .get(&item.task.id)
                            .is_some_and(|due| *due > now)
                        {
                            continue;
                        }
                        next_sync_at
                            .insert(item.task.id, now + Duration::from_secs(u64::from(interval)));
                        let _operation_lock = sync_core.work_item_operation_lock().lock().await;
                        if let Err(error) = sync_core.sync_external_work_item(item.task.id).await {
                            eprintln!(
                                "external work-item sync failed for {}: {error}",
                                item.task.id
                            );
                        }
                    }
                }
            });
        }

        let listener = bind_agent_socket(&socket)?;
        println!(
            "locusd serving {} harnesses on {}",
            core.registry().len(),
            socket.display()
        );

        // The run supervisor publishes one host-owned registration file per active run. A missing
        // registration is refused rather than falling back to a host-wide workspace.
        let capabilities = AgentSocketCapabilities::default();
        let registration_root = env::var("LOCUS_RUN_REGISTRY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| agent_registration_root(&socket));
        let watcher_capabilities = capabilities.clone();
        let cleanup_debug = core.debug().clone();
        tokio::spawn(async move {
            loop {
                let before = watcher_capabilities.run_ids().unwrap_or_default();
                let result = if registration_root.exists() {
                    read_agent_registrations(&registration_root)
                        .and_then(|registrations| watcher_capabilities.replace(&registrations))
                } else {
                    watcher_capabilities.replace(&[])
                };
                match result {
                    Ok(()) => {
                        let after = watcher_capabilities.run_ids().unwrap_or_default();
                        for run_id in before.difference(&after) {
                            cleanup_debug.end_run(*run_id);
                        }
                    }
                    Err(error) => {
                        eprintln!("agent registration reconciliation failed: {error}");
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
        let router = Arc::new(DaemonRouter {
            lsp: LspRouter { catalog },
            debug: DebugRouter {
                registry: core.debug().clone(),
                capabilities: capabilities.clone(),
                container_runtime,
                recording_for_tests: false,
            },
            handoff: HandoffRouter {
                registry: core.handoffs(),
                capabilities: capabilities.clone(),
                successor_contexts: Arc::new(Mutex::new(BTreeMap::new())),
            },
        });
        serve_agent_socket_shared(&listener, capabilities, router).await
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lease_resolves_a_descriptor_without_a_workspace_path() {
        let router = LspRouter {
            catalog: LanguageCatalog::builtin().unwrap(),
        };
        let lease = router
            .lease(&[
                "lsp.def".into(),
                "src/main.rs".into(),
                "1".into(),
                "0".into(),
            ])
            .unwrap();
        assert_eq!(lease["id"], "rust");
        assert_eq!(lease["command"][0], "rust-analyzer");
    }

    #[test]
    fn lease_rejects_an_unsupported_verb() {
        let router = LspRouter {
            catalog: LanguageCatalog::builtin().unwrap(),
        };
        assert!(router
            .lease(&["lsp.unknown".into(), "main.rs".into()])
            .is_err());
    }

    #[test]
    fn lease_rejects_an_absolute_workspace_path() {
        let router = LspRouter {
            catalog: LanguageCatalog::builtin().unwrap(),
        };
        let error = router
            .lease(&[
                "lsp.def".into(),
                "/etc/passwd".into(),
                "1".into(),
                "0".into(),
            ])
            .unwrap_err();
        assert!(error.to_string().contains("must be relative"));
    }

    fn debug_router_with_plugin() -> (DebugRouter, RunId) {
        let run_id = RunId::generate();
        let capabilities = AgentSocketCapabilities::default();
        capabilities
            .replace(&[locus_core::runtime::daemon::AgentRunRegistration {
                run_id,
                nonce: "nonce".into(),
                lsp_enabled: false,
                debug_adapters: vec!["python-debug-adapter".into()],
                debug_configs: std::collections::BTreeMap::from([(
                    "app".into(),
                    locus_core::services::project::DebugRunConfig::new(
                        "python-debug-adapter",
                        "python -m app",
                    )
                    .unwrap()
                    .with_adapter_command(["python-debug-adapter", "--stdio"])
                    .unwrap(),
                )]),
                handoff_context: None,
            }])
            .unwrap();
        (
            DebugRouter {
                registry: DebugSessionRegistry::default(),
                capabilities,
                container_runtime: None,
                recording_for_tests: true,
            },
            run_id,
        )
    }

    #[derive(Default)]
    struct TestAdapterProcess {
        alive: bool,
    }

    impl locus_core::runtime::dap::DebugAdapterProcess for TestAdapterProcess {
        fn request(
            &mut self,
            command: &str,
            _arguments: serde_json::Value,
        ) -> std::result::Result<locus_core::runtime::dap::DapResponse, DapError> {
            if !self.alive {
                return Err(DapError::AdapterRequestFailed("adapter is stopped".into()));
            }
            Ok(locus_core::runtime::dap::DapResponse {
                seq: 0,
                message_type: "response".into(),
                request_seq: 0,
                success: true,
                command: command.into(),
                message: None,
                body: Some(serde_json::json!({})),
            })
        }

        fn terminate(&mut self) {
            self.alive = false;
        }

        fn is_alive(&self) -> bool {
            self.alive
        }
    }

    struct TestAdapterRuntime {
        launches: Arc<Mutex<Vec<DebugAdapterLaunch>>>,
    }

    impl ContainerRuntime for TestAdapterRuntime {
        fn build_or_reuse_image(
            &mut self,
            _image: &str,
        ) -> anyhow::Result<locus_core::runtime::container::ImageDisposition> {
            Ok(locus_core::runtime::container::ImageDisposition::Reused)
        }

        fn start_container(
            &mut self,
            _container: &locus_core::runtime::container::ContainerLaunch,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        fn attach_pty(
            &mut self,
            _container: &str,
            _attachment: locus_core::sandbox::mounts::PtyAttachment,
            _stream: locus_core::runtime::container::PtyStream,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        fn stop_container(&mut self, _container: &str) -> anyhow::Result<()> {
            Ok(())
        }

        fn launch_debug_adapter(
            &mut self,
            launch: &DebugAdapterLaunch,
        ) -> anyhow::Result<Box<dyn locus_core::runtime::dap::DebugAdapterProcess>> {
            self.launches
                .lock()
                .expect("test adapter launch lock")
                .push(launch.clone());
            Ok(Box::new(TestAdapterProcess { alive: true }))
        }
    }

    #[test]
    fn debug_start_launches_adapter_through_container_runtime() {
        let (mut router, run_id) = debug_router_with_plugin();
        let launches = Arc::new(Mutex::new(Vec::new()));
        router.container_runtime = Some(Arc::new(Mutex::new(Box::new(TestAdapterRuntime {
            launches: launches.clone(),
        })
            as Box<dyn ContainerRuntime>)));
        router.recording_for_tests = false;

        let result = router
            .route_debug(
                run_id,
                AgentSocketVerb::DebugStart,
                &["--config".into(), "app".into()],
            )
            .unwrap();
        let launches = launches.lock().unwrap();
        assert_eq!(launches.len(), 1);
        assert_eq!(launches[0].container, format!("locus-agent-{run_id}"));
        assert_eq!(launches[0].command, ["python-debug-adapter", "--stdio"]);
        assert_eq!(result["status"], "running");
    }

    #[test]
    fn debug_start_does_not_fall_back_to_a_recording_adapter() {
        let (mut router, run_id) = debug_router_with_plugin();
        router.recording_for_tests = false;
        let error = router
            .route_debug(
                run_id,
                AgentSocketVerb::DebugStart,
                &["--config".into(), "app".into()],
            )
            .unwrap_err();
        assert!(error.to_string().contains("no container runtime"));
    }

    #[test]
    fn handoff_route_closes_predecessor_and_opens_successor() {
        let run_id = RunId::generate();
        let project_id = locus_core::ids::ProjectId::generate();
        let session_id = SessionId::generate();
        let context =
            HandoffContext::new(project_id, session_id, run_id, "agent/feature", "task-17")
                .unwrap();
        let capabilities = AgentSocketCapabilities::default();
        capabilities
            .replace(&[locus_core::runtime::daemon::AgentRunRegistration {
                run_id,
                nonce: "nonce".into(),
                lsp_enabled: false,
                debug_adapters: Vec::new(),
                debug_configs: BTreeMap::new(),
                handoff_context: Some(context),
            }])
            .unwrap();
        let registry = Arc::new(Mutex::new(HandoffRegistry::default()));
        let router = HandoffRouter {
            registry: registry.clone(),
            capabilities,
            successor_contexts: Arc::new(Mutex::new(BTreeMap::new())),
        };
        router
            .authorize(
                run_id,
                &[
                    "auditor".into(),
                    "--why".into(),
                    "context".into(),
                    "exhausted".into(),
                ],
            )
            .unwrap();
        let result = router
            .route(
                run_id,
                &[
                    "auditor".into(),
                    "--why".into(),
                    "context".into(),
                    "exhausted".into(),
                ],
            )
            .unwrap();
        let successor = result["successor_session_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let state = registry.lock().unwrap();
        assert_eq!(state.predecessor(successor), Some(session_id));
        assert_eq!(
            state.session(session_id).unwrap().status,
            locus_core::services::handoff::HandoffSessionStatus::Closed
        );
        assert_eq!(state.session(successor).unwrap().branch, "agent/feature");
        assert_eq!(state.session(successor).unwrap().task, "task-17");
    }

    #[test]
    fn debug_start_uses_the_run_plugin_config() {
        let (router, run_id) = debug_router_with_plugin();
        router
            .authorize_debug(
                run_id,
                AgentSocketVerb::DebugStart,
                &["--config".into(), "app".into()],
            )
            .unwrap();
        let result = router
            .route_debug(
                run_id,
                AgentSocketVerb::DebugStart,
                &["--config".into(), "app".into()],
            )
            .unwrap();
        assert_eq!(result["adapter"], "python-debug-adapter");
        assert_eq!(result["run_command"], "python -m app");
    }

    #[test]
    fn debug_start_rejects_an_unallowlisted_plugin() {
        let (router, run_id) = debug_router_with_plugin();
        router
            .capabilities
            .set_debug_adapters(run_id, std::iter::empty())
            .unwrap();
        let error = router
            .authorize_debug(
                run_id,
                AgentSocketVerb::DebugStart,
                &["--config".into(), "app".into()],
            )
            .unwrap_err();
        assert!(error.to_string().contains("not available"));
    }
}
