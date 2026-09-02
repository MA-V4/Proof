use proof_dsl::ParseError;

pub fn validate_draft(spec_text: &str) -> Result<proof_dsl::ast::ProductSpec, ParseError> {
    proof_dsl::parse(spec_text)
}
