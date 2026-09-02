use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSpec {
    pub name: String,
    pub jurisdiction: Jurisdiction,
    pub regulator: Regulator,
    pub category: ProductCategory,
    pub interest: Option<InterestBlock>,
    pub fees: Option<FeesBlock>,
    pub protection: Option<ProtectionBlock>,
    pub obligations: Option<ObligationsBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    UK,
    EU,
    US,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Regulator {
    FCA,
    PRA,
    CFPB,
    EBA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductCategory {
    Deposit,
    Credit,
    Mortgage,
    Investment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestBlock {
    pub base_rate: Rate,
    pub tiers: Vec<Tier>,
    pub promotional: Option<Promotional>,
    pub accrual: Accrual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier {
    pub condition: Condition,
    pub rate: RateExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    BalanceGte(Decimal),
    BalanceLt(Decimal),
    DaysSinceJoinedLte(u32),
    ProductCountGte(u32),
    Otherwise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateExpr {
    Literal(Rate),
    BaseRate,
    BaseRatePlus(Rate),
    BaseRateMinus(Rate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate(pub Decimal); // stored as percentage: 4.5 = 4.5%

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotional {
    pub condition: Condition,
    pub rate: RateExpr,
    pub expires_after_days: u32,
    pub non_renewable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accrual {
    pub frequency: AccrualFrequency,
    pub basis: DayCountBasis,
    pub compound: CompoundFrequency,
    pub minimum_payable: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccrualFrequency {
    Daily,
    Monthly,
    Quarterly,
    Annually,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DayCountBasis {
    Act365,
    Act360,
    Thirty360,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompoundFrequency {
    Daily,
    Monthly,
    Quarterly,
    Annually,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesBlock {
    pub fees: Vec<Fee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub name: String,
    pub amount: FeeAmount,
    pub waivable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeeAmount {
    Fixed(Decimal),
    Percentage(Rate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionBlock {
    pub scheme: String,
    pub limit: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationsBlock {
    pub cooling_off_days: Option<u32>,
    pub rate_change_notice_days: Option<u32>,
    pub annual_summary: bool,
}
