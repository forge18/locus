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
    /// Trusted run ancestry resolved by the core before it handles the socket request.
    pub context: InvocationContext,
    /// Hard limits, optionally narrowed by the owning workflow.
    pub limits: InvocationLimits,
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

/// A completed child result routed back to the run that invoked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedRunReturn {
    pub run_id: Uuid,
    pub exit_code: i32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallerDelivery {
    pub caller_run_id: Uuid,
    pub result: NestedRunReturn,
}

/// Attaches a child result to its originating caller rather than treating it as a handoff.
pub fn return_to_caller(plan: &NestedRunPlan, result: NestedRunReturn) -> Result<CallerDelivery> {
    if result.run_id != plan.run_id {
        anyhow::bail!("nested result does not belong to this invocation")
    }
    Ok(CallerDelivery {
        caller_run_id: plan.caller_run_id,
        result,
    })
}

pub const MAX_INVOCATION_DEPTH: usize = 3;
pub const MAX_INVOCATION_FAN_OUT: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRef {
    pub name: String,
    pub version: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationContext {
    /// Agent definitions from the root through the calling run, inclusive.
    pub ancestry: Vec<AgentRef>,
    /// Children already started by the calling run.
    pub children_started: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvocationLimits {
    pub max_depth: usize,
    pub max_fan_out: usize,
}

impl InvocationLimits {
    pub const HARD: Self = Self {
        max_depth: MAX_INVOCATION_DEPTH,
        max_fan_out: MAX_INVOCATION_FAN_OUT,
    };

    /// Workflows may narrow the machine-safe hard bounds, never widen them.
    pub fn workflow(max_depth: usize, max_fan_out: usize) -> Result<Self> {
        if max_depth > Self::HARD.max_depth || max_fan_out > Self::HARD.max_fan_out {
            anyhow::bail!("workflow invocation limits may only lower the hard bounds")
        }
        Ok(Self {
            max_depth,
            max_fan_out,
        })
    }
}

/// Reject an invocation before a container or clone is created.
pub fn validate_invocation(
    context: &InvocationContext,
    target: &AgentRef,
    limits: InvocationLimits,
) -> Result<()> {
    if context.ancestry.len() >= limits.max_depth {
        anyhow::bail!("nested invocation exceeds depth limit {}", limits.max_depth)
    }
    if context.children_started >= limits.max_fan_out {
        anyhow::bail!(
            "nested invocation exceeds fan-out limit {}",
            limits.max_fan_out
        )
    }
    if context.ancestry.contains(target) {
        anyhow::bail!("nested invocation would create a cycle")
    }
    Ok(())
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
        validate_invocation(
            &request.context,
            &AgentRef {
                name: request.agent.clone(),
                version: request.version,
            },
            request.limits,
        )?;

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
mod depth_limit {
    use super::*;

    #[test]
    fn refuses_a_fourth_nested_level() {
        let context = InvocationContext {
            ancestry: vec![
                AgentRef {
                    name: "root".into(),
                    version: 1,
                },
                AgentRef {
                    name: "child".into(),
                    version: 1,
                },
                AgentRef {
                    name: "grandchild".into(),
                    version: 1,
                },
            ],
            children_started: 0,
        };
        let error = validate_invocation(
            &context,
            &AgentRef {
                name: "too-deep".into(),
                version: 1,
            },
            InvocationLimits::HARD,
        )
        .expect_err("depth four is refused");
        assert!(error.to_string().contains("depth limit 3"));
    }
}

#[cfg(test)]
mod fanout_limit {
    use super::*;

    #[test]
    fn refuses_a_fifth_child() {
        let context = InvocationContext {
            ancestry: vec![AgentRef {
                name: "root".into(),
                version: 1,
            }],
            children_started: 4,
        };
        let error = validate_invocation(
            &context,
            &AgentRef {
                name: "fifth".into(),
                version: 1,
            },
            InvocationLimits::HARD,
        )
        .expect_err("fifth child is refused");
        assert!(error.to_string().contains("fan-out limit 4"));
    }
}

#[cfg(test)]
mod cycle_check {
    use super::*;

    #[test]
    fn refuses_a_target_already_in_its_ancestry() {
        let root = AgentRef {
            name: "root".into(),
            version: 1,
        };
        let context = InvocationContext {
            ancestry: vec![
                root.clone(),
                AgentRef {
                    name: "reviewer".into(),
                    version: 2,
                },
            ],
            children_started: 0,
        };
        let error = validate_invocation(&context, &root, InvocationLimits::HARD)
            .expect_err("cycle is refused");
        assert!(error.to_string().contains("cycle"));
    }
}

#[cfg(test)]
mod three_limits {
    use super::*;

    #[test]
    fn each_guard_rejects_while_the_other_two_are_permitted() {
        let target = AgentRef {
            name: "target".into(),
            version: 1,
        };
        let no_cycle = InvocationContext {
            ancestry: vec![],
            children_started: 0,
        };
        assert!(validate_invocation(&no_cycle, &target, InvocationLimits::HARD).is_ok());

        let depth_only = InvocationContext {
            ancestry: vec![
                AgentRef {
                    name: "one".into(),
                    version: 1,
                },
                AgentRef {
                    name: "two".into(),
                    version: 1,
                },
                AgentRef {
                    name: "three".into(),
                    version: 1,
                },
            ],
            children_started: 0,
        };
        assert!(validate_invocation(&depth_only, &target, InvocationLimits::HARD).is_err());

        let fanout_only = InvocationContext {
            ancestry: vec![],
            children_started: 4,
        };
        assert!(validate_invocation(&fanout_only, &target, InvocationLimits::HARD).is_err());

        let cycle_only = InvocationContext {
            ancestry: vec![target.clone()],
            children_started: 0,
        };
        assert!(validate_invocation(&cycle_only, &target, InvocationLimits::HARD).is_err());
    }
}

#[cfg(test)]
mod workflow_lowers_only {
    use super::*;

    #[test]
    fn accepts_narrower_bounds_and_refuses_wider_ones() {
        assert_eq!(
            InvocationLimits::workflow(2, 3).expect("narrower bounds work"),
            InvocationLimits {
                max_depth: 2,
                max_fan_out: 3
            }
        );
        assert!(InvocationLimits::workflow(4, 4).is_err());
        assert!(InvocationLimits::workflow(3, 5).is_err());
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
                context: InvocationContext {
                    ancestry: vec![AgentRef {
                        name: "builder".into(),
                        version: 1,
                    }],
                    children_started: 0,
                },
                limits: InvocationLimits::HARD,
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

    #[test]
    fn refuses_an_invalid_invocation_before_starting_a_container() {
        let launcher = RecordingLauncher::default();
        let supervisor = InvocationSupervisor::new(&launcher);
        let result = supervisor.invoke(InvocationRequest {
            caller_run_id: Uuid::new_v4(),
            agent: "fourth-child".into(),
            version: 1,
            clone_remote: "file:///var/lib/locus/repos/project.git".into(),
            context: InvocationContext {
                ancestry: vec![
                    AgentRef {
                        name: "root".into(),
                        version: 1,
                    },
                    AgentRef {
                        name: "child".into(),
                        version: 1,
                    },
                    AgentRef {
                        name: "grandchild".into(),
                        version: 1,
                    },
                ],
                children_started: 0,
            },
            limits: InvocationLimits::HARD,
        });

        assert!(result.is_err());
        assert!(launcher.plans.lock().expect("launcher lock").is_empty());
    }
}

#[cfg(test)]
mod returns {
    use super::*;

    #[test]
    fn routes_a_child_completion_back_to_its_caller() {
        let caller_run_id = Uuid::new_v4();
        let child_run_id = Uuid::new_v4();
        let plan = NestedRunPlan {
            caller_run_id,
            run_id: child_run_id,
            agent: "reviewer".into(),
            version: 2,
            container_name: "locus-agent-child".into(),
            clone_command: "git clone".into(),
        };
        let result = NestedRunReturn {
            run_id: child_run_id,
            exit_code: 0,
            summary: "review complete".into(),
        };

        assert_eq!(
            return_to_caller(&plan, result.clone()).expect("return is routed"),
            CallerDelivery {
                caller_run_id,
                result,
            }
        );
    }
}
