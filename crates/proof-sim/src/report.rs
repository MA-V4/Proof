use proof_regulatory::{RegulatoryFlag, Verdict};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResult {
    pub customer_id: String,
    pub balance:     Decimal,
    pub old_amount:  Decimal,
    pub new_amount:  Decimal,
    pub delta:       Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub spec_name:          String,
    pub old_version:        String,
    pub new_version:        String,
    pub customers_total:    u64,
    pub customers_worse:    u64,
    pub customers_better:   u64,
    pub customers_neutral:  u64,
    pub daily_delta:        Decimal,
    pub monthly_delta:      Decimal,
    pub avg_delta_worse:    Option<Decimal>,
    pub regulatory_flags:   Vec<RegulatoryFlag>,
    pub verdict:            Verdict,
}