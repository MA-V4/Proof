use crate::divergence::{Divergence, FieldDiff};
use anyhow::Result;
use chrono::Utc;
use proof_dsl::ProductSpec;
use proof_eval::{
    evaluate,
    types::{EvalInput, EvalOutput},
};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Evaluate `spec` against `input`, then diff against `system_output`.
/// Returns Some(Divergence) if any field mismatches, None if clean.
pub fn compare(
    spec: &ProductSpec,
    input: &EvalInput,
    system_output: &EvalOutput,
) -> Result<Option<Divergence>> {
    let spec_output = evaluate(spec, input)?;
    let mut diffs = Vec::new();

    diff_decimal(
        "amount",
        spec_output.amount,
        system_output.amount,
        &mut diffs,
    );
    diff_decimal(
        "rate_applied",
        spec_output.rate_applied,
        system_output.rate_applied,
        &mut diffs,
    );
    diff_str(
        "applied_tier",
        &spec_output.applied_tier,
        &system_output.applied_tier,
        &mut diffs,
    );

    if diffs.is_empty() {
        return Ok(None);
    }

    Ok(Some(Divergence {
        id: Uuid::new_v4(),
        detected_at: Utc::now(),
        customer_id: input.customer_id.clone(),
        spec_name: spec.name.clone(),
        event_type: input.event_type.to_string(),
        balance: input.balance,
        spec_output,
        system_output: system_output.clone(),
        diffs,
    }))
}

fn diff_decimal(
    field: &str,
    spec: Option<Decimal>,
    sys: Option<Decimal>,
    out: &mut Vec<FieldDiff>,
) {
    match (spec, sys) {
        (Some(s), Some(y)) if s != y => {
            let delta = y - s;
            let pct = if s.is_zero() {
                None
            } else {
                Some((delta / s * Decimal::from(100)).round_dp(2))
            };
            out.push(FieldDiff {
                field: field.into(),
                spec_value: s.to_string(),
                system_value: y.to_string(),
                delta: Some(delta),
                delta_pct: pct,
            });
        }
        (Some(s), None) => out.push(FieldDiff {
            field: field.into(),
            spec_value: s.to_string(),
            system_value: "(missing)".into(),
            delta: None,
            delta_pct: None,
        }),
        (None, Some(y)) => out.push(FieldDiff {
            field: field.into(),
            spec_value: "(none)".into(),
            system_value: y.to_string(),
            delta: None,
            delta_pct: None,
        }),
        _ => {}
    }
}

fn diff_str(field: &str, spec: &Option<String>, sys: &Option<String>, out: &mut Vec<FieldDiff>) {
    if spec != sys {
        out.push(FieldDiff {
            field: field.into(),
            spec_value: spec.as_deref().unwrap_or("(none)").into(),
            system_value: sys.as_deref().unwrap_or("(none)").into(),
            delta: None,
            delta_pct: None,
        });
    }
}
