use crate::detector::Language;
use regex::Regex;

#[derive(Debug, Default)]
pub struct ExtractedSpec {
    pub product_name: String,
    pub base_rate: Option<f64>,
    pub tiers: Vec<ExtractedTier>,
    pub accrual_basis: u32,
    pub promotional: Option<ExtractedPromo>,
    pub fees: Vec<ExtractedFee>,
    pub fscs_limit: Option<f64>,
    pub cooling_off: Option<u32>,
    pub notice_days: Option<u32>,
    pub min_payable: Option<f64>,
}

#[derive(Debug)]
pub struct ExtractedTier {
    pub threshold: f64,
    pub rate_modifier: f64,
    pub inferred: bool,
}

#[derive(Debug)]
pub struct ExtractedPromo {
    pub days: u32,
    pub rate_modifier: f64,
}

#[derive(Debug)]
pub struct ExtractedFee {
    pub name: String,
    pub amount: f64,
    pub is_percentage: bool,
    pub waivable: bool,
}

pub fn extract_heuristic(source: &str, _language: &Language) -> ExtractedSpec {
    let mut spec = ExtractedSpec {
        accrual_basis: 365,
        ..Default::default()
    };

    spec.product_name = detect_product_name(source);
    spec.base_rate = detect_base_rate(source);
    spec.tiers = detect_tiers(source);
    spec.accrual_basis = detect_accrual_basis(source);
    spec.promotional = detect_promotional(source);
    spec.fees = detect_fees(source);
    spec.fscs_limit = detect_fscs(source);
    spec.cooling_off = detect_notice(source, &["cooling_off", "cooling off", "cooloff"]);
    spec.notice_days = detect_notice(
        source,
        &["rate_change_notice", "notice_period", "notice_days"],
    );
    spec.min_payable = detect_min_payable(source);

    spec
}

fn detect_product_name(source: &str) -> String {
    // Try class name first
    let class_re = Regex::new(r"class\s+([A-Z][A-Za-z]+)").unwrap();
    if let Some(cap) = class_re.captures(source) {
        return cap[1].to_string();
    }

    // Try function name containing calculate
    let fn_re = Regex::new(r"def\s+calculate_([a-z_]+)|function\s+calculate([A-Za-z]+)").unwrap();
    if let Some(cap) = fn_re.captures(source) {
        let name = cap
            .get(1)
            .or(cap.get(2))
            .map(|m| m.as_str())
            .unwrap_or("product");
        return to_pascal_case(name);
    }

    // Try docstring or comment
    for line in source.lines().take(10) {
        let line = line
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches('#')
            .trim();
        if line.len() > 3 && line.len() < 40 && !line.starts_with("import") {
            let words: Vec<&str> = line.split_whitespace().collect();
            if !words.is_empty() {
                return words[0].to_string();
            }
        }
    }

    "FinancialProduct".to_string()
}

fn detect_base_rate(source: &str) -> Option<f64> {
    let patterns = [
        r"BASE_RATE\s*=\s*([\d.]+)",
        r"base_rate\s*=\s*([\d.]+)",
        r"INTEREST_RATE\s*=\s*([\d.]+)",
        r"ANNUAL_RATE\s*=\s*([\d.]+)",
        r"rate\s*=\s*([\d.]+)\s*#.*base",
    ];

    for pat in &patterns {
        let re = Regex::new(pat).unwrap();
        if let Some(cap) = re.captures(source) {
            if let Ok(val) = cap[1].parse::<f64>() {
                // If < 1 assume decimal form: 0.045 -> 4.5%
                let rate = if val < 1.0 { val * 100.0 } else { val };
                if rate > 0.0 && rate < 50.0 {
                    // sanity check
                    return Some(rate);
                }
            }
        }
    }
    None
}

fn detect_tiers(source: &str) -> Vec<ExtractedTier> {
    let mut tiers = Vec::new();

    // Match: balance >= 10_000 or balance >= 10000, followed by rate assignment
    let tier_re = Regex::new(
        r"(?:if|elif|else if)\s*\(?balance\s*>=\s*([\d_]+)\)?\s*[:\{]?\s*\n?\s*(?:.*?rate[^=\n]*=\s*[^\n]*?\+\s*([\d.]+)|.*?rate[^=\n]*=\s*([\d.]+))"
    ).unwrap();

    for cap in tier_re.captures_iter(source) {
        let threshold_str = cap[1].replace('_', "");
        let threshold = threshold_str.parse::<f64>().unwrap_or(0.0);

        // Rate modifier: from +X.XX pattern or just the rate
        let modifier = if let Some(m) = cap.get(2) {
            m.as_str().parse::<f64>().unwrap_or(0.0)
        } else if let Some(m) = cap.get(3) {
            let v = m.as_str().parse::<f64>().unwrap_or(0.0);
            if v < 1.0 {
                v * 100.0
            } else {
                v
            }
        } else {
            0.0
        };

        tiers.push(ExtractedTier {
            threshold,
            rate_modifier: modifier,
            inferred: false,
        });
    }

    // Sort tiers descending by threshold (highest first)
    tiers.sort_by(|a, b| b.threshold.partial_cmp(&a.threshold).unwrap());
    tiers.dedup_by(|a, b| (a.threshold - b.threshold).abs() < 0.01);
    tiers
}

