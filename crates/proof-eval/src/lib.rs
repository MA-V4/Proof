// Execution engine.
// Given a ProductSpec and an EvalInput, deterministically computes the correct output.
// This is what PROOF compares against your system's actual output.

pub mod evaluator;
pub mod fees;
pub mod interest;
pub mod types;

pub use evaluator::evaluate;
pub use types::{EvalInput, EvalOutput, EventType};
