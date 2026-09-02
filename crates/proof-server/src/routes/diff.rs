use crate::error::AppResult;
use axum::Json;
use proof_dsl::{diff_specs, DiffItem};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DiffRequest {
    pub old_spec_text: String,
    pub new_spec_text: String,
}

pub async fn diff(Json(body): Json<DiffRequest>) -> AppResult<Json<Vec<DiffItem>>> {
    let old = proof_dsl::parse(&body.old_spec_text)
        .map_err(|e| anyhow::anyhow!("old spec error: {}", e))?;
    let new = proof_dsl::parse(&body.new_spec_text)
        .map_err(|e| anyhow::anyhow!("new spec error: {}", e))?;
    Ok(Json(diff_specs(&old, &new)))
}
