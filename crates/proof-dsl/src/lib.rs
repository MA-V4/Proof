pub mod ast;
pub mod diff;
pub mod error;
pub mod lexer;
pub mod parser;

pub use ast::ProductSpec;
pub use diff::{diff_specs, DiffItem};
pub use error::ParseError;
pub use parser::parse;
