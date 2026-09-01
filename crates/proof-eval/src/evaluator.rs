use proof_dsl::ProductSpec;
use crate::types::{EvalInput, EvalOutput, EventType};
use anyhow::{Result, anyhow};

pub fn evaluate(spec: &ProductSpec, input: &EvalInput) -> Result<EvalOutput> {
    match &input.event_type {
        EventType::DailyAccrual => {
            let interest = spec.interest.as_ref()
                .ok_or_else(|| anyhow!("spec '{}' has no interest block", spec.name))?;
            crate::interest::daily_accrual(interest, input)
        }
        EventType::FeeCharge { fee_name } => {
            let fees = spec.fees.as_ref()
                .ok_or_else(|| anyhow!("spec '{}' has no fees block", spec.name))?;
            crate::fees::apply_fee(fees, fee_name, input)
        }
        other => Err(anyhow!("event type {:?} not yet implemented", other)),
    }
}
