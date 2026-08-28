//! The run and container supervisors: spawn, stream, normalize, persist, cancel, and the
//! session/run/turn model everything above depends on. PLAN.md §Process topology.

pub mod acp;
pub mod backend;
pub mod boot;
pub mod container;
pub mod controls;
pub mod daemon;
pub mod dap;
pub mod dispatch;
pub mod invoke;
pub mod normalize;
pub mod routing;
pub mod run;
pub mod session;
