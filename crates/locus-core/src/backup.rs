//! Creates one portable backup artifact containing Locus's durable data.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};

/// The Postgres schemas that contain Locus-owned state.
pub const BACKUP_SCHEMAS: [&str; 8] = [
    "core",
    "agents",
    "board",
    "wiki",
    "memory",
    "workflows",
    "mail",
    "market",
];

/// Parameters for one backup artifact.
pub struct BackupConfig {
    pub database_url: String,
    pub artifact_root: PathBuf,
    pub destination: PathBuf,
}

impl BackupConfig {
    pub fn new(
        database_url: impl Into<String>,
        artifact_root: impl Into<PathBuf>,
        destination: impl Into<PathBuf>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            artifact_root: artifact_root.into(),
            destination: destination.into(),
        }
    }
}

/// Parameters for daily and weekly backup artifacts managed by retention.
pub struct RetainedBackupConfig {
    pub database_url: String,
    pub artifact_root: PathBuf,
    pub backup_root: PathBuf,
}

impl RetainedBackupConfig {
    pub fn new(
        database_url: impl Into<String>,
        artifact_root: impl Into<PathBuf>,
        backup_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            artifact_root: artifact_root.into(),
            backup_root: backup_root.into(),
        }
    }
}

/// Supplies time so retention artifact names are deterministic in tests.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

/// Production clock for retained backups.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

static SYSTEM_CLOCK: SystemClock = SystemClock;

/// Runs an external command without coupling backup tests to a host binary.
pub trait ProcessRunner: Send + Sync {
    fn run(&self, program: &str, arguments: &[String]) -> Result<Vec<u8>>;
}

/// Production process runner for `pg_dump`.
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, program: &str, arguments: &[String]) -> Result<Vec<u8>> {
        let output = Command::new(program)
            .args(arguments)
            .output()
            .with_context(|| format!("run {program}"))?;
        if !output.status.success() {
            bail!("{program} exited with status {}", output.status);
        }
        Ok(output.stdout)
    }
}

/// Writes the SQL dump and artifact blob tree into one backup artifact.
pub trait BackupFilesystem: Send + Sync {
    fn create_archive(
        &self,
        destination: &Path,
        sql_dump: &[u8],
        artifact_root: &Path,
    ) -> Result<()>;

    fn list_files(&self, directory: &Path) -> Result<Vec<PathBuf>>;

    fn remove_file(&self, path: &Path) -> Result<()>;
}

/// Production tar writer. It publishes only fully written archive files.
pub struct SystemBackupFilesystem;

impl BackupFilesystem for SystemBackupFilesystem {
    fn create_archive(
        &self,
        destination: &Path,
        sql_dump: &[u8],
        artifact_root: &Path,
    ) -> Result<()> {
        if !artifact_root.is_dir() {
            bail!(
                "artifact root is not a directory: {}",
                artifact_root.display()
            );
        }

        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("create backup directory {}", parent.display()))?;
        let temporary = temporary_path(destination)?;

        let result = (|| {
            let file = File::create(&temporary)
                .with_context(|| format!("create backup artifact {}", temporary.display()))?;
            let mut archive = tar::Builder::new(file);
            let mut header = tar::Header::new_gnu();
            header.set_size(sql_dump.len() as u64);
            header.set_mode(0o600);
            header.set_cksum();
            archive
                .append_data(&mut header, "database.sql", sql_dump)
                .context("write SQL dump into backup artifact")?;
            archive
                .append_dir_all("artifacts", artifact_root)
                .context("write artifact blob tree into backup artifact")?;
            archive.finish().context("finish backup artifact")?;
            fs::rename(&temporary, destination).with_context(|| {
                format!(
                    "publish backup artifact from {} to {}",
                    temporary.display(),
                    destination.display()
                )
            })?;
            Ok(())
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    fn list_files(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        fs::read_dir(directory)
            .with_context(|| format!("list backup directory {}", directory.display()))?
            .map(|entry| {
                let entry = entry.context("read backup directory entry")?;
                let file_type = entry.file_type().context("read backup file type")?;
                Ok(file_type.is_file().then(|| entry.path()))
            })
            .filter_map(Result::transpose)
            .collect()
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        fs::remove_file(path).with_context(|| format!("remove expired backup {}", path.display()))
    }
}

fn temporary_path(destination: &Path) -> Result<PathBuf> {
    let file_name = destination
        .file_name()
        .context("backup destination has no file name")?
        .to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("read system time for backup temporary file")?
        .as_nanos();
    Ok(destination.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce)))
}

