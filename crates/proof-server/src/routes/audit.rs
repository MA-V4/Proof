use axum::{extract::{Path, Query, State}, Json};
use crate::{error::AppResult, state::SharedState};
use proof_audit::AuditEntry;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<usize>,
}

pub async fn get_audit(
    State(state): State<SharedState>,
    Query(q): Query<AuditQuery>,
    Path(spec_name): Path<String>,
) -> AppResult<Json<Vec<AuditEntry>>> {
    let s = state.read().await;
    let limit = q.limit.unwrap_or(100);
    let entries: Vec<AuditEntry> = s.audit
        .for_spec(&spec_name)
        .into_iter()
        .rev()
        .take(limit)
        .cloned()
        .collect();
    Ok(Json(entries))
}

pub async fn get_all_audit(
    State(state): State<SharedState>,
    Query(q): Query<AuditQuery>,
) -> AppResult<Json<Vec<AuditEntry>>> {
    let s = state.read().await;
    let limit = q.limit.unwrap_or(200);
    let entries: Vec<AuditEntry> = s.audit
        .all()
        .iter()
        .rev()
        .take(limit)
        .cloned()
        .collect();
    Ok(Json(entries))
}