use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id:          Uuid,
    pub timestamp:   DateTime<Utc>,
    pub kind:        AuditEntryKind,
    pub actor:       String,
    pub spec_name:   String,
    pub spec_hash:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditEntryKind {
    SpecPublished  { version: String },
    SpecSignedOff  { version: String, approver: String },
    DivergenceDetected { divergence_id: Uuid },
    DivergenceResolved { divergence_id: Uuid },
    SimulationRun  { old_version: String, new_version: String },
}

pub struct AuditLog;
impl AuditLog {
    pub fn append(&self, _entry: AuditEntry) {
        todo!("Phase 8 — append to immutable audit log")
    }
    pub fn export_fca_pack(&self, _spec_name: &str) -> Vec<u8> {
        todo!("Phase 8 — export regulator audit pack")
    }
}
