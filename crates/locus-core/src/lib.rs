//! The Locus core: everything an agent can do, implemented once.
//!
//! Shared services live here rather than in any harness adapter, so memory, mail,
//! and the rest behave identically no matter which harness a run uses. See PLAN.md
//! §"Shared services — one Rust implementation, every harness".

pub mod acp;
pub mod agents;
pub mod artifact;
pub mod ask;
pub mod backup;
pub mod board;
pub mod bus;
pub mod daemon;
pub mod invoke;
pub mod ipc;
pub mod lint;
pub mod mail;
pub mod materialize;
pub mod memory;
pub mod models;
pub mod provider;
pub mod registry;
pub mod restore;
pub mod run;
pub mod sandbox;
pub mod session;
pub mod smoke;
pub mod store;
pub mod telemetry;
pub mod testkit;
pub mod tools;
pub mod wiki;
