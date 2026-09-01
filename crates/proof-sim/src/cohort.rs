use crate::report::CustomerResult;
use rust_decimal::Decimal;

pub struct CohortSummary {
    pub worse:           u64,
    pub better:          u64,
    pub neutral:         u64,
    pub total_delta:     Decimal,
    pub avg_delta_worse: Option<Decimal>,
}

pub fn analyze(results: &[CustomerResult]) -> CohortSummary {
    let mut worse = 0u64;
    let mut better = 0u64;
    let mut neutral = 0u64;
    let mut total_delta = Decimal::ZERO;
    let mut worse_sum = Decimal::ZERO;

    for r in results {
        total_delta += r.delta;
        if r.delta < Decimal::ZERO {
            worse += 1;
            worse_sum += r.delta;
        } else if r.delta > Decimal::ZERO {
            better += 1;
        } else {
            neutral += 1;
        }
    }

    let avg_delta_worse = if worse > 0 {
        Some((worse_sum / Decimal::from(worse)).round_dp(4))
    } else {
        None
    };

    CohortSummary { worse, better, neutral, total_delta, avg_delta_worse }
}