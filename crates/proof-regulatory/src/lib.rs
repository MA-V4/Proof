pub mod cfpb;
pub mod fca;
pub mod pra;
pub mod registry;
pub mod types;

pub use registry::{RegulatoryRegistry, Verdict};
pub use types::{CheckInput, RegulatoryFlag, Severity};