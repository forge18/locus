//! The harness registry: what a harness declares, and the code that turns those
//! declarations into a config tree. PLAN.md §Process topology calls this the harness
//! registry — "load `harnesses/*`, materialize config per run".
//!
//! Nothing here names a harness. `scripts/check-no-harness-names-in-core.sh` enforces it.

pub mod adapter;
pub mod canary;
pub mod materialize;
pub mod models;
pub mod registry;
pub mod selection;
