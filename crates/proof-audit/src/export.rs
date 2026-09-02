use crate::log::{AuditEntry, AuditEntryKind};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FcaAuditPack {
    pub generated_at: DateTime<Utc>,
    pub spec_name: String,
    pub spec_hash: String,
    pub total_verified: usize,
    pub divergences: usize,
    pub divergences_resolved: usize,
    pub simulations: usize,
    pub sign_offs: Vec<SignOffRecord>,
    pub entries: Vec<AuditEntry>,
}

#[derive(Debug, Serialize)]
pub struct SignOffRecord {
    pub timestamp: DateTime<Utc>,
    pub approver: String,
    pub spec_hash: String,
}

pub fn build_fca_pack(spec_name: &str, entries: &[AuditEntry]) -> FcaAuditPack {
    let spec_entries: Vec<&AuditEntry> = entries
        .iter()
        .filter(|e| e.spec_name == spec_name)
        .collect();

    let spec_hash = spec_entries
        .last()
        .map(|e| e.spec_hash.clone())
        .unwrap_or_default();

    let total_verified = spec_entries
        .iter()
        .filter(|e| matches!(e.kind, AuditEntryKind::Verified { .. }))
        .count();

    let divergences = spec_entries
        .iter()
        .filter(|e| matches!(e.kind, AuditEntryKind::DivergenceDetected { .. }))
        .count();

    let divergences_resolved = spec_entries
        .iter()
        .filter(|e| matches!(e.kind, AuditEntryKind::DivergenceResolved { .. }))
        .count();

    let simulations = spec_entries
        .iter()
        .filter(|e| matches!(e.kind, AuditEntryKind::SimulationRun { .. }))
        .count();

    let sign_offs = spec_entries
        .iter()
        .filter_map(|e| {
            if let AuditEntryKind::SpecSignedOff { ref approver } = e.kind {
                Some(SignOffRecord {
                    timestamp: e.timestamp,
                    approver: approver.clone(),
                    spec_hash: e.spec_hash.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    FcaAuditPack {
        generated_at: Utc::now(),
        spec_name: spec_name.to_string(),
        spec_hash,
        total_verified,
        divergences,
        divergences_resolved,
        simulations,
        sign_offs,
        entries: spec_entries.into_iter().cloned().collect(),
    }
}
