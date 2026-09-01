pub mod ast;
pub mod diff;
pub mod error;
pub mod lexer;
pub mod parser;

pub use ast::ProductSpec;
pub use diff::{diff_specs, DiffItem};
pub use parser::parse;
pub use error::ParseError;