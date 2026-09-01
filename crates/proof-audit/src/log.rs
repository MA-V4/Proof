use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id:        Uuid,
    pub timestamp: DateTime<Utc>,
    pub spec_name: String,
    pub spec_hash: String,
    pub actor:     String,
    #[serde(flatten)]
    pub kind:      AuditEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditEntryKind {
    SpecLoaded     { source: String },
    Verified       { customer_id: String, ok: bool },
    DivergenceDetected { divergence_id: String },
    DivergenceResolved { divergence_id: String },
    SimulationRun  { verdict: String },
    SpecSignedOff  { approver: String },
}

/// In-memory append-only audit log. Phase 8 persists this to SQLite.
#[derive(Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self { Self::default() }

    pub fn append(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    pub fn for_spec(&self, name: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.spec_name == name).collect()
    }

    pub fn all(&self) -> &[AuditEntry] { &self.entries }
    pub fn total(&self) -> usize { self.entries.len() }
}