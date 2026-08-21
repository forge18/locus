//! Restores the database SQL from a backup archive into an isolated scratch database.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

/// Parameters for restoring one backup archive into a scratch database.
pub struct RestoreConfig {
    pub archive: PathBuf,
    pub scratch_database_url: String,
}

impl RestoreConfig {
    pub fn new(archive: impl Into<PathBuf>, scratch_database_url: impl Into<String>) -> Self {
        Self {
            archive: archive.into(),
            scratch_database_url: scratch_database_url.into(),
        }
    }
}

/// Runs a restore command without coupling restore tests to a host binary.
pub trait RestoreProcessRunner: Send + Sync {
    fn run(&self, program: &str, arguments: &[String], standard_input: &[u8]) -> Result<()>;
}

/// Reads the database SQL member of a backup archive without coupling tests to tar files.
pub trait RestoreFilesystem: Send + Sync {
    fn read_database_dump(&self, archive: &Path) -> Result<Vec<u8>>;
}

/// Coordinates archive reads and restoring into the supplied scratch database only.
pub struct Restore<'a> {
    process: &'a dyn RestoreProcessRunner,
    filesystem: &'a dyn RestoreFilesystem,
}

impl<'a> Restore<'a> {
    pub fn new(
        process: &'a dyn RestoreProcessRunner,
        filesystem: &'a dyn RestoreFilesystem,
    ) -> Self {
        Self {
            process,
            filesystem,
        }
    }

    /// Restores `database.sql` into the configured scratch database.
    pub fn into_scratch(&self, _: &RestoreConfig) -> Result<()> {
        bail!("restore into scratch is not implemented")
    }
}

#[cfg(test)]
mod into_scratch {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use anyhow::Result;

    use super::{Restore, RestoreConfig, RestoreFilesystem, RestoreProcessRunner};

    struct CapturingProcess {
        invocations: Mutex<Vec<(String, Vec<String>, Vec<u8>)>>,
    }

    impl RestoreProcessRunner for CapturingProcess {
        fn run(&self, program: &str, arguments: &[String], standard_input: &[u8]) -> Result<()> {
            self.invocations
                .lock()
                .expect("record restore invocation")
                .push((
                    program.to_owned(),
                    arguments.to_vec(),
                    standard_input.to_vec(),
                ));
            Ok(())
        }
    }

    struct ArchiveFilesystem;

    impl RestoreFilesystem for ArchiveFilesystem {
        fn read_database_dump(&self, archive: &Path) -> Result<Vec<u8>> {
            assert_eq!(archive, Path::new("/var/lib/locus/backups/backup.tar"));
            Ok(b"CREATE SCHEMA core;\n".to_vec())
        }
    }

    #[test]
    fn into_scratch() {
        let process = CapturingProcess {
            invocations: Mutex::new(Vec::new()),
        };
        let filesystem = ArchiveFilesystem;
        let restore = Restore::new(&process, &filesystem);
        let scratch_database_url = "postgres://locus@localhost/locus_restore_drill";

        restore
            .into_scratch(&RestoreConfig::new(
                PathBuf::from("/var/lib/locus/backups/backup.tar"),
                scratch_database_url,
            ))
            .expect("restore into the scratch database");

        assert_eq!(
            *process.invocations.lock().expect("read restore invocation"),
            vec![(
                "psql".to_owned(),
                vec![
                    format!("--dbname={scratch_database_url}"),
                    "--single-transaction".to_owned(),
                    "--set=ON_ERROR_STOP=1".to_owned(),
                    "--file=-".to_owned(),
                ],
                b"CREATE SCHEMA core;\n".to_vec(),
            )]
        );
    }
}
