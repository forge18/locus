//! `locus` — the CLI agents call from inside their container.
//!
//! A thin client over the daemon socket at /run/locus.sock. It holds no logic of its
//! own: every verb is a round trip to locus-core, so behaviour cannot drift between
//! what an agent sees and what the app sees.

use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};
use locus_core::{
    backup::{Backup, RetainedBackupConfig, SystemBackupFilesystem, SystemProcessRunner},
    registry::load_from_directory,
};

mod hook;
pub mod sock;

const DEFAULT_ARTIFACT_ROOT: &str = "/var/lib/locus/artifacts";
const DEFAULT_BACKUP_ROOT: &str = "/var/lib/locus/backups";

fn main() -> Result<()> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        None => {
            println!("locus {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("backup") => backup(),
        Some("harness") => harness(),
        Some("hook") => {
            let _ = hook::run();
            Ok(())
        }
        Some(_) => dispatch(&arguments),
    }
}

fn dispatch(arguments: &[String]) -> Result<()> {
    let (verb, args) = sock::resolve_verb(arguments)
        .ok_or_else(|| anyhow::anyhow!("unknown command: {}", arguments.join(" ")))?;
    let runtime = tokio::runtime::Runtime::new().context("start socket client runtime")?;
    let response = runtime.block_on(sock::dispatch(sock::DEFAULT_SOCKET_PATH, verb, args))?;
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
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
