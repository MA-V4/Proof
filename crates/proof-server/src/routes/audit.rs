use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use proof_audit::{AuditEntry, AuditEntryKind, FcaAuditPack};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::AppResult, state::SharedState};

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
}

pub async fn get_spec_audit(
    State(state): State<SharedState>,
    Path(spec_name): Path<String>,
    Query(q): Query<AuditQuery>,
) -> AppResult<Json<Vec<AuditEntry>>> {
    let db = state.read().await.db.clone();
    let entries = db.get_audit(Some(&spec_name), q.limit.unwrap_or(100)).await?;
    Ok(Json(entries))
}

pub async fn get_all_audit(
    State(state): State<SharedState>,
    Query(q): Query<AuditQuery>,
) -> AppResult<Json<Vec<AuditEntry>>> {
    let db = state.read().await.db.clone();
    let entries = db.get_audit(None, q.limit.unwrap_or(200)).await?;
    Ok(Json(entries))
}

pub async fn export_fca_pack(
    State(state): State<SharedState>,
    Path(spec_name): Path<String>,
) -> AppResult<Json<FcaAuditPack>> {
    let db   = state.read().await.db.clone();
    let pack = db.fca_pack(&spec_name).await?;
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
    let (hash, db) = {
        let s = state.read().await;
        let hash = s.spec_hashes.get(&spec_name).cloned().unwrap_or_default();
        let db   = s.db.clone();
        (hash, db)
    };

    let entry = AuditEntry {
        id:        Uuid::new_v4(),
        timestamp: Utc::now(),
        spec_name: spec_name.clone(),
        spec_hash: hash,
        actor:     body.approver.clone(),
        kind:      AuditEntryKind::SpecSignedOff { approver: body.approver },
    };

    db.insert_audit(&entry).await?;
    Ok(Json(SignOffResponse { ok: true, entry }))
}