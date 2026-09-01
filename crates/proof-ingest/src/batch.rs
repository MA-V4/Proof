use crate::normaliser::{normalise, SystemEvent};
use proof_eval::types::{EvalInput, EvalOutput};
use anyhow::{Context, Result};
use std::path::Path;

/// Read an NDJSON file and return (input, system_output) pairs.
/// One JSON object per line. Blank lines and // comments are skipped.
pub fn read_batch(path: impl AsRef<Path>) -> Result<Vec<(EvalInput, EvalOutput)>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") { continue; }
        let event: SystemEvent = serde_json::from_str(line)
            .with_context(|| format!("{} line {}: invalid JSON", path.display(), i + 1))?;
        out.push(normalise(event));
    }
    Ok(out)
}