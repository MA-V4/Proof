use anyhow::{Context, Result};
use proof_dsl::ast::ProductSpec;
use proof_eval::{evaluate, types::EvalInput};
use proof_regulatory::{CheckInput, RegulatoryRegistry};
use rust_decimal::Decimal;
use std::path::Path;

use crate::report::{CustomerResult, SimulationReport};
use crate::cohort::analyze;

/// Run a portfolio through two specs and return a SimulationReport.
pub fn run_simulation(
    old_spec:     &ProductSpec,
    new_spec:     &ProductSpec,
    portfolio:    Vec<EvalInput>,
) -> Result<SimulationReport> {
    let mut results = Vec::new();

    for event in portfolio {
        let old_amount = evaluate(old_spec, &event)
            .ok()
            .and_then(|o| o.amount)
            .unwrap_or(Decimal::ZERO);

        let new_amount = evaluate(new_spec, &event)
            .ok()
            .and_then(|o| o.amount)
            .unwrap_or(Decimal::ZERO);

        results.push(CustomerResult {
            customer_id: event.customer_id.clone(),
            balance:     event.balance,
            old_amount:  old_amount.round_dp(4),
            new_amount:  new_amount.round_dp(4),
            delta:       (new_amount - old_amount).round_dp(4),
        });
    }

    let cohorts = analyze(&results);

    let check_input = CheckInput {
        customers_worse:   cohorts.worse,
        customers_better:  cohorts.better,
        customers_neutral: cohorts.neutral,
        monthly_delta:     cohorts.total_delta * Decimal::from(30),
    };

    let registry = RegulatoryRegistry::new();
    let flags    = registry.check(&check_input);
    let verdict  = registry.verdict(&flags);

    Ok(SimulationReport {
        spec_name:         old_spec.name.clone(),
        old_version:       "current".into(),
        new_version:       "proposed".into(),
        customers_total:   results.len() as u64,
        customers_worse:   cohorts.worse,
        customers_better:  cohorts.better,
        customers_neutral: cohorts.neutral,
        daily_delta:       cohorts.total_delta.round_dp(4),
        monthly_delta:     (cohorts.total_delta * Decimal::from(30)).round_dp(2),
        avg_delta_worse:   cohorts.avg_delta_worse,
        regulatory_flags:  flags,
        verdict,
    })
}

/// Read a portfolio NDJSON file into a Vec<EvalInput>.
pub fn read_portfolio(path: impl AsRef<Path>) -> Result<Vec<EvalInput>> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    let content = raw.strip_prefix('\u{FEFF}').unwrap_or(&raw);
    let mut out = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") { continue; }
        let event: EvalInput = serde_json::from_str(line)
            .with_context(|| format!("{} line {}: invalid JSON", path.display(), i + 1))?;
        out.push(event);
    }

    Ok(out)
}