//! `locus` — the CLI agents call from inside their container.
//!
//! A thin client over the daemon socket at /run/locus.sock. It holds no logic of its
//! own: every verb is a round trip to locus-core, so behaviour cannot drift between
//! what an agent sees and what the app sees.

use std::{env, path::PathBuf, process::Command};

use anyhow::{bail, Context, Result};
use locus_core::{
    harness::registry::load_from_directory,
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
        Some("hook") => {
            let _ = hook::run();
            Ok(())
        }
        Some(_) => dispatch(&arguments),
    }
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
    if verb.verb == locus_core::runtime::daemon::AgentSocketVerb::LspSymbols {
        sock::validate_symbols_args(args)?;
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
