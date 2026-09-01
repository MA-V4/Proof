// Phase 5 deliverable — /specs endpoints.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::{error::AppResult, state::SharedState};
use serde::Serialize;

#[derive(Serialize)]
pub struct SpecSummary {
    name:        String,
    divergences: usize,
    status:      &'static str,
}

pub async fn list_specs(State(state): State<SharedState>) -> Json<Vec<SpecSummary>> {
    let s = state.read().await;
    let specs: Vec<SpecSummary> = s
        .specs
        .keys()
        .map(|name| {
            let count = s.divergences.iter().filter(|d| &d.spec_name == name).count();
            SpecSummary {
                name:        name.clone(),
                divergences: count,
                status:      if count == 0 { "clean" } else { "divergence" },
            }
        })
        .collect();
    Json(specs)
}

pub async fn get_divergences(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> AppResult<Json<Vec<proof_verify::Divergence>>> {
    let s = state.read().await;
    if !s.specs.contains_key(&name) {
        return Err(anyhow::anyhow!("spec '{}' not found", name).into());
    }
    let divs: Vec<_> = s.divergences.iter()
        .filter(|d| d.spec_name == name)
        .cloned()
        .collect();
    Ok(Json(divs))
}

pub async fn resolve_divergence(
    State(state): State<SharedState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut s = state.write().await;
    let before = s.divergences.len();
    s.divergences.retain(|d| !(d.spec_name == name && d.id.to_string() == id));
    if s.divergences.len() < before {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, format!("divergence {} not found", id)))
    }
}