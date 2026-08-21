//! Nested agent invocation starts isolated child runs through the core-owned launcher.

use anyhow::Result;
use uuid::Uuid;

use crate::sandbox::workspace_clone_command;

/// Request received from a running agent through `locus agent invoke`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationRequest {
    pub caller_run_id: Uuid,
    pub agent: String,
    pub version: i32,
    pub clone_remote: String,
}

/// The isolated child run handed to the host run supervisor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedRunPlan {
    pub caller_run_id: Uuid,
    pub run_id: Uuid,
    pub agent: String,
    pub version: i32,
    pub container_name: String,
    pub clone_command: String,
}

/// Host-only boundary that creates the child container and clone.
pub trait NestedRunLauncher {
    fn start(&self, plan: &NestedRunPlan) -> Result<()>;
}

/// Core entry point for creating a nested agent run.
pub struct InvocationSupervisor<'launcher, Launcher> {
    launcher: &'launcher Launcher,
}

impl<'launcher, Launcher> InvocationSupervisor<'launcher, Launcher>
where
    Launcher: NestedRunLauncher,
{
    pub fn new(launcher: &'launcher Launcher) -> Self {
        Self { launcher }
    }

    pub fn invoke(&self, request: InvocationRequest) -> Result<NestedRunPlan> {
        let run_id = Uuid::new_v4();
        let plan = NestedRunPlan {
            caller_run_id: request.caller_run_id,
            run_id,
            agent: request.agent,
            version: request.version,
            container_name: format!("locus-agent-{run_id}"),
            clone_command: workspace_clone_command(&request.clone_remote, &run_id.to_string())?,
        };
        self.launcher.start(&plan)?;
        Ok(plan)
    }
}

#[cfg(test)]
mod nested_run {
    use super::*;

    #[derive(Default)]
    struct RecordingLauncher {
        plans: std::sync::Mutex<Vec<NestedRunPlan>>,
    }

    impl NestedRunLauncher for RecordingLauncher {
        fn start(&self, plan: &NestedRunPlan) -> anyhow::Result<()> {
            self.plans.lock().expect("launcher lock").push(plan.clone());
            Ok(())
        }
    }

    #[test]
    fn starts_in_its_own_container_and_clone() {
        let launcher = RecordingLauncher::default();
        let supervisor = InvocationSupervisor::new(&launcher);
        let caller_run_id = uuid::Uuid::new_v4();

        let nested = supervisor
            .invoke(InvocationRequest {
                caller_run_id,
                agent: "reviewer".into(),
                version: 2,
                clone_remote: "file:///var/lib/locus/repos/project.git".into(),
            })
            .expect("nested run starts");

        assert_ne!(
            nested.run_id, caller_run_id,
            "nested run has its own run id"
        );
        assert_eq!(
            nested.container_name,
            format!("locus-agent-{}", nested.run_id)
        );
        assert!(nested
            .clone_command
            .contains("git clone 'file:///var/lib/locus/repos/project.git' /workspace"));
        assert!(nested
            .clone_command
            .contains(&format!("agent/'{}'", nested.run_id)));
        assert_eq!(
            launcher.plans.lock().expect("launcher lock").as_slice(),
            &[nested]
        );
    }
}
