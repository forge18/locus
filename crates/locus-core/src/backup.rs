#[cfg(test)]
mod covers_both_trees {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use anyhow::Result;

    use super::{Backup, BackupConfig, BackupFilesystem, ProcessRunner, BACKUP_SCHEMAS};

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
            self.archives
                .lock()
                .expect("record backup archive")
                .push((
                    destination.to_owned(),
                    sql_dump.to_vec(),
                    artifact_root.to_owned(),
                ));
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
                invocations[0]
                    .1
                    .contains(&format!("--schema={schema}")),
                "pg_dump includes the {schema} schema"
            );
        }

        let archives = filesystem.archives.lock().expect("read backup archive");
        assert_eq!(archives.len(), 1);
        assert_eq!(archives[0].0, Path::new("/var/lib/locus/backups/backup.tar"));
        assert_eq!(archives[0].1, b"-- PostgreSQL database dump\n");
        assert_eq!(archives[0].2, Path::new("/var/lib/locus/artifacts"));
    }
}
