use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub spec_name:         String,
    pub old_version:       String,
    pub new_version:       String,
    pub customers_total:   u64,
    pub customers_worse:   u64,
    pub customers_better:  u64,
    pub customers_neutral: u64,
    pub monthly_delta:     Decimal,
    pub regulatory_flags:  Vec<RegulatoryFlag>,
    pub verdict:           Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryFlag {
    pub rule:        String,
    pub severity:    Severity,
    pub description: String,
    pub action:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity { Info, Review, Block }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict { DeployClean, DeployWithReview, DoNotDeploy }
