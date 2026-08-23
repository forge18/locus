//! The Locus core: everything an agent can do, implemented once.
//!
//! The module tree mirrors PLAN.md §Process topology rather than the order features were
//! built, so where a thing lives says what it is:
//!
//! - [`harness`] — load `harnesses/*`, validate them, materialize a config tree per run
//! - [`runtime`] — spawn, stream, normalize, persist, cancel; the session/run/turn model
//! - [`sandbox`] — one container per run, and the credential boundary around it
//! - [`store`] — Postgres, and the only place a query lives
//! - [`services`] — shared services: memory, mail, board, wiki, telemetry, tools
//!
//! Shared services live here rather than in any harness adapter, so memory, mail, and the
//! rest behave identically no matter which harness a run uses. See PLAN.md §"Shared
//! services — one Rust implementation, every harness".

pub mod harness;
pub mod ipc;
pub mod runtime;
pub mod sandbox;
pub mod services;
pub mod smoke;
pub mod store;
pub mod testkit;
