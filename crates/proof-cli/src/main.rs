use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use proof_verify::alert::AlertSink;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("check")    => cmd_check(&args[2..]),
        Some("ir")       => cmd_ir(&args[2..]),
        Some("verify")   => cmd_verify(&args[2..]),
        Some("simulate") => { eprintln!("simulate: Phase 6"); Ok(()) }
        Some("audit")    => { eprintln!("audit: Phase 8"); Ok(()) }
        Some("diff")     => { eprintln!("diff: Phase 9"); Ok(()) }
        _ => {
            println!("PROOF — financial logic, verified pure.\n");
            println!("  check    <spec.proof> [--input json]          Evaluate a spec");
            println!("  ir       <spec.proof>                         Dump AST as JSON");
            println!("  verify   <spec.proof> --event json            Single event check");
            println!("  verify   <spec.proof> --batch events.ndjson   Batch verification");
            Ok(())
        }
    }
}

// ─── check ───────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct CheckInput {
    balance:           Decimal,
    #[serde(default = "default_event")]
    event:             String,
    #[serde(default)]
    days_since_joined: Option<u32>,
    #[serde(default)]
    product_count:     Option<u32>,
}
fn default_event() -> String { "daily_accrual".into() }

fn cmd_check(args: &[String]) -> Result<()> {
    let spec_path = args.first()
        .ok_or_else(|| anyhow!("usage: proof check <spec.proof> [--input '{{...}}']"))?;

    let spec = load_spec(spec_path)?;

    let Some(json) = flag(args, "--input") else {
        println!("\nPROOF — parsed successfully\n");
        println!("  Product:      {}", spec.name);
        println!("  Jurisdiction: {:?}", spec.jurisdiction);
        println!("  Regulator:    {:?}", spec.regulator);
        println!("  Category:     {:?}", spec.category);
        if let Some(i) = &spec.interest {
            println!("  Interest:     base_rate={}%  tiers={}", i.base_rate.0, i.tiers.len());
        }
        if spec.fees.is_some()        { println!("  Fees:         yes"); }
        if spec.protection.is_some()  { println!("  Protection:   yes"); }
        if spec.obligations.is_some() { println!("  Obligations:  yes"); }
        println!();
        return Ok(());
    };

    let ci: CheckInput = serde_json::from_str(json)
        .map_err(|e| anyhow!("--input JSON error: {}", e))?;

    let event_type = parse_event_type(&ci.event)?;
    let input = proof_eval::types::EvalInput {
        customer_id:       "cli".into(),
        event_type:        event_type.clone(),
        balance:           ci.balance,
        days_since_joined: ci.days_since_joined,
        product_count:     ci.product_count,
    };

    let output = proof_eval::evaluate(&spec, &input)
        .map_err(|e| anyhow!("evaluation error: {}", e))?;

    println!("\nPROOF v{}\n", env!("CARGO_PKG_VERSION"));
    println!("  Specification: {}", spec.name);
    println!("  Event:         {}", event_type);
    println!("  Balance:       £{}", ci.balance);
    if let Some(d) = ci.days_since_joined { println!("  Member for:    {} days", d); }
    println!();
    if let Some(t) = &output.applied_tier  { println!("  Tier:          {}", t); }
    if let Some(r) = &output.rate_applied  { println!("  Rate:          {}%", r); }
    if let Some(a) = &output.amount        { println!("  Result:        £{}", a); }
    println!();
    for line in &output.reasoning          { println!("  {}", line); }
    println!();
    println!("  Spec check: OK\n");
    Ok(())
}

// ─── verify ──────────────────────────────────────────────────────────────────

fn cmd_verify(args: &[String]) -> Result<()> {
    let spec_path = args.first()
        .ok_or_else(|| anyhow!("usage: proof verify <spec.proof> --event '{{...}}' | --batch file.ndjson"))?;

    let spec = load_spec(spec_path)?;

    // single event
    if let Some(json) = flag(args, "--event") {
        let event: proof_ingest::SystemEvent = serde_json::from_str(json)
            .map_err(|e| anyhow!("--event JSON error: {}", e))?;
        let (input, system_output) = proof_ingest::normalise(event);

        println!("\nPROOF v{}\n", env!("CARGO_PKG_VERSION"));
        println!("  Specification: {}", spec.name);
        println!("  Customer:      {}", input.customer_id);
        println!("  Event:         {}", input.event_type);
        println!("  Balance:       £{}", input.balance);

        match proof_verify::compare(&spec, &input, &system_output)? {
            None    => { println!("\n  Result: OK — matches specification\n"); }
            Some(d) => { proof_verify::alert::TerminalSink.emit(&d); }
        }
        return Ok(());
    }

    // batch file
    if let Some(path) = flag(args, "--batch") {
        let events = proof_ingest::read_batch(path)?;
        let mut divergences = 0usize;

        println!("\nPROOF v{}", env!("CARGO_PKG_VERSION"));
        println!("  Spec:   {}   File: {}\n", spec.name, path);

        for (input, system_output) in &events {
            match proof_verify::compare(&spec, input, system_output)? {
                None    => { println!("  OK  {}  £{}", input.customer_id, input.balance); }
                Some(d) => {
                    proof_verify::alert::TerminalSink.emit(&d);
                    divergences += 1;
                }
            }
        }

        println!();
        println!("  Verified: {}   Divergences: {}", events.len(), divergences);
        if divergences > 0 {
            std::process::exit(1); // non-zero exit lets CI catch it
        }
        println!();
        return Ok(());
    }

    Err(anyhow!("provide --event '{{...}}' or --batch file.ndjson"))
}

// ─── ir ──────────────────────────────────────────────────────────────────────

fn cmd_ir(args: &[String]) -> Result<()> {
    let spec_path = args.first()
        .ok_or_else(|| anyhow!("usage: proof ir <spec.proof>"))?;
    let spec = load_spec(spec_path)?;
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn load_spec(path: &str) -> Result<proof_dsl::ast::ProductSpec> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("cannot read {}: {}", path, e))?;
    proof_dsl::parse(&src).map_err(|e| anyhow!("parse error in {}: {}", path, e))
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].as_str())
}

fn parse_event_type(s: &str) -> Result<proof_eval::types::EventType> {
    Ok(match s {
        "daily_accrual"        => proof_eval::types::EventType::DailyAccrual,
        "monthly_interest"     => proof_eval::types::EventType::MonthlyInterestPayment,
        s if s.starts_with("fee:") => proof_eval::types::EventType::FeeCharge {
            fee_name: s.trim_start_matches("fee:").into(),
        },
        other => return Err(anyhow!("unknown event: '{}' — try: daily_accrual, fee:<name>", other)),
    })
}