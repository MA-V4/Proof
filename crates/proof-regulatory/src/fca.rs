use crate::types::{CheckInput, RegulatoryFlag, Severity};

pub fn check(input: &CheckInput) -> Vec<RegulatoryFlag> {
    let mut flags = Vec::new();

    if input.customers_worse > 0 {
        flags.push(RegulatoryFlag {
            rule:        "FCA Consumer Duty".into(),
            severity:    Severity::Review,
            description: format!(
                "{} customer{} receive a material rate reduction",
                input.customers_worse,
                if input.customers_worse == 1 { "" } else { "s" }
            ),
            action:      "14-day advance notice required. Individual notification mandatory.".into(),
            notice_days: Some(14),
        });
    }

    if input.customers_worse > 10_000 {
        flags.push(RegulatoryFlag {
            rule:        "FCA Consumer Duty - Reporting threshold".into(),
            severity:    Severity::Review,
            description: format!("{} customers affected - FCA reporting threshold exceeded", input.customers_worse),
            action:      "Submit FCA change notification before activation.".into(),
            notice_days: None,
        });
    }

    flags
}