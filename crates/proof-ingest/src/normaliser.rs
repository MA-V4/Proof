use proof_eval::types::{EvalInput, EvalOutput, EventType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A single event line from the customer's running system.
/// Contains both the input context and what the system computed.
/// This is the NDJSON line format for batch files and the webhook payload shape.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemEvent {
    #[serde(default = "unknown")]
    pub customer_id: String,
    pub event_type: EventType,
    pub balance: Decimal,
    #[serde(default)]
    pub days_since_joined: Option<u32>,
    #[serde(default)]
    pub product_count: Option<u32>,
    pub system_output: SystemOutput,
}

/// The subset of EvalOutput that the customer's system reports.
/// reasoning is always empty - the system doesn't explain itself.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemOutput {
    pub amount: Option<Decimal>,
    pub applied_tier: Option<String>,
    pub rate_applied: Option<Decimal>,
}

fn unknown() -> String {
    "unknown".into()
}

/// Split a SystemEvent into (EvalInput, EvalOutput) for the comparator.
pub fn normalise(event: SystemEvent) -> (EvalInput, EvalOutput) {
    let input = EvalInput {
        customer_id: event.customer_id,
        event_type: event.event_type,
        balance: event.balance,
        days_since_joined: event.days_since_joined,
        product_count: event.product_count,
    };
    let output = EvalOutput {
        applied_tier: event.system_output.applied_tier,
        rate_applied: event.system_output.rate_applied,
        amount: event.system_output.amount,
        reasoning: vec![],
    };
    (input, output)
}
