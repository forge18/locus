//! Persistence for the autorouting decision recorded on a run (`agents.runs`).
//!
//! Moved out of `runtime/routing.rs` so every query in the crate lives under `store/`.

use crate::ids::RunId;
use anyhow::{bail, Result};
use sqlx::query;

use crate::{
    runtime::routing::{ComplexityBand, RoutingDecision},
    store::Store,
};

impl Store {
    /// Store the routing selection alongside the actual model that answers a run.
    pub async fn record_routing_decision(
        &self,
        run_id: RunId,
        decision: &RoutingDecision,
    ) -> Result<()> {
        let updated = query(
            "UPDATE agents.runs
             SET resolved_model_id = $2,
                 routing_requested_band = $3,
                 routing_selected_band = $4,
                 routing_effort = $5,
                 routing_approval_required = $6
             WHERE id = $1",
        )
        .bind(run_id)
        .bind(&decision.model_id)
        .bind(band_name(decision.requested_band))
        .bind(decision.selected_band.map(band_name))
        .bind(decision.effort.to_string())
        .bind(decision.approval_required)
        .execute(self.pool())
        .await?;
        if updated.rows_affected() != 1 {
            bail!("run `{run_id}` does not exist")
        }
        Ok(())
    }
}

fn band_name(band: ComplexityBand) -> &'static str {
    match band {
        ComplexityBand::XtraLow => "xtra-low",
        ComplexityBand::Low => "low",
        ComplexityBand::Medium => "medium",
        ComplexityBand::High => "high",
        ComplexityBand::XtraHigh => "xtra-high",
        ComplexityBand::Max => "max",
    }
}
