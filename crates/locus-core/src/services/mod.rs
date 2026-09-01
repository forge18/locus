//! Shared services — one Rust implementation, every harness.
//!
//! Memory, communication, board, wiki, telemetry, and tool access are agent capabilities,
//! not harness features. Each is written once here and reaches every harness through the
//! same `locus` CLI over `/run/locus.sock`. PLAN.md §Shared services.
//!
//! The rule this enforces: a capability never gets a per-harness implementation.

pub mod agent_interface;
pub mod agents;
pub mod analytics;
pub mod arbiter;
pub mod artifact;
pub mod ask;
pub mod board;
pub mod bots;
pub mod browse;
pub mod calibration;
pub mod capabilities;
pub mod compact;
pub mod condition;
pub mod handoff;
pub mod inbox;
pub mod interact;
pub mod lint;
pub mod mail;
pub mod manage;
pub mod market;
pub mod media;
pub mod memory;
pub mod metrics;
pub mod planning;
pub mod project;
pub mod provider;
pub mod qa;
pub mod schedule;
pub mod task;
pub mod telemetry;
pub mod tools;
pub mod wiki;
pub mod workflow;