/// Coordinates the database dump and artifact tree archive through injectable seams.
pub struct Backup<'a> {
    process: &'a dyn ProcessRunner,
    filesystem: &'a dyn BackupFilesystem,
    clock: &'a dyn Clock,
}

impl<'a> Backup<'a> {
    pub fn new(process: &'a dyn ProcessRunner, filesystem: &'a dyn BackupFilesystem) -> Self {
        Self::with_clock(process, filesystem, &SYSTEM_CLOCK)
    }

    pub fn with_clock(
        process: &'a dyn ProcessRunner,
        filesystem: &'a dyn BackupFilesystem,
        clock: &'a dyn Clock,
    ) -> Self {
        Self {
            process,
            filesystem,
            clock,
        }
    }

    /// Produces one tar archive containing `database.sql` and `artifacts/`.
    pub fn create(&self, config: &BackupConfig) -> Result<()> {
        let sql_dump = self.dump(&config.database_url)?;
        self.filesystem
            .create_archive(&config.destination, &sql_dump, &config.artifact_root)
    }

    /// Produces the current daily and weekly artifacts, then removes expired ones.
    pub fn create_retained(&self, config: &RetainedBackupConfig) -> Result<()> {
        let index = backup_index(self.clock.now())?;
        let daily = config.backup_root.join(format!("daily-{index:010}.tar"));
        let weekly = config
            .backup_root
            .join(format!("weekly-{:010}.tar", (index + 3) / 7));
        let sql_dump = self.dump(&config.database_url)?;

        self.filesystem
            .create_archive(&daily, &sql_dump, &config.artifact_root)?;
        self.filesystem
            .create_archive(&weekly, &sql_dump, &config.artifact_root)?;
        self.retain(&config.backup_root)
    }

    /// Retains the seven newest daily and four newest weekly artifacts.
    pub fn retain(&self, backup_root: &Path) -> Result<()> {
        let files = self.filesystem.list_files(backup_root)?;
        for (prefix, count) in [("daily-", 7), ("weekly-", 4)] {
            let mut artifacts = files
                .iter()
                .filter_map(|path| backup_artifact_index(path, prefix).map(|index| (index, path)))
                .collect::<Vec<_>>();
            artifacts.sort_unstable_by(|(left, _), (right, _)| right.cmp(left));
            for (_, path) in artifacts.into_iter().skip(count) {
                self.filesystem.remove_file(path)?;
            }
        }
        Ok(())
    }

    fn dump(&self, database_url: &str) -> Result<Vec<u8>> {
        let mut arguments = vec![
            "--format=plain".to_owned(),
            "--no-owner".to_owned(),
            "--no-privileges".to_owned(),
        ];
        arguments.extend(BACKUP_SCHEMAS.map(|schema| format!("--schema={schema}")));
        arguments.push(format!("--dbname={database_url}"));
        self.process.run("pg_dump", &arguments)
    }
}

fn backup_index(time: SystemTime) -> Result<u64> {
    Ok(time
        .duration_since(UNIX_EPOCH)
        .context("read backup clock")?
        .as_secs()
        / 86_400)
}

fn backup_artifact_index(path: &Path, prefix: &str) -> Option<u64> {
    path.file_name()?
        .to_str()?
        .strip_prefix(prefix)?
        .strip_suffix(".tar")?
        .parse()
        .ok()
}

