use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalInput {
    #[serde(default = "default_customer")]
    pub customer_id:       String,
    pub event_type:        EventType,
    pub balance:           Decimal,
    #[serde(default)]
    pub days_since_joined: Option<u32>,
    #[serde(default)]
    pub product_count:     Option<u32>,
}

fn default_customer() -> String { "cli".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    DailyAccrual,
    MonthlyInterestPayment,
    FeeCharge { fee_name: String },
    EligibilityCheck,
    LimitCalculation,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DailyAccrual              => write!(f, "daily_accrual"),
            Self::MonthlyInterestPayment    => write!(f, "monthly_interest_payment"),
            Self::FeeCharge { fee_name }    => write!(f, "fee_charge({})", fee_name),
            Self::EligibilityCheck          => write!(f, "eligibility_check"),
            Self::LimitCalculation          => write!(f, "limit_calculation"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalOutput {
    pub applied_tier: Option<String>,
    pub rate_applied:  Option<Decimal>,
    pub amount:        Option<Decimal>,
    pub reasoning:     Vec<String>,
}
