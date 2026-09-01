use axum::{extract::{Path, Query, State}, Json};
use crate::{error::AppResult, state::SharedState};
use proof_audit::{AuditEntry, AuditEntryKind, AuditLog, build_fca_pack, FcaAuditPack};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

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
        .into_iter().rev().take(limit).cloned().collect();
    Ok(Json(entries))
}

pub async fn get_all_audit(
    State(state): State<SharedState>,
    Query(q): Query<AuditQuery>,
) -> AppResult<Json<Vec<AuditEntry>>> {
    let s = state.read().await;
    let limit = q.limit.unwrap_or(200);
    let entries: Vec<AuditEntry> = s.audit
        .all().iter().rev().take(limit).cloned().collect();
    Ok(Json(entries))
}

pub async fn export_fca_pack(
    State(state): State<SharedState>,
    Path(spec_name): Path<String>,
) -> AppResult<Json<FcaAuditPack>> {
    let s = state.read().await;
    let pack = build_fca_pack(&spec_name, s.audit.all());
    Ok(Json(pack))
}

#[derive(Deserialize)]
pub struct SignOffRequest {
    pub approver: String,
}

#[derive(Serialize)]
pub struct SignOffResponse {
    pub ok:    bool,
    pub entry: AuditEntry,
}

pub async fn sign_off(
    State(state): State<SharedState>,
    Path(spec_name): Path<String>,
    Json(body): Json<SignOffRequest>,
) -> AppResult<Json<SignOffResponse>> {
    let mut s = state.write().await;

    let hash = s.spec_hashes.get(&spec_name)
        .cloned()
        .unwrap_or_default();

    let entry = AuditEntry {
        id:        Uuid::new_v4(),
        timestamp: Utc::now(),
        spec_name: spec_name.clone(),
        spec_hash: hash,
        actor:     body.approver.clone(),
        kind:      AuditEntryKind::SpecSignedOff { approver: body.approver },
    };

    s.log_audit(entry.clone());
    Ok(Json(SignOffResponse { ok: true, entry }))
}