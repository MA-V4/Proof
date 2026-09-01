use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryFlag {
    pub rule:        String,
    pub severity:    Severity,
    pub description: String,
    pub action:      String,
    pub notice_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity { Info, Review, Block }

/// The numbers the regulatory checkers need to evaluate obligations.
/// Kept generic so proof-regulatory doesn't depend on proof-sim types.
#[derive(Debug, Clone)]
pub struct CheckInput {
    pub customers_worse:   u64,
    pub customers_better:  u64,
    pub customers_neutral: u64,
    pub monthly_delta:     Decimal,
}