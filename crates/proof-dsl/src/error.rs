use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token at line {line}: {message}")]
    UnexpectedToken { line: usize, message: String },
    #[error("unknown keyword: {0}")]
    UnknownKeyword(String),
    #[error("invalid rate format: {0}")]
    InvalidRate(String),
    #[error("invalid money format: {0}")]
    InvalidMoney(String),
    #[error("{0}")]
    Other(String),
}
