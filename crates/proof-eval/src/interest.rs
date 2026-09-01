use proof_dsl::ast::{Condition, DayCountBasis, InterestBlock, RateExpr, Rate};
use crate::types::{EvalInput, EvalOutput};
use rust_decimal::Decimal;
use anyhow::Result;

pub fn daily_accrual(interest: &InterestBlock, input: &EvalInput) -> Result<EvalOutput> {
    let (tier_label, effective_rate) = resolve_effective_rate(interest, input);

    let days_in_year = Decimal::from(match interest.accrual.basis {
        DayCountBasis::Act365   => 365,
        DayCountBasis::Act360   => 360,
        DayCountBasis::Thirty360 => 360,
    });

    // daily = balance × (rate% / 100) / days_in_year
    let rate_decimal = effective_rate / Decimal::from(100);
    let raw_daily    = input.balance * rate_decimal / days_in_year;

    // apply minimum payable floor
    let daily = match interest.accrual.minimum_payable {
        Some(min) if raw_daily < min => Decimal::ZERO,
        _                            => raw_daily,
    };
    let daily = daily.round_dp(2);

    Ok(EvalOutput {
        applied_tier: Some(tier_label.clone()),
        rate_applied:  Some(effective_rate),
        amount:        Some(daily),
        reasoning: vec![
            format!("Tier:    {}", tier_label),
            format!("Rate:    {}%", effective_rate),
            format!("Basis:   ACT/{}", days_in_year),
            format!("Daily:   £{} × {}% / {} = £{}", input.balance, effective_rate, days_in_year, daily),
        ],
    })
}

fn resolve_effective_rate(interest: &InterestBlock, input: &EvalInput) -> (String, Decimal) {
    // promotional rate takes priority
    if let Some(promo) = &interest.promotional {
        if condition_matches(&promo.condition, input) {
            let rate = resolve_rate(&promo.rate, &interest.base_rate);
            return ("promotional".into(), rate);
        }
    }
    // walk tiers in order; first match wins
    for (i, tier) in interest.tiers.iter().enumerate() {
        if condition_matches(&tier.condition, input) {
            let rate  = resolve_rate(&tier.rate, &interest.base_rate);
            let label = if matches!(tier.condition, Condition::Otherwise) {
                "base".into()
            } else {
                format!("tier_{}", i + 1)
            };
            return (label, rate);
        }
    }
    ("base".into(), interest.base_rate.0)
}

pub fn condition_matches(condition: &Condition, input: &EvalInput) -> bool {
    match condition {
        Condition::BalanceGte(t)        => input.balance >= *t,
        Condition::BalanceLt(t)         => input.balance < *t,
        Condition::DaysSinceJoinedLte(d) => input.days_since_joined.map_or(false, |v| v <= *d),
        Condition::ProductCountGte(c)   => input.product_count.map_or(false, |v| v >= *c),
        Condition::Otherwise            => true,
    }
}

pub fn resolve_rate(expr: &RateExpr, base: &Rate) -> Decimal {
    match expr {
        RateExpr::Literal(r)       => r.0,
        RateExpr::BaseRate         => base.0,
        RateExpr::BaseRatePlus(r)  => base.0 + r.0,
        RateExpr::BaseRateMinus(r) => base.0 - r.0,
    }
}
