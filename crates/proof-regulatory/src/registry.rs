use crate::types::{CheckInput, RegulatoryFlag, Severity};

pub struct RegulatoryRegistry;

impl RegulatoryRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self, input: &CheckInput) -> Vec<RegulatoryFlag> {
        let mut flags = Vec::new();
        flags.extend(crate::fca::check(input));
        // pra::check and cfpb::check added when those rule libraries are built out
        flags
    }

    /// Derive an overall verdict from the flags.
    /// Block > Review > Clean.
    pub fn verdict(&self, flags: &[RegulatoryFlag]) -> Verdict {
        if flags.iter().any(|f| f.severity == Severity::Block) {
            return Verdict::DoNotDeploy;
        }
        if flags.iter().any(|f| f.severity == Severity::Review) {
            return Verdict::DeployWithReview;
        }
        Verdict::DeployClean
    }
}

impl Default for RegulatoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Verdict {
    DeployClean,
    DeployWithReview,
    DoNotDeploy,
}
