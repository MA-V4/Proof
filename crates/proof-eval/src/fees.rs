use proof_dsl::ast::{FeesBlock, FeeAmount};
use crate::types::{EvalInput, EvalOutput};
use rust_decimal::Decimal;
use anyhow::{Result, anyhow};

pub fn apply_fee(fees: &FeesBlock, fee_name: &str, input: &EvalInput) -> Result<EvalOutput> {
    let fee = fees.fees.iter()
        .find(|f| f.name == fee_name)
        .ok_or_else(|| anyhow!("fee '{}' not found in spec", fee_name))?;

    let amount = match &fee.amount {
        FeeAmount::Fixed(a)        => *a,
        FeeAmount::Percentage(r)   => (input.balance * r.0 / Decimal::from(100)).round_dp(2),
    };

    Ok(EvalOutput {
        applied_tier: None,
        rate_applied:  None,
        amount:        Some(amount),
        reasoning: vec![
            format!("Fee: {} - £{}", fee_name, amount),
            format!("Waivable: {}", fee.waivable),
        ],
    })
}
