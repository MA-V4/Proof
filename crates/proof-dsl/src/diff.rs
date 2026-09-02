use crate::ast::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiffItem {
    BaseRateChanged {
        old: Decimal,
        new: Decimal,
        delta: Decimal,
    },
    TierThresholdChanged {
        tier_index: usize,
        old_threshold: Decimal,
        new_threshold: Decimal,
    },
    TierRateChanged {
        tier_index: usize,
        old_rate: String,
        new_rate: String,
    },
    TierAdded {
        tier_index: usize,
    },
    TierRemoved {
        tier_index: usize,
    },
    PromotionalRateChanged {
        old: String,
        new: String,
    },
    ObligationChanged {
        field: String,
        old: String,
        new: String,
    },
}

pub fn diff_specs(old: &ProductSpec, new: &ProductSpec) -> Vec<DiffItem> {
    let mut items = Vec::new();

    if let (Some(oi), Some(ni)) = (&old.interest, &new.interest) {
        if oi.base_rate.0 != ni.base_rate.0 {
            items.push(DiffItem::BaseRateChanged {
                old: oi.base_rate.0,
                new: ni.base_rate.0,
                delta: ni.base_rate.0 - oi.base_rate.0,
            });
        }

        let max = oi.tiers.len().max(ni.tiers.len());
        for i in 0..max {
            match (oi.tiers.get(i), ni.tiers.get(i)) {
                (Some(ot), Some(nt)) => {
                    if let (Some(old_t), Some(new_t)) = (threshold(ot), threshold(nt)) {
                        if old_t != new_t {
                            items.push(DiffItem::TierThresholdChanged {
                                tier_index: i,
                                old_threshold: old_t,
                                new_threshold: new_t,
                            });
                        }
                    }
                    let or_ = rate_str(&ot.rate);
                    let nr_ = rate_str(&nt.rate);
                    if or_ != nr_ {
                        items.push(DiffItem::TierRateChanged {
                            tier_index: i,
                            old_rate: or_,
                            new_rate: nr_,
                        });
                    }
                }
                (None, Some(_)) => items.push(DiffItem::TierAdded { tier_index: i }),
                (Some(_), None) => items.push(DiffItem::TierRemoved { tier_index: i }),
                _ => {}
            }
        }

        if let (Some(op), Some(np)) = (&oi.promotional, &ni.promotional) {
            let or_ = rate_str(&op.rate);
            let nr_ = rate_str(&np.rate);
            if or_ != nr_ {
                items.push(DiffItem::PromotionalRateChanged { old: or_, new: nr_ });
            }
        }
    }

    if let (Some(oo), Some(no_)) = (&old.obligations, &new.obligations) {
        if oo.cooling_off_days != no_.cooling_off_days {
            items.push(DiffItem::ObligationChanged {
                field: "cooling_off".into(),
                old: fmt_days(oo.cooling_off_days),
                new: fmt_days(no_.cooling_off_days),
            });
        }
        if oo.rate_change_notice_days != no_.rate_change_notice_days {
            items.push(DiffItem::ObligationChanged {
                field: "rate_change_notice".into(),
                old: fmt_days(oo.rate_change_notice_days),
                new: fmt_days(no_.rate_change_notice_days),
            });
        }
    }

    items
}

fn threshold(tier: &Tier) -> Option<Decimal> {
    match &tier.condition {
        Condition::BalanceGte(d) | Condition::BalanceLt(d) => Some(*d),
        Condition::DaysSinceJoinedLte(d) => Some(Decimal::from(*d)),
        _ => None,
    }
}

fn rate_str(expr: &RateExpr) -> String {
    match expr {
        RateExpr::Literal(r) => format!("{}%", r.0),
        RateExpr::BaseRate => "base_rate".into(),
        RateExpr::BaseRatePlus(r) => format!("base_rate + {}%", r.0),
        RateExpr::BaseRateMinus(r) => format!("base_rate - {}%", r.0),
    }
}

fn fmt_days(d: Option<u32>) -> String {
    d.map(|v| format!("{} days", v))
        .unwrap_or_else(|| "none".into())
}
