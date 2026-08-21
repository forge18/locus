//! `locus` — the CLI agents call from inside their container.
//!
//! A thin client over the daemon socket at /run/locus.sock. It holds no logic of its
//! own: every verb is a round trip to locus-core, so behaviour cannot drift between
//! what an agent sees and what the app sees.

use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};
use locus_core::backup::{
    Backup, RetainedBackupConfig, SystemBackupFilesystem, SystemProcessRunner,
};

const DEFAULT_ARTIFACT_ROOT: &str = "/var/lib/locus/artifacts";
const DEFAULT_BACKUP_ROOT: &str = "/var/lib/locus/backups";

fn main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        None => {
            println!("locus {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("backup") => backup(),
        Some(command) => bail!("unknown command: {command}"),
    }
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
