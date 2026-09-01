use axum::{extract::State, Json};
use proof_eval::types::EvalInput;
use proof_sim::SimulationReport;
use crate::{error::AppResult, state::SharedState};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SimulateRequest {
    /// Name of the currently loaded spec (old version)
    pub old_spec_name: String,
    /// Raw .proof file contents for the proposed new version
    pub new_spec_text: String,
    /// Portfolio of events to replay
    pub portfolio: Vec<EvalInput>,
}

pub async fn simulate(
    State(state): State<SharedState>,
    Json(body): Json<SimulateRequest>,
) -> AppResult<Json<SimulationReport>> {
    let s = state.read().await;

    let old_spec = s.specs.get(&body.old_spec_name)
        .ok_or_else(|| anyhow::anyhow!("spec '{}' not found", body.old_spec_name))?
        .clone();

    drop(s); // release read lock before doing CPU work

    let new_spec = proof_dsl::parse(&body.new_spec_text)
        .map_err(|e| anyhow::anyhow!("new spec parse error: {}", e))?;

    let report = proof_sim::run_simulation(&old_spec, &new_spec, body.portfolio)
        .map_err(|e| anyhow::anyhow!("simulation error: {}", e))?;

    Ok(Json(report))
}
