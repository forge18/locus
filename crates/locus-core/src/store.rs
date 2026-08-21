#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use super::{PostgresConfig, PostgresContainer};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

    impl Drop for DockerCleanup {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["rm", "--force", &self.container_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("docker")
                .args(["volume", "rm", "--force", &self.volume_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn container_lifecycle() {
        let suffix = format!("{}-{}", std::process::id(), unused_port());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container = PostgresContainer::new(PostgresConfig::for_test(
            container_name.clone(),
            volume_name,
            unused_port(),
        ));

        container.start().await.expect("start the pgvector container");
        assert!(container.is_healthy().await.expect("inspect health"));

        let vector_extension = Command::new("docker")
            .args([
                "exec",
                &container_name,
                "psql",
                "-U",
                "locus",
                "-d",
                "locus",
                "-v",
                "ON_ERROR_STOP=1",
                "-tAc",
                "CREATE EXTENSION IF NOT EXISTS vector; SELECT extname FROM pg_extension WHERE extname = 'vector'",
            ])
            .output()
            .expect("run psql in the container");
        assert!(
            vector_extension.status.success(),
            "pgvector query failed: {}",
            String::from_utf8_lossy(&vector_extension.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&vector_extension.stdout).trim(),
            "vector"
        );

        container.stop().await.expect("stop the container");
        assert!(!container.is_healthy().await.expect("inspect stopped health"));

        container.start().await.expect("restart the container");
        assert!(container.is_healthy().await.expect("inspect restarted health"));
    }
}
