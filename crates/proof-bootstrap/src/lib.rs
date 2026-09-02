pub mod ai;
pub mod detector;
pub mod extractor;
pub mod renderer;
pub mod validator;

pub use detector::{detect_language, Language};
pub use extractor::{extract_heuristic, ExtractedSpec};
pub use renderer::render_spec;
pub use validator::validate_draft;

use anyhow::Result;
use std::path::Path;

pub struct BootstrapResult {
    pub spec_text: String,
    pub language: String,
    pub valid: bool,
    pub parse_error: Option<String>,
    pub used_ai: bool,
}

/// Zero-cost heuristic extraction.
pub fn bootstrap(source_path: &Path) -> Result<BootstrapResult> {
    let source = std::fs::read_to_string(source_path)?;
    let language = detect_language(source_path);
    let extracted = extract_heuristic(&source, &language);
    let spec_text = render_spec(&extracted);
    let valid = validate_draft(&spec_text);

    Ok(BootstrapResult {
        language: language.to_string(),
        parse_error: valid.as_ref().err().map(|e| e.to_string()),
        valid: valid.is_ok(),
        spec_text,
        used_ai: false,
    })
}

/// AI-assisted extraction via Claude API.
pub fn bootstrap_with_ai(source_path: &Path, api_key: &str) -> Result<BootstrapResult> {
    let source = std::fs::read_to_string(source_path)?;
    let language = detect_language(source_path);
    let spec_text = ai::extract_with_claude(&source, &language, api_key)?;
    let valid = validate_draft(&spec_text);

    Ok(BootstrapResult {
        language: language.to_string(),
        parse_error: valid.as_ref().err().map(|e| e.to_string()),
        valid: valid.is_ok(),
        spec_text,
        used_ai: true,
    })
}
