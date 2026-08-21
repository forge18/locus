//! The Locus core: everything an agent can do, implemented once.
//!
//! Shared services live here rather than in any harness adapter, so memory, mail,
//! and the rest behave identically no matter which harness a run uses. See PLAN.md
//! §"Shared services — one Rust implementation, every harness".

pub mod backup;
pub mod board;
pub mod bus;
pub mod mail;
pub mod memory;
pub mod store;
pub mod telemetry;
pub mod tools;
pub mod wiki;
