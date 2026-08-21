//! Spawn one configured agent container for a queued run.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::{
    materialize::{materialize, ExtensionSet, MaterializationReport, MaterializedTree, PluginHost},
    registry::HarnessDefinition,
    sandbox::{
        agent_image_tag, agent_mounts, project_network, Mount, PortAllocator, PtyAttachment,
        ToolPin, AGENT_PTY,
    },
    session::{Run, RunStatus},
};

/// Whether the container runtime built the image or reused its existing cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDisposition {
    Built,
    Reused,
}

/// The complete, harness-agnostic request made to the container runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerLaunch {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub entrypoint: String,
    pub environment: Vec<String>,
    pub mounts: Vec<Mount>,
    pub network: String,
}

/// The narrow container boundary required by run spawning.
///
/// The supplied container adapter owns image caching, container creation, and PTY plumbing; this
/// supervisor owns their ordering and the run state transition.
pub trait ContainerRuntime {
    fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition>;
    fn start_container(&mut self, container: &ContainerLaunch) -> Result<()>;
    fn attach_pty(&mut self, container: &str, attachment: PtyAttachment) -> Result<()>;
}

/// Inputs owned by the caller for one queued run.
pub struct SpawnRequest<'a> {
    pub project_id: &'a str,
    pub harness: &'a HarnessDefinition,
    pub extensions: &'a ExtensionSet,
    pub config_root: PathBuf,
    pub socket_source: PathBuf,
    pub base_image_digest: String,
    pub tools: Vec<ToolPin>,
    pub plugin: Option<&'a PluginHost>,
}

/// The started container and the materialized configuration used for its prompt prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnedRun {
    pub container: ContainerLaunch,
    pub config: MaterializedTree,
    pub materialization: MaterializationReport,
    pub image: String,
    pub image_disposition: ImageDisposition,
    pub port: u16,
}

/// Materialize the run configuration, ensure its agent image, then start and attach its PTY.
pub fn spawn(
    run: &mut Run,
    request: SpawnRequest<'_>,
    ports: &PortAllocator,
    runtime: &mut impl ContainerRuntime,
) -> Result<SpawnedRun> {
    if run.status != RunStatus::Queued {
        bail!("only queued runs may be spawned")
    }

    let (config, materialization) = materialize(
        request.harness,
        request.extensions,
        &request.config_root,
        request.plugin,
    )
    .context("materialize run configuration")?;
    config
        .write_to(&request.config_root)
        .context("write run configuration")?;

    let image = agent_image_tag(&request.base_image_digest, &request.tools);
    let image_disposition = runtime
        .build_or_reuse_image(&image)
        .context("build or reuse agent image")?;
    let port = ports.allocate()?;
    let container = ContainerLaunch {
        name: format!("locus-agent-{}", run.id),
        image: image.clone(),
        command: std::iter::once(request.harness.binary.clone())
            .chain(request.harness.launch.argv.iter().cloned())
            .collect(),
        entrypoint: crate::sandbox::entrypoint_setup().into(),
        environment: vec![format!("LOCUS_PORT={port}")],
        mounts: agent_mounts(
            request.socket_source.display().to_string(),
            request.config_root.display().to_string(),
        )
        .to_vec(),
        network: project_network(request.project_id),
    };
    if let Err(error) = runtime.start_container(&container) {
        ports.release(port);
        return Err(error).context("start agent container");
    }
    if let Err(error) = runtime.attach_pty(&container.name, AGENT_PTY) {
        ports.release(port);
        return Err(error).context("attach agent PTY");
    }

    run.status = RunStatus::Running;
    Ok(SpawnedRun {
        container,
        config,
        materialization,
        image,
        image_disposition,
        port,
    })
}

#[cfg(test)]
mod spawns {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::Result;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::{
        materialize::{ExtensionEntry, ExtensionSet},
        registry::load_from_directory,
        sandbox::{
            agent_image_tag, Mount, PortAllocator, PtyAttachment, ToolPin, AGENT_PTY, CONFIG_SOURCE,
        },
        session::{Run, RunStatus},
    };

    #[derive(Default)]
    struct RecordingRuntime {
        calls: Vec<String>,
        started: Option<ContainerLaunch>,
        attached: Option<(String, PtyAttachment)>,
    }

    impl ContainerRuntime for RecordingRuntime {
        fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition> {
            self.calls.push(format!("image:{image}"));
            Ok(ImageDisposition::Built)
        }

        fn start_container(&mut self, container: &ContainerLaunch) -> Result<()> {
            self.calls.push(format!("start:{}", container.name));
            self.started = Some(container.clone());
            Ok(())
        }

        fn attach_pty(&mut self, container: &str, attachment: PtyAttachment) -> Result<()> {
            self.calls.push(format!("pty:{container}"));
            self.attached = Some((container.into(), attachment));
            Ok(())
        }
    }

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("locus-run-spawns-{}", Uuid::new_v4()))
    }

    #[test]
    fn materializes_builds_starts_and_attaches_the_agent_pty() {
        let registry =
            load_from_directory(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"))
                .expect("registry loads");
        let mut extensions = ExtensionSet::default();
        extensions.insert(
            "context",
            vec![ExtensionEntry::new("base.md", json!({}), "base context")],
        );
        let config_root = root();
        let run_id = Uuid::new_v4();
        let mut run = Run {
            id: run_id,
            session_id: Uuid::new_v4(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Queued,
            events: vec![],
            usage: None,
            exit_code: None,
            artifacts: vec![],
        };
        let request = SpawnRequest {
            project_id: "project-1",
            harness: registry.by_name("claude").expect("claude harness"),
            extensions: &extensions,
            config_root: config_root.clone(),
            socket_source: PathBuf::from("/tmp/locus.sock"),
            base_image_digest: "sha256:base".into(),
            tools: vec![ToolPin {
                name: "rg".into(),
                version: "14".into(),
            }],
            plugin: None,
        };
        let mut runtime = RecordingRuntime::default();
        let ports = PortAllocator::default();

        let spawned = spawn(&mut run, request, &ports, &mut runtime).expect("run spawns");

        let image = agent_image_tag(
            "sha256:base",
            &[ToolPin {
                name: "rg".into(),
                version: "14".into(),
            }],
        );
        assert_eq!(run.status, RunStatus::Running);
        assert_eq!(spawned.image, image);
        assert!(spawned.config.file("CLAUDE.md").is_some());
        assert_eq!(
            fs::read_to_string(config_root.join("CLAUDE.md")).unwrap(),
            "base context"
        );
        assert_eq!(
            runtime.calls,
            [
                format!("image:{image}"),
                format!("start:locus-agent-{run_id}"),
                format!("pty:locus-agent-{run_id}")
            ]
        );
        assert_eq!(spawned.image_disposition, ImageDisposition::Built);
        assert_eq!(spawned.container.mounts[1].destination, CONFIG_SOURCE);
        assert_eq!(
            spawned.container.mounts,
            vec![
                Mount {
                    source: "/tmp/locus.sock".into(),
                    destination: "/run/locus.sock".into(),
                    read_only: false
                },
                Mount {
                    source: config_root.display().to_string(),
                    destination: CONFIG_SOURCE.into(),
                    read_only: true
                }
            ]
        );
        assert_eq!(spawned.container.network, "locus-project-1");
        assert!(spawned
            .container
            .environment
            .iter()
            .any(|value| value == &format!("LOCUS_PORT={}", spawned.port)));
        assert_eq!(
            runtime.attached,
            Some((spawned.container.name.clone(), AGENT_PTY))
        );

        let _ = fs::remove_dir_all(config_root);
    }
}
