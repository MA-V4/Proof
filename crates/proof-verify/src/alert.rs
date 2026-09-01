use crate::divergence::Divergence;

pub trait AlertSink: Send + Sync {
    fn emit(&self, d: &Divergence);
}

/// Prints a formatted divergence report to stderr.
pub struct TerminalSink;

impl AlertSink for TerminalSink {
    fn emit(&self, d: &Divergence) {
        eprintln!();
        eprintln!("  ╔══ DIVERGENCE ═══════════════════════════════════════╗");
        eprintln!("  ║  Spec:      {}", d.spec_name);
        eprintln!("  ║  Customer:  {}   Balance: £{}", d.customer_id, d.balance);
        eprintln!("  ║  Event:     {}", d.event_type);
        eprintln!("  ╠══════════════════════════════════════════════════════╣");
        eprintln!("  ║  {:<20}  {:<12}  {:<12}  {}", "Field", "Spec", "System", "Delta");
        eprintln!("  ║  {}", "─".repeat(54));
        for diff in &d.diffs {
            let delta = diff.delta.map(|v| format!("{:+}", v)).unwrap_or_default();
            eprintln!("  ║  {:<20}  {:<12}  {:<12}  {}", diff.field, diff.spec_value, diff.system_value, delta);
        }
        eprintln!("  ╚══════════════════════════════════════════════════════╝");
        eprintln!();
    }
}

/// Writes each divergence as a JSON line — pipe to a file or another tool.
pub struct JsonSink;

impl AlertSink for JsonSink {
    fn emit(&self, d: &Divergence) {
        println!("{}", serde_json::to_string(d).unwrap_or_default());
    }
}