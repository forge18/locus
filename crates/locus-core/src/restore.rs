//! Restores the database SQL from a backup archive into an isolated scratch database.

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};

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

/// Production process runner for `psql`.
pub struct SystemRestoreProcessRunner;

impl RestoreProcessRunner for SystemRestoreProcessRunner {
    fn run(&self, program: &str, arguments: &[String], standard_input: &[u8]) -> Result<()> {
        let mut child = Command::new(program)
            .args(arguments)
            .stdin(Stdio::piped())
            .spawn()
            .with_context(|| format!("run {program}"))?;
        child
            .stdin
            .as_mut()
            .context("open psql standard input")?
            .write_all(standard_input)
            .context("write SQL dump to psql")?;
        let status = child
            .wait()
            .with_context(|| format!("wait for {program}"))?;
        if !status.success() {
            bail!("{program} exited with status {status}");
        }
        Ok(())
    }
}

/// Reads the database SQL member of a backup archive without coupling tests to tar files.
pub trait RestoreFilesystem: Send + Sync {
    fn read_database_dump(&self, archive: &Path) -> Result<Vec<u8>>;
}

/// Production tar reader for the SQL dump in a backup archive.
pub struct SystemRestoreFilesystem;

impl RestoreFilesystem for SystemRestoreFilesystem {
    fn read_database_dump(&self, archive: &Path) -> Result<Vec<u8>> {
        let file = File::open(archive)
            .with_context(|| format!("open backup archive {}", archive.display()))?;
        let mut archive = tar::Archive::new(file);
        let mut sql_dump = None;

        for entry in archive.entries().context("read backup archive entries")? {
            let mut entry = entry.context("read backup archive entry")?;
            if entry.path().context("read backup archive entry path")? == Path::new("database.sql")
            {
                if sql_dump.is_some() {
                    bail!("backup archive contains multiple database.sql entries");
                }
                let mut contents = Vec::new();
                entry
                    .read_to_end(&mut contents)
                    .context("read database SQL from backup archive")?;
                sql_dump = Some(contents);
            }
        }

        sql_dump.context("backup archive does not contain database.sql")
    }
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
    pub fn into_scratch(&self, config: &RestoreConfig) -> Result<()> {
        let sql_dump = self.filesystem.read_database_dump(&config.archive)?;
        self.process.run(
            "psql",
            &[
                format!("--dbname={}", config.scratch_database_url),
                "--single-transaction".to_owned(),
                "--set=ON_ERROR_STOP=1".to_owned(),
                "--file=-".to_owned(),
            ],
            &sql_dump,
        )
    }
}

#[cfg(test)]
mod into_scratch {
    use std::{
        fs::{self, File},
        path::{Path, PathBuf},
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    use anyhow::Result;

    use super::{Restore, RestoreConfig, RestoreFilesystem, RestoreProcessRunner};

    type Invocation = (String, Vec<String>, Vec<u8>);

    struct CapturingProcess {
        invocations: Mutex<Vec<Invocation>>,
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
    fn reads_database_sql_from_archive() {
        let root = std::env::temp_dir().join(format!(
            "locus-restore-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("read test clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create test directory");
        let archive_path = root.join("backup.tar");
        let mut archive = tar::Builder::new(File::create(&archive_path).expect("create archive"));
        let mut header = tar::Header::new_gnu();
        header.set_size(b"CREATE SCHEMA core;\n".len() as u64);
        header.set_mode(0o600);
        header.set_cksum();
        archive
            .append_data(&mut header, "database.sql", &b"CREATE SCHEMA core;\n"[..])
            .expect("write SQL dump");
        archive.finish().expect("finish archive");

        let filesystem = super::SystemRestoreFilesystem;
        assert_eq!(
            filesystem
                .read_database_dump(&archive_path)
                .expect("read database SQL"),
            b"CREATE SCHEMA core;\n"
        );

        fs::remove_dir_all(root).expect("remove test directory");
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
