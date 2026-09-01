// Phase 7: registry of all loaded regulatory rule libraries.
pub struct RegulatoryRegistry;

impl RegulatoryRegistry {
    pub fn new() -> Self { Self }
}

impl Default for RegulatoryRegistry {
    fn default() -> Self { Self::new() }
}