fn detect_accrual_basis(source: &str) -> u32 {
    if source.contains("/ 360") || source.contains("/360") || source.contains("ACT/360") {
        return 360;
    }
    if source.contains("30/360") || source.contains("Thirty360") {
        return 360;
    }
    365 // default ACT/365
}

fn detect_promotional(source: &str) -> Option<ExtractedPromo> {
    // Look for: days_since_joined <= 90, PROMOTIONAL_DAYS = 90
    let days_re =
        Regex::new(r"(?:days_since_joined|days_joined|member_days|join_days)\s*<=?\s*(\d+)")
            .unwrap();

    let promo_const_re =
        Regex::new(r"PROMOTIONAL_DAYS\s*=\s*(\d+)|PROMO_DAYS\s*=\s*(\d+)|INTRO_DAYS\s*=\s*(\d+)")
            .unwrap();

    let days = days_re
        .captures(source)
        .and_then(|c| c[1].parse::<u32>().ok())
        .or_else(|| {
            promo_const_re.captures(source).and_then(|c| {
                c.get(1)
                    .or(c.get(2))
                    .or(c.get(3))
                    .and_then(|m| m.as_str().parse::<u32>().ok())
            })
        })?;

    // Look for rate modifier associated with promotional block
    let rate_re = Regex::new(r"\+\s*([\d.]+)").unwrap();
    let rate_modifier = rate_re
        .captures_iter(source)
        .filter_map(|c| c[1].parse::<f64>().ok())
        .find(|&r| r > 0.5 && r < 10.0) // reasonable promo bonus range
        .unwrap_or(0.0);

    Some(ExtractedPromo {
        days,
        rate_modifier,
    })
}

fn detect_fees(source: &str) -> Vec<ExtractedFee> {
    let mut fees = Vec::new();

    // Late payment fee
    let late_re =
        Regex::new(r"(?:LATE_PAYMENT_FEE|LATE_FEE|late_payment_fee|late_fee)\s*=\s*([\d.]+)")
            .unwrap();
    if let Some(cap) = late_re.captures(source) {
        if let Ok(amount) = cap[1].parse::<f64>() {
            fees.push(ExtractedFee {
                name: "late_payment".to_string(),
                amount,
                is_percentage: false,
                waivable: source.to_lowercase().contains("waiv"),
            });
        }
    }

    // Early repayment charge
    let erc_re = Regex::new(r"(?:EARLY_REPAYMENT|ERC|early_repayment)\w*\s*=\s*([\d.]+)").unwrap();
    if let Some(cap) = erc_re.captures(source) {
        if let Ok(amount) = cap[1].parse::<f64>() {
            let is_pct = amount <= 1.0;
            fees.push(ExtractedFee {
                name: "early_repayment".to_string(),
                amount: if is_pct { amount * 100.0 } else { amount },
                is_percentage: is_pct,
                waivable: false,
            });
        }
    }

    fees
}

fn detect_fscs(source: &str) -> Option<f64> {
    let re =
        Regex::new(r"(?:FSCS_LIMIT|FSCS|fscs_limit|protection_limit)\s*=\s*([\d_]+(?:\.\d+)?)")
            .unwrap();
    re.captures(source)
        .and_then(|c| c[1].replace('_', "").parse::<f64>().ok())
}

fn detect_notice(source: &str, keys: &[&str]) -> Option<u32> {
    for key in keys {
        let pat = format!(r"(?i){}\s*=\s*(\d+)", key.replace('_', "[_\\s]?"));
        if let Ok(re) = Regex::new(&pat) {
            if let Some(cap) = re.captures(source) {
                if let Ok(days) = cap[1].parse::<u32>() {
                    return Some(days);
                }
            }
        }
    }
    None
}

fn detect_min_payable(source: &str) -> Option<f64> {
    let re = Regex::new(
        r"(?:MINIMUM_DAILY_PAYMENT|MIN_PAYMENT|minimum_payment|min_daily)\s*=\s*([\d.]+)",
    )
    .unwrap();
    re.captures(source).and_then(|c| c[1].parse::<f64>().ok())
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}
