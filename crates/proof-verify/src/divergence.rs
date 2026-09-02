use chrono::{DateTime, Utc};
use proof_eval::types::EvalOutput;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub customer_id: String,
    pub spec_name: String,
    pub event_type: String,
    pub balance: Decimal,
    pub spec_output: EvalOutput,
    pub system_output: EvalOutput,
    pub diffs: Vec<FieldDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    pub field: String,
    pub spec_value: String,
    pub system_value: String,
    pub delta: Option<Decimal>,
    pub delta_pct: Option<Decimal>,
}
