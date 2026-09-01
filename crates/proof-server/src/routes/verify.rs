// Phase 5 deliverable — /verify endpoints.
use axum::{extract::{Path, State}, Json};
use chrono::Utc;
use crate::{
    error::AppResult,
    state::{RecentEvent, SharedState},
};
use proof_ingest::SystemEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct VerifyResponse {
    ok:         bool,
    divergence: Option<proof_verify::Divergence>,
}

pub async fn verify_event(
    State(state): State<SharedState>,
    Path(spec_name): Path<String>,
    Json(event): Json<SystemEvent>,
) -> AppResult<Json<VerifyResponse>> {
    let mut s = state.write().await;

    let spec = s.specs.get(&spec_name)
        .ok_or_else(|| anyhow::anyhow!("spec '{}' not found", spec_name))?
        .clone();

    let event_type_str = event.event_type.to_string();
    let customer_id    = event.customer_id.clone();
    let (input, system_output) = proof_ingest::normalise(event);

    let result = proof_verify::compare(&spec, &input, &system_output)?;
    let ok = result.is_none();

    s.push_event(RecentEvent {
        customer_id: customer_id.clone(),
        spec_name:   spec_name.clone(),
        event_type:  event_type_str,
        ok,
        timestamp:   Utc::now(),
    });

    if let Some(ref d) = result {
        s.divergences.push(d.clone());
    }

    Ok(Json(VerifyResponse { ok, divergence: result }))
}

#[derive(Deserialize)]
pub struct BatchRequest {
    events: Vec<SystemEvent>,
}

#[derive(Serialize)]
pub struct BatchResponse {
    verified:    usize,
    divergences: usize,
    results:     Vec<VerifyResponse>,
}

pub async fn verify_batch(
    State(state): State<SharedState>,
    Path(spec_name): Path<String>,
    Json(body): Json<BatchRequest>,
) -> AppResult<Json<BatchResponse>> {
    let mut s = state.write().await;

    let spec = s.specs.get(&spec_name)
        .ok_or_else(|| anyhow::anyhow!("spec '{}' not found", spec_name))?
        .clone();

    let mut results = Vec::new();
    let mut div_count = 0usize;

    for event in body.events {
        let event_type_str = event.event_type.to_string();
        let customer_id    = event.customer_id.clone();
        let (input, system_output) = proof_ingest::normalise(event);

        let result = proof_verify::compare(&spec, &input, &system_output)?;
        let ok = result.is_none();

        s.push_event(RecentEvent {
            customer_id,
            spec_name:  spec_name.clone(),
            event_type: event_type_str,
            ok,
            timestamp:  chrono::Utc::now(),
        });

        if let Some(ref d) = result {
            s.divergences.push(d.clone());
            div_count += 1;
        }

        results.push(VerifyResponse { ok, divergence: result });
    }

    Ok(Json(BatchResponse {
        verified:    results.len(),
        divergences: div_count,
        results,
    }))
}