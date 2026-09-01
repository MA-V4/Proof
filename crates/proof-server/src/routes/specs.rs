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
    divergences: i64,
    status:      &'static str,
}

pub async fn list_specs(State(state): State<SharedState>) -> AppResult<Json<Vec<SpecSummary>>> {
    let s  = state.read().await;
    let db = s.db.clone();
    let names: Vec<String> = s.specs.keys().cloned().collect();
    drop(s);

    let mut specs = Vec::new();
    for name in names {
        let count = db.count_divergences(Some(&name)).await.unwrap_or(0);
        specs.push(SpecSummary {
            status: if count == 0 { "clean" } else { "divergence" },
            name,
            divergences: count,
        });
    }

    Ok(Json(specs))
}

pub async fn get_divergences(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> AppResult<Json<Vec<proof_verify::Divergence>>> {
    let s  = state.read().await;
    let db = s.db.clone();
    if !s.specs.contains_key(&name) {
        return Err(anyhow::anyhow!("spec '{}' not found", name).into());
    }
    drop(s);

    let divs = db.get_divergences(Some(&name)).await?;
    Ok(Json(divs))
}

pub async fn resolve_divergence(
    State(state): State<SharedState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let s  = state.read().await;
    let db = s.db.clone();
    drop(s);

    match db.resolve_divergence(&id).await {
        Ok(true)  => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((StatusCode::NOT_FOUND, format!("divergence {} not found", id))),
        Err(e)    => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}