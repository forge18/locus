//! The Locus core: everything an agent can do, implemented once.
//!
//! The module tree mirrors PLAN.md §Process topology rather than the order features were
//! built, so where a thing lives says what it is:
//!
//! - [`core`] — the composition root: the one place the subsystems are assembled
//! - [`ids`] — typed identifiers, so a session id cannot stand in for a run id
//! - [`harness`] — load `harnesses/*`, validate them, materialize a config tree per run
//! - [`runtime`] — spawn, stream, normalize, persist, cancel; the session/run/turn model
//! - [`sandbox`] — one container per run, and the credential boundary around it
//! - [`store`] — Postgres, and the only place a query lives
//! - [`services`] — shared services: memory, mail, board, wiki, telemetry, tools
//!
//! Shared services live here rather than in any harness adapter, so memory, mail, and the
//! rest behave identically no matter which harness a run uses. See PLAN.md §"Shared
//! services — one Rust implementation, every harness".

pub mod bus;
pub mod core;
pub mod harness;
pub mod ids;
pub mod ipc;
pub mod runtime;
pub mod sandbox;
pub mod services;
pub mod store;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;
