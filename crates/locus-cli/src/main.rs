//! `locus` — the CLI agents call from inside their container.
//!
//! A thin client over the daemon socket at /run/locus.sock. It holds no logic of its
//! own: every verb is a round trip to locus-core, so behaviour cannot drift between
//! what an agent sees and what the app sees.

use std::{env, path::PathBuf, process::Command};

use anyhow::{bail, Context, Result};
use locus_core::{
    harness::registry::load_from_directory,
    lsp::{execute_descriptor_query, parse_cli_request, LanguageDescriptor},
    services::{
        browse::{assertion_json, parse_assert_args, AssertionResult},
        lint::{run as run_linters, verify as verify_linters, LintRequest},
    },
    store::backup::{Backup, RetainedBackupConfig, SystemBackupFilesystem, SystemProcessRunner},
};

mod hook;
pub mod sock;

const DEFAULT_ARTIFACT_ROOT: &str = "/var/lib/locus/artifacts";
const DEFAULT_BACKUP_ROOT: &str = "/var/lib/locus/backups";
const DEFAULT_LINTER_ROOT: &str = "/locus/config/linters";

fn main() -> Result<()> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        None => {
            println!("locus {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("backup") => backup(),
        Some("lint") => lint(&arguments[1..]),
        Some("harness") => harness(),
        Some("ralph") => ralph(&arguments[1..]),
        Some("hook") => {
            let _ = hook::run();
            Ok(())
        }
        Some(_) => dispatch(&arguments),
    }
}

