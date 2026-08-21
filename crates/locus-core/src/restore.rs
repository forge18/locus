//! Restores the database SQL from a backup archive into an isolated scratch database.

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};

use crate::backup::BACKUP_SCHEMAS;

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

/// Queries the exact row count across the schemas stored in a backup artifact.
pub trait RestoreRowCountQuery: Send + Sync {
    fn row_count(&self, database_url: &str) -> Result<u64>;
}

/// Production row-count query runner for `psql`.
pub struct SystemRestoreRowCountQuery;

impl RestoreRowCountQuery for SystemRestoreRowCountQuery {
    fn row_count(&self, database_url: &str) -> Result<u64> {
        let output = Command::new("psql")
            .args([
                format!("--dbname={database_url}"),
                "--no-align".to_owned(),
                "--quiet".to_owned(),
                "--tuples-only".to_owned(),
                format!("--command={}", row_count_sql()),
            ])
            .output()
            .context("run psql row-count query")?;
        if !output.status.success() {
            bail!("psql row-count query exited with status {}", output.status);
        }
        std::str::from_utf8(&output.stdout)
            .context("read psql row-count output")?
            .trim()
            .parse()
            .context("parse psql row-count output")
    }
}

fn row_count_sql() -> String {
    let schemas = BACKUP_SCHEMAS
        .iter()
        .map(|schema| format!("'{schema}'"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "CREATE FUNCTION pg_temp.locus_restore_row_count() RETURNS bigint LANGUAGE plpgsql AS $$ \
         DECLARE total bigint := 0; relation record; table_count bigint; \
         BEGIN FOR relation IN SELECT schemaname, tablename FROM pg_tables WHERE schemaname IN ({schemas}) LOOP \
         EXECUTE format('SELECT count(*) FROM %I.%I', relation.schemaname, relation.tablename) INTO table_count; \
         total := total + table_count; END LOOP; RETURN total; END; $$; \
         SELECT pg_temp.locus_restore_row_count();"
    )
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
    row_count_query: &'a dyn RestoreRowCountQuery,
}

static SYSTEM_RESTORE_ROW_COUNT_QUERY: SystemRestoreRowCountQuery = SystemRestoreRowCountQuery;

impl<'a> Restore<'a> {
    pub fn new(
        process: &'a dyn RestoreProcessRunner,
        filesystem: &'a dyn RestoreFilesystem,
    ) -> Self {
        Self::with_row_count_query(process, filesystem, &SYSTEM_RESTORE_ROW_COUNT_QUERY)
    }

    /// Injects the row-count query so restore drills can be tested without a database.
    pub fn with_row_count_query(
        process: &'a dyn RestoreProcessRunner,
        filesystem: &'a dyn RestoreFilesystem,
        row_count_query: &'a dyn RestoreRowCountQuery,
    ) -> Self {
        Self {
            process,
            filesystem,
            row_count_query,
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

    /// Restores into scratch and verifies its rows match the source database.
    pub fn drill(&self, config: &RestoreConfig, source_database_url: &str) -> Result<()> {
        self.into_scratch(config)?;
        let source_count = self.row_count_query.row_count(source_database_url)?;
        let scratch_count = self
            .row_count_query
            .row_count(&config.scratch_database_url)?;
        if source_count != scratch_count {
            bail!(
                "restore drill row-count mismatch: source has {source_count} rows, scratch has {scratch_count} rows"
            );
        }
        Ok(())
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

#[cfg(test)]
mod drill_asserts_counts {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use anyhow::Result;

    use super::{
        Restore, RestoreConfig, RestoreFilesystem, RestoreProcessRunner, RestoreRowCountQuery,
    };

    struct NoopProcess;

    impl RestoreProcessRunner for NoopProcess {
        fn run(&self, _: &str, _: &[String], _: &[u8]) -> Result<()> {
            Ok(())
        }
    }

    struct DumpFilesystem;

    impl RestoreFilesystem for DumpFilesystem {
        fn read_database_dump(&self, _: &Path) -> Result<Vec<u8>> {
            Ok(b"CREATE SCHEMA core;\n".to_vec())
        }
    }

    struct FixedRowCountQuery {
        source_database_url: String,
        source_count: u64,
        scratch_database_url: String,
        scratch_count: u64,
        invocations: Mutex<Vec<String>>,
    }

    impl RestoreRowCountQuery for FixedRowCountQuery {
        fn row_count(&self, database_url: &str) -> Result<u64> {
            self.invocations
                .lock()
                .expect("record row-count query")
                .push(database_url.to_owned());
            Ok(if database_url == self.source_database_url {
                self.source_count
            } else {
                assert_eq!(database_url, self.scratch_database_url);
                self.scratch_count
            })
        }
    }

    #[test]
    fn drill_asserts_counts() {
        let process = NoopProcess;
        let filesystem = DumpFilesystem;
        let source_database_url = "postgres://locus@localhost/locus";
        let scratch_database_url = "postgres://locus@localhost/locus_restore_drill";
        let row_count_query = FixedRowCountQuery {
            source_database_url: source_database_url.to_owned(),
            source_count: 3,
            scratch_database_url: scratch_database_url.to_owned(),
            scratch_count: 3,
            invocations: Mutex::new(Vec::new()),
        };
        let restore = Restore::with_row_count_query(&process, &filesystem, &row_count_query);

        restore
            .drill(
                &RestoreConfig::new(
                    PathBuf::from("/var/lib/locus/backups/backup.tar"),
                    scratch_database_url,
                ),
                source_database_url,
            )
            .expect("restore drill verifies source and scratch row counts");

        assert_eq!(
            *row_count_query
                .invocations
                .lock()
                .expect("read row-count queries"),
            vec![
                source_database_url.to_owned(),
                scratch_database_url.to_owned(),
            ]
        );
    }

    #[test]
    fn names_row_count_mismatch() {
        let process = NoopProcess;
        let filesystem = DumpFilesystem;
        let row_count_query = FixedRowCountQuery {
            source_database_url: "postgres://locus@localhost/locus".to_owned(),
            source_count: 3,
            scratch_database_url: "postgres://locus@localhost/locus_restore_drill".to_owned(),
            scratch_count: 2,
            invocations: Mutex::new(Vec::new()),
        };
        let restore = Restore::with_row_count_query(&process, &filesystem, &row_count_query);

        let error = restore
            .drill(
                &RestoreConfig::new(
                    PathBuf::from("/var/lib/locus/backups/backup.tar"),
                    &row_count_query.scratch_database_url,
                ),
                &row_count_query.source_database_url,
            )
            .expect_err("restore drill rejects a row-count mismatch");

        assert_eq!(
            error.to_string(),
            "restore drill row-count mismatch: source has 3 rows, scratch has 2 rows"
        );
    }
}
