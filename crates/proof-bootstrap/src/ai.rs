use crate::detector::Language;
use anyhow::{anyhow, Result};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-sonnet-4-6";

/// Claude-assisted extraction for higher quality results.
/// Only called when the user explicitly passes --ai with an API key.
pub fn extract_with_claude(source: &str, language: &Language, api_key: &str) -> Result<String> {
    let prompt = build_prompt(source, language);

    let client = reqwest::blocking::Client::new();

    let body = serde_json::json!({
        "model":      MODEL,
        "max_tokens": 2048,
        "messages":   [{ "role": "user", "content": prompt }]
    });

    let resp = client
        .post(API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| anyhow!("API request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(anyhow!("Claude API error {}: {}", status, text));
    }

    let data: serde_json::Value = resp
        .json()
        .map_err(|e| anyhow!("failed to parse API response: {}", e))?;

    let content = data["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow!("no text in API response"))?;

    Ok(clean(content))
}

fn clean(raw: &str) -> String {
    let s = raw.trim();
    let s = s.strip_prefix("```proof").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}

fn build_prompt(source: &str, language: &Language) -> String {
    format!(
        r#"Analyze the following {lang} code and produce a PROOF specification.

PROOF syntax example:
product SavingsAccount {{
  jurisdiction: UK
  regulator:    FCA
  category:     deposit

  interest {{
    base_rate: 4.50%
    tiers {{
      when balance >= 10_000  rate: base_rate + 1.00%
      when balance >= 1_000   rate: base_rate + 0.50%
      otherwise               rate: base_rate
    }}
    promotional {{
      condition:      days_since_joined <= 90
      rate:           base_rate + 2.00%
      expires_after:  90 days
      non_renewable:  true
    }}
    accrual {{
      frequency:        daily
      basis:            ACT/365
      compound:         annually
      minimum_payable:  GBP 0.01
    }}
  }}

  fees {{
    fee "late_payment" {{
      amount:   GBP 12.00
      waivable: true
    }}
  }}

  protection {{
    scheme: FSCS
    limit:  GBP 85_000
  }}

  obligations {{
    cooling_off:          14 days
    rate_change_notice:   14 days
    annual_summary:       required
  }}
}}

Rules:
- Output ONLY raw .proof text. No markdown fences, no explanation.
- Mark inferred values with # INFERRED.
- Mark unknown values with # TODO: verify.
- Convert decimal rates: 0.045 becomes 4.50%.
- Use underscores in large numbers: 10000 becomes 10_000.
- Only include blocks you have evidence for.

Source code:
{source}"#,
        lang = language,
        source = source,
    )
}