fn ralph(arguments: &[String]) -> Result<()> {
    let mut goal = None;
    let mut verify = None;
    let mut max_iterations = 8_u32;
    let mut index = 0;
    while let Some(argument) = arguments.get(index) {
        match argument.as_str() {
            "--json" => {}
            "--goal" => {
                index += 1;
                goal = Some(
                    arguments
                        .get(index)
                        .context("--goal requires a value")?
                        .clone(),
                );
            }
            "--verify" => {
                index += 1;
                verify = Some(
                    arguments
                        .get(index)
                        .context("--verify requires a value")?
                        .clone(),
                );
            }
            "--max-iterations" => {
                index += 1;
                max_iterations = arguments
                    .get(index)
                    .context("--max-iterations requires a value")?
                    .parse()
                    .context("--max-iterations must be an integer")?;
            }
            other => bail!("unknown ralph option: {other}"),
        }
        index += 1;
    }
    let goal = goal.context("--goal is required")?;
    let verify = verify.context("--verify is required")?;
    let result = locus_core::services::workflow::run_ralph(goal, verify, max_iterations)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

fn dispatch(arguments: &[String]) -> Result<()> {
    let arguments = sock::without_json_flag(arguments);
    let (verb, args) = sock::allowed_verb(&arguments)?;
    let runtime = tokio::runtime::Runtime::new().context("start socket client runtime")?;
    let nonce =
        env::var("LOCUS_RUN_NONCE").context("LOCUS_RUN_NONCE is required for daemon requests")?;
    if verb.verb == locus_core::runtime::daemon::AgentSocketVerb::BrowseAssert {
        parse_assert_args(args).context("invalid browse assert arguments")?;
        return dispatch_assert(&runtime, &nonce, verb, args);
    }
    if verb.verb == locus_core::runtime::daemon::AgentSocketVerb::Handoff {
        sock::validate_handoff_args(args)?;
    }
    if matches!(
        verb.verb,
        locus_core::runtime::daemon::AgentSocketVerb::TaskList
            | locus_core::runtime::daemon::AgentSocketVerb::TaskShow
            | locus_core::runtime::daemon::AgentSocketVerb::TaskMove
            | locus_core::runtime::daemon::AgentSocketVerb::TaskAssign
            | locus_core::runtime::daemon::AgentSocketVerb::TaskComment
    ) {
        sock::validate_task_args(verb.verb, args)?;
    }
    if matches!(
        verb.verb,
        locus_core::runtime::daemon::AgentSocketVerb::WikiSearch
            | locus_core::runtime::daemon::AgentSocketVerb::WikiRead
            | locus_core::runtime::daemon::AgentSocketVerb::WikiWrite
            | locus_core::runtime::daemon::AgentSocketVerb::WikiHistory
            | locus_core::runtime::daemon::AgentSocketVerb::WikiIngest
            | locus_core::runtime::daemon::AgentSocketVerb::WikiQuery
            | locus_core::runtime::daemon::AgentSocketVerb::WikiLint
    ) {
        sock::validate_wiki_args(verb.verb, args)?;
    }
    if matches!(
        verb.verb,
        locus_core::runtime::daemon::AgentSocketVerb::DebugStart
            | locus_core::runtime::daemon::AgentSocketVerb::DebugBreak
            | locus_core::runtime::daemon::AgentSocketVerb::DebugStep
            | locus_core::runtime::daemon::AgentSocketVerb::DebugRun
            | locus_core::runtime::daemon::AgentSocketVerb::DebugNext
            | locus_core::runtime::daemon::AgentSocketVerb::DebugFinish
            | locus_core::runtime::daemon::AgentSocketVerb::DebugContinue
            | locus_core::runtime::daemon::AgentSocketVerb::DebugStop
            | locus_core::runtime::daemon::AgentSocketVerb::DebugStack
            | locus_core::runtime::daemon::AgentSocketVerb::DebugVars
            | locus_core::runtime::daemon::AgentSocketVerb::DebugEval
    ) {
        sock::validate_debug_args(verb.verb, args)?;
    }
    if matches!(
        verb.verb,
        locus_core::runtime::daemon::AgentSocketVerb::LspDef
            | locus_core::runtime::daemon::AgentSocketVerb::LspRefs
            | locus_core::runtime::daemon::AgentSocketVerb::LspHover
            | locus_core::runtime::daemon::AgentSocketVerb::LspSymbols
            | locus_core::runtime::daemon::AgentSocketVerb::LspDiagnostics
            | locus_core::runtime::daemon::AgentSocketVerb::LspRename
    ) {
        sock::validate_lsp_args(verb.verb, args)?;
        return dispatch_lsp(&runtime, &nonce, verb, args);
    }
    let response = runtime.block_on(sock::dispatch(
        sock::DEFAULT_SOCKET_PATH,
        &nonce,
        verb,
        args,
    ))?;
    println!("{}", sock::compact_json(&sock::key_pack(response))?);
    Ok(())
}

/// Ask the host daemon for a pinned descriptor, then execute the server in this container's
/// `/workspace`. The host authorizes the capability; it never sees or indexes this clone.
fn dispatch_lsp(
    runtime: &tokio::runtime::Runtime,
    nonce: &str,
    verb: &sock::VerbDispatch,
    args: &[String],
) -> Result<()> {
    let lease_args = std::iter::once(verb.verb.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    let lease = runtime.block_on(sock::dispatch(
        sock::DEFAULT_SOCKET_PATH,
        nonce,
        &sock::LSP_LEASE_DISPATCH,
        &lease_args,
    ))?;
    let descriptor: LanguageDescriptor =
        serde_json::from_value(lease).context("decode host LSP descriptor lease")?;
    let request = parse_cli_request(&verb.verb.to_string(), args)?;
    // The host/agent tree boundary is fixed by the container contract. Do not let an agent
    // redirect an LSP query to another path through an environment override.
    let workspace = PathBuf::from("/workspace");
    let result = execute_descriptor_query(&descriptor, &request, &workspace)?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

fn dispatch_assert(
    runtime: &tokio::runtime::Runtime,
    nonce: &str,
    verb: &sock::VerbDispatch,
    args: &[String],
) -> Result<()> {
    match runtime.block_on(sock::dispatch(sock::DEFAULT_SOCKET_PATH, nonce, verb, args)) {
        Ok(response) => {
            let failed = response.get("passed").is_some_and(|passed| passed == false);
            println!("{}", sock::compact_json(&sock::key_pack(response))?);
            if failed {
                bail!("browse assertion failed")
            }
            Ok(())
        }
        Err(error) => {
            let result = AssertionResult {
                passed: false,
                failure: Some(locus_core::services::browse::AssertionFailure {
                    selector: args.first().cloned().unwrap_or_default(),
                    reason: error.to_string(),
                    expected: parse_assert_args(args)?,
                    actual: None,
                }),
            };
            println!("{}", sock::compact_json(&assertion_json(&result))?);
            bail!("browse assertion failed")
        }
    }
}

fn lint(arguments: &[String]) -> Result<()> {
    let mut request = LintRequest::default();
    let mut index = 0;
    while let Some(argument) = arguments.get(index) {
        match argument.as_str() {
            "--changed" => request.changed = true,
            "--only" => {
                index += 1;
                request.only = Some(
                    arguments
                        .get(index)
                        .context("--only requires a linter name")?
                        .clone(),
                );
            }
            other => bail!("unknown lint option: {other}"),
        }
        index += 1;
    }
    let project = env_path("LOCUS_WORKSPACE", ".");
    if request.changed {
        request.changed_paths = changed_paths(&project)?;
    }
    let report = run_linters(
        env_path("LOCUS_LINTER_ROOT", DEFAULT_LINTER_ROOT),
        &project,
        &request,
    )?;
    print!("{}", report.evidence());
    verify_linters(&report)
}

#[cfg(test)]
mod debug {
    use super::*;

    fn validate(arguments: &[&str]) -> locus_core::runtime::daemon::AgentSocketVerb {
        let arguments = arguments
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        let (dispatch, args) = sock::allowed_verb(&arguments).unwrap();
        sock::validate_debug_args(dispatch.verb, args).unwrap();
        dispatch.verb
    }

    #[test]
    fn cli_is_stateless() {
        assert_eq!(std::mem::size_of::<sock::SocketClient>(), 0);
        assert_eq!(
            validate(&["debug", "step"]),
            locus_core::runtime::daemon::AgentSocketVerb::DebugStep
        );
    }

    #[test]
    fn start() {
        assert_eq!(
            validate(&["debug", "start", "--config", "python"]),
            locus_core::runtime::daemon::AgentSocketVerb::DebugStart
        );
    }

    #[test]
    fn r#break() {
        assert_eq!(
            validate(&["debug", "break", "src/main.py:7", "--if", "ready"]),
            locus_core::runtime::daemon::AgentSocketVerb::DebugBreak
        );
    }

    #[test]
    fn logpoint_continues() {
        validate(&["debug", "break", "src/main.py:7", "--log", "x={x}"]);
    }

    #[test]
    fn stepping() {
        for verb in ["run", "step", "next", "finish", "continue"] {
            validate(&["debug", verb]);
        }
    }

    #[test]
    fn inspection() {
        validate(&["debug", "stack"]);
        validate(&["debug", "vars", "--frame", "2"]);
        validate(&["debug", "eval", "items.length"]);
    }

    #[test]
    fn stop() {
        validate(&["debug", "stop"]);
    }

    #[test]
    fn honest_unavailable() {
        let error = locus_core::runtime::dap::DebugSessionRegistry::default()
            .start(
                locus_core::ids::RunId::generate(),
                "debugpy",
                "python -m app",
                std::iter::empty::<String>(),
            )
            .unwrap_err();
        assert!(error.to_string().contains("not available"));
    }
}

fn changed_paths(project: &std::path::Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(project)
        .output()
        .context("read run diff for --changed")?;
    if !output.status.success() {
        bail!("read run diff for --changed failed")
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(PathBuf::from)
        .collect())
}

fn harness() -> Result<()> {
    match env::args().nth(2).as_deref() {
        Some("lint") => harness_lint(),
        Some(command) => bail!("unknown harness command: {command}"),
        None => bail!("missing harness command"),
    }
}

fn harness_lint() -> Result<()> {
    let registry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let definitions = load_from_directory(&registry)
        .with_context(|| format!("failed to lint harness registry `{}`", registry.display()))?;
    println!(
        "{} harness definitions passed validation",
        definitions.len()
    );
    Ok(())
}

fn backup() -> Result<()> {
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL is required for locus backup")?;
    let artifact_root = env_path("LOCUS_ARTIFACT_ROOT", DEFAULT_ARTIFACT_ROOT);
    let backup_root = env_path("LOCUS_BACKUP_ROOT", DEFAULT_BACKUP_ROOT);
    let process = SystemProcessRunner;
    let filesystem = SystemBackupFilesystem;

    Backup::new(&process, &filesystem).create_retained(&RetainedBackupConfig::new(
        database_url,
        artifact_root,
        &backup_root,
    ))?;
    println!("{}", backup_root.display());
    Ok(())
}

fn env_path(variable: &str, default: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}

#[cfg(test)]
mod handoff {
    use super::sock;
    use locus_core::runtime::daemon::AgentSocketVerb;

    #[test]
    fn ends_session() {
        let command = [
            "handoff".into(),
            "auditor".into(),
            "--why".into(),
            "context".into(),
            "exhausted".into(),
        ];
        let (dispatch, args) = sock::allowed_verb(&command).expect("handoff is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::Handoff);
        sock::validate_handoff_args(args).expect("handoff requires a reason");
    }
}

#[cfg(test)]
mod task {
    use super::sock;
    use locus_core::runtime::daemon::AgentSocketVerb;

    #[test]
    fn verbs() {
        let cases = [
            (vec!["task", "list"], AgentSocketVerb::TaskList),
            (vec!["task", "show", "task-1"], AgentSocketVerb::TaskShow),
            (
                vec!["task", "move", "task-1", "in_progress"],
                AgentSocketVerb::TaskMove,
            ),
            (
                vec!["task", "assign", "task-1", "builder"],
                AgentSocketVerb::TaskAssign,
            ),
            (
                vec!["task", "comment", "task-1", "blocked", "on", "approval"],
                AgentSocketVerb::TaskComment,
            ),
        ];
        for (command, expected) in cases {
            let command = command.into_iter().map(String::from).collect::<Vec<_>>();
            let (dispatch, args) = sock::allowed_verb(&command).expect("task verb allowlisted");
            assert_eq!(dispatch.verb, expected);
            sock::validate_task_args(dispatch.verb, args).expect("task args valid");
        }
    }
}

#[cfg(test)]
mod wiki {
    use super::sock;
    use locus_core::runtime::daemon::AgentSocketVerb;

    fn check(command: &[&str], expected: AgentSocketVerb) {
        let command = command
            .iter()
            .map(|value| (*value).into())
            .collect::<Vec<String>>();
        let (dispatch, args) = sock::allowed_verb(&command).expect("wiki verb allowlisted");
        assert_eq!(dispatch.verb, expected);
        sock::validate_wiki_args(dispatch.verb, args).expect("wiki arguments valid");
    }

    #[test]
    fn ingest() {
        check(
            &["wiki", "ingest", "README.md"],
            AgentSocketVerb::WikiIngest,
        );
    }

    #[test]
    fn lint() {
        check(&["wiki", "lint"], AgentSocketVerb::WikiLint);
    }

    #[test]
    fn verbs() {
        check(&["wiki", "search", "daemon"], AgentSocketVerb::WikiSearch);
        check(&["wiki", "read", "daemon"], AgentSocketVerb::WikiRead);
        check(
            &["wiki", "write", "daemon", "body"],
            AgentSocketVerb::WikiWrite,
        );
        check(&["wiki", "history", "daemon"], AgentSocketVerb::WikiHistory);
    }

    #[test]
    fn query_files_synthesis() {
        check(
            &["wiki", "query", "how", "does", "isolation", "work"],
            AgentSocketVerb::WikiQuery,
        );
    }
}

#[cfg(test)]
mod ralph {
    use super::ralph;

    #[test]
    fn runs() {
        let output = std::panic::catch_unwind(|| {
            ralph(&[
                "--goal".into(),
                "ship it".into(),
                "--verify".into(),
                "true".into(),
            ])
        })
        .expect("ralph parser does not panic");
        assert!(output.is_ok());
    }
}

#[cfg(test)]
mod artifact {
    use super::sock;
    use locus_core::runtime::daemon::AgentSocketVerb;

    #[test]
    fn put() {
        let command = [
            "artifact".into(),
            "put".into(),
            "plan".into(),
            "plan.md".into(),
        ];
        let (dispatch, args) = sock::allowed_verb(&command)
            .unwrap_or_else(|error| panic!("artifact put dispatches: {error}"));
        assert_eq!(dispatch.verb, AgentSocketVerb::ArtifactPut);
        assert_eq!(args, ["plan", "plan.md"]);
    }

    #[test]
    fn get_roundtrip() {
        let command = ["artifact".into(), "get".into(), "artifact-id".into()];
        let (dispatch, args) = sock::allowed_verb(&command)
            .unwrap_or_else(|error| panic!("artifact get dispatches: {error}"));
        assert_eq!(dispatch.verb, AgentSocketVerb::ArtifactGet);
        assert_eq!(args, ["artifact-id"]);
    }

    #[test]
    fn comments() {
        let command = ["artifact".into(), "comments".into()];
        let (dispatch, args) = sock::allowed_verb(&command)
            .unwrap_or_else(|error| panic!("artifact comments dispatches: {error}"));
        assert_eq!(dispatch.verb, AgentSocketVerb::ArtifactComments);
        assert!(args.is_empty());
    }

    #[test]
    fn for_context() {
        let command = [
            "artifact".into(),
            "get".into(),
            "artifact-id".into(),
            "--for-context".into(),
        ];
        let (dispatch, args) = sock::allowed_verb(&command).expect("artifact context route");
        assert_eq!(dispatch.verb, AgentSocketVerb::ArtifactGet);
        assert_eq!(args, ["artifact-id", "--for-context"]);
    }
}

#[cfg(test)]
mod tools {
    use super::sock;
    use locus_core::runtime::daemon::AgentSocketVerb;

    #[test]
    fn list() {
        let command = vec!["tools".into(), "list".into()];
        let (dispatch, args) = sock::allowed_verb(&command).expect("tools list is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::ToolsList);
        assert!(args.is_empty());
    }

    #[test]
    fn docs() {
        let command = vec!["tools".into(), "docs".into(), "rg".into()];
        let (dispatch, args) = sock::allowed_verb(&command).expect("tools docs is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::ToolsDocs);
        assert_eq!(args, ["rg"]);
    }
}

#[cfg(test)]
mod lint {
    use super::*;
    use locus_core::runtime::daemon::AgentSocketVerb;

    #[test]
    fn runs_all() {
        let command = vec!["lint".into()];
        assert_eq!(
            sock::allowed_verb(&command)
                .expect("lint is allowlisted")
                .0
                .verb,
            AgentSocketVerb::Lint
        );
    }

    #[test]
    fn only() {
        let arguments: Vec<String> = vec!["--only".into(), "format".into()];
        assert_eq!(arguments[1], "format");
    }

    #[test]
    fn changed() {
        let request = LintRequest {
            changed: true,
            ..LintRequest::default()
        };
        assert!(request.changed);
    }

    #[test]
    fn exit_code() {
        let report = locus_core::services::lint::LintReport::default();
        assert!(verify_linters(&report).is_ok());
    }

    #[test]
    fn prints_the_rule() {
        let report = locus_core::services::lint::LintReport::default();
        assert!(report.evidence().is_empty());
    }
}

#[cfg(test)]
mod browse {
    use super::*;
    use locus_core::runtime::daemon::AgentSocketVerb;
    use locus_core::services::browse::{
        assert_page, parse_assert_args, AssertOptions, Element, Page,
    };

    #[test]
    fn open_waits_for_ready() {
        let command = ["browse".into(), "open".into(), "/".into()];
        assert_eq!(
            sock::allowed_verb(&command).unwrap().0.verb,
            AgentSocketVerb::BrowseOpen
        );
    }

    #[test]
    fn open() {
        let command = ["browse".into(), "open".into(), "/settings".into()];
        assert_eq!(sock::allowed_verb(&command).unwrap().1, ["/settings"]);
    }

    #[test]
    fn interactions() {
        for command in [
            vec!["browse", "click", "#save"],
            vec!["browse", "fill", "#name", "Locus"],
            vec!["browse", "press", "#name", "Enter"],
        ] {
            let command = command.into_iter().map(String::from).collect::<Vec<_>>();
            assert!(sock::allowed_verb(&command).is_ok());
        }
    }

    #[test]
    fn assert() {
        let args = ["#save", "--text", "Saved", "--visible", "--count", "1"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let options = parse_assert_args(&args).unwrap();
        let page = Page::default().element(
            "#save",
            Element {
                text: "Saved".into(),
                visible: true,
                count: 1,
            },
        );
        assert!(assert_page(&page, &options).passed);
    }

    #[test]
    fn assert_exit_code() {
        let options = AssertOptions {
            selector: "#missing".into(),
            ..Default::default()
        };
        let result = assert_page(&Page::default(), &options);
        assert!(!result.passed);
        assert!(assertion_json(&result).get("failure").is_some());
    }

    #[test]
    fn screenshot() {
        assert_eq!(
            sock::allowed_verb(&["browse".into(), "screenshot".into()])
                .unwrap()
                .0
                .verb,
            AgentSocketVerb::BrowseScreenshot
        );
    }

    #[test]
    fn record() {
        assert_eq!(
            sock::allowed_verb(&["browse".into(), "record".into(), "start".into()])
                .unwrap()
                .0
                .verb,
            AgentSocketVerb::BrowseRecord
        );
    }

    #[test]
    fn console_network() {
        for name in ["console", "network"] {
            assert!(sock::allowed_verb(&["browse".into(), name.into()]).is_ok());
        }
    }
}
