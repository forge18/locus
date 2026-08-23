//! Shared services — one Rust implementation, every harness.
//!
//! Memory, communication, board, wiki, telemetry, and tool access are agent capabilities,
//! not harness features. Each is written once here and reaches every harness through the
//! same `locus` CLI over `/run/locus.sock`. PLAN.md §Shared services.
//!
//! The rule this enforces: a capability never gets a per-harness implementation.

pub mod agents;
pub mod artifact;
pub mod ask;
pub mod board;
pub mod lint;
pub mod mail;
pub mod memory;
pub mod planning;
pub mod project;
pub mod provider;
pub mod telemetry;
pub mod tools;
pub mod wiki;
pub mod workflow;