#[cfg(test)]
mod covers_both_trees {
    use std::{
        fs::{self, File},
        io::Read,
        path::{Path, PathBuf},
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    use anyhow::Result;

    use super::{
        Backup, BackupConfig, BackupFilesystem, ProcessRunner, SystemBackupFilesystem,
        BACKUP_SCHEMAS,
    };

    struct FakeProcess {
        invocations: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl ProcessRunner for FakeProcess {
        fn run(&self, program: &str, arguments: &[String]) -> Result<Vec<u8>> {
            self.invocations
                .lock()
                .expect("record pg_dump invocation")
                .push((program.to_owned(), arguments.to_vec()));
            Ok(b"-- PostgreSQL database dump\n".to_vec())
        }
    }

    struct CapturingFilesystem {
        archives: Mutex<Vec<(PathBuf, Vec<u8>, PathBuf)>>,
    }

    impl BackupFilesystem for CapturingFilesystem {
        fn create_archive(
            &self,
            destination: &Path,
            sql_dump: &[u8],
            artifact_root: &Path,
        ) -> Result<()> {
            self.archives.lock().expect("record backup archive").push((
                destination.to_owned(),
                sql_dump.to_vec(),
                artifact_root.to_owned(),
            ));
            Ok(())
        }

        fn list_files(&self, _: &Path) -> Result<Vec<PathBuf>> {
            Ok(Vec::new())
        }

        fn remove_file(&self, _: &Path) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn covers_both_trees() {
        let process = FakeProcess {
            invocations: Mutex::new(Vec::new()),
        };
        let filesystem = CapturingFilesystem {
            archives: Mutex::new(Vec::new()),
        };
        let backup = Backup::new(&process, &filesystem);
        let config = BackupConfig::new(
            "postgres://locus:secret@localhost/locus",
            "/var/lib/locus/artifacts",
            "/var/lib/locus/backups/backup.tar",
        );

        backup.create(&config).expect("create a backup");

        let invocations = process.invocations.lock().expect("read pg_dump invocation");
        assert_eq!(invocations.len(), 1);
        assert_eq!(invocations[0].0, "pg_dump");
        for schema in BACKUP_SCHEMAS {
            assert!(
                invocations[0].1.contains(&format!("--schema={schema}")),
                "pg_dump includes the {schema} schema"
            );
        }

        let archives = filesystem.archives.lock().expect("read backup archive");
        assert_eq!(archives.len(), 1);
        assert_eq!(
            archives[0].0,
            Path::new("/var/lib/locus/backups/backup.tar")
        );
        assert_eq!(archives[0].1, b"-- PostgreSQL database dump\n");
        assert_eq!(archives[0].2, Path::new("/var/lib/locus/artifacts"));
    }

    #[test]
    fn writes_one_artifact_with_sql_and_blobs() {
        let root = std::env::temp_dir().join(format!(
            "locus-backup-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("read test clock")
                .as_nanos()
        ));
        let artifact_root = root.join("artifacts");
        fs::create_dir_all(artifact_root.join("project/run"))
            .expect("create nested artifact directory");
        fs::write(artifact_root.join("project/run/screenshot.webp"), b"blob")
            .expect("write artifact blob");

        let process = FakeProcess {
            invocations: Mutex::new(Vec::new()),
        };
        let filesystem = SystemBackupFilesystem;
        let destination = root.join("backup.tar");
        let backup = Backup::new(&process, &filesystem);
        backup
            .create(&BackupConfig::new(
                "postgres://locus@localhost/locus",
                &artifact_root,
                &destination,
            ))
            .expect("write backup artifact");

        let mut paths = Vec::new();
        let mut sql_dump = String::new();
        let mut archive =
            tar::Archive::new(File::open(&destination).expect("open backup artifact"));
        for entry in archive.entries().expect("read backup entries") {
            let mut entry = entry.expect("read backup entry");
            let path = entry
                .path()
                .expect("read backup entry path")
                .to_string_lossy()
                .into_owned();
            if path == "database.sql" {
                entry
                    .read_to_string(&mut sql_dump)
                    .expect("read SQL dump from backup");
            }
            paths.push(path);
        }

        assert_eq!(sql_dump, "-- PostgreSQL database dump\n");
        assert!(paths.contains(&"artifacts/project/run/screenshot.webp".to_owned()));
        fs::remove_dir_all(root).expect("remove test backup directory");
    }
}

#[cfg(test)]
mod retention {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use anyhow::Result;

    use super::{Backup, BackupFilesystem, Clock, ProcessRunner, RetainedBackupConfig};

    struct FakeProcess;

    impl ProcessRunner for FakeProcess {
        fn run(&self, _: &str, _: &[String]) -> Result<Vec<u8>> {
            Ok(b"-- PostgreSQL database dump\n".to_vec())
        }
    }

    struct FixedClock(SystemTime);

    impl Clock for FixedClock {
        fn now(&self) -> SystemTime {
            self.0
        }
    }

    struct RetainingFilesystem {
        files: Mutex<Vec<PathBuf>>,
        archives: Mutex<Vec<PathBuf>>,
        removed: Mutex<Vec<PathBuf>>,
    }

    impl BackupFilesystem for RetainingFilesystem {
        fn create_archive(&self, destination: &Path, _: &[u8], _: &Path) -> Result<()> {
            self.archives
                .lock()
                .expect("record retained archive")
                .push(destination.to_owned());
            let mut files = self.files.lock().expect("record retained backup");
            files.retain(|path| path != destination);
            files.push(destination.to_owned());
            Ok(())
        }

        fn list_files(&self, _: &Path) -> Result<Vec<PathBuf>> {
            Ok(self.files.lock().expect("list backups").clone())
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            self.files
                .lock()
                .expect("remove retained backup")
                .retain(|candidate| candidate != path);
            self.removed
                .lock()
                .expect("record removed backup")
                .push(path.to_owned());
            Ok(())
        }
    }

    #[test]
    fn retention() {
        let root = PathBuf::from("/var/lib/locus/backups");
        let mut files = (1..62)
            .map(|day| root.join(format!("daily-{day:010}.tar")))
            .collect::<Vec<_>>();
        files.extend((1..9).map(|week| root.join(format!("weekly-{week:010}.tar"))));
        files.push(root.join("operator-copy.tar"));
        let filesystem = RetainingFilesystem {
            files: Mutex::new(files),
            archives: Mutex::new(Vec::new()),
            removed: Mutex::new(Vec::new()),
        };
        let process = FakeProcess;
        let clock = FixedClock(UNIX_EPOCH + Duration::from_secs(62 * 86_400));
        let backup = Backup::with_clock(&process, &filesystem, &clock);

        backup
            .create_retained(&RetainedBackupConfig::new(
                "postgres://locus@localhost/locus",
                "/var/lib/locus/artifacts",
                &root,
            ))
            .expect("create retained backup");

        assert_eq!(
            *filesystem.archives.lock().expect("read retained archives"),
            vec![
                root.join("daily-0000000062.tar"),
                root.join("weekly-0000000009.tar"),
            ]
        );
        let removed = filesystem.removed.lock().expect("read removed backups");
        assert_eq!(
            removed
                .iter()
                .filter(|path| path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("daily-"))
                .count(),
            55
        );
        assert_eq!(
            removed
                .iter()
                .filter(|path| path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("weekly-"))
                .count(),
            5
        );
        assert!(!removed.contains(&root.join("operator-copy.tar")));
        let remaining = filesystem.files.lock().expect("read retained backups");
        assert_eq!(
            remaining
                .iter()
                .filter(|path| path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("daily-"))
                .count(),
            7
        );
        assert_eq!(
            remaining
                .iter()
                .filter(|path| path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("weekly-"))
                .count(),
            4
        );
    }
}
