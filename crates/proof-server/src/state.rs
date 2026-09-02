use chrono::{DateTime, Utc};
use proof_audit::{AuditEntry, AuditEntryKind};
use proof_dsl::ast::ProductSpec;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::Db;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub specs: HashMap<String, ProductSpec>,
    #[allow(dead_code)]
    pub spec_sources: HashMap<String, String>,
    pub spec_hashes: HashMap<String, String>,
    pub events_verified: u64,
    pub recent: VecDeque<RecentEvent>,
    pub db: Db,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentEvent {
    pub customer_id: String,
    pub spec_name: String,
    pub event_type: String,
    pub ok: bool,
    pub timestamp: DateTime<Utc>,
}

impl AppState {
    pub async fn load(dir: impl AsRef<Path>, db: Db) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        let mut specs = HashMap::new();
        let mut spec_sources = HashMap::new();
        let mut spec_hashes = HashMap::new();

        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path.extension().map_or(false, |e| e == "proof") {
                    let src = std::fs::read_to_string(&path)?;
                    match proof_dsl::parse(&src) {
                        Ok(spec) => {
                            let hash = proof_audit::hash_spec(&src);
                            tracing::info!(
                                "loaded spec: {} ({})",
                                spec.name,
                                proof_audit::short_hash(&hash)
                            );
                            let audit_entry = AuditEntry {
                                id: Uuid::new_v4(),
                                timestamp: Utc::now(),
                                spec_name: spec.name.clone(),
                                spec_hash: hash.clone(),
                                actor: "system".into(),
                                kind: AuditEntryKind::SpecLoaded {
                                    source: path.file_name().unwrap().to_string_lossy().into(),
                                },
                            };
                            let _ = db.insert_audit(&audit_entry).await;
                            spec_sources.insert(spec.name.clone(), src);
                            spec_hashes.insert(spec.name.clone(), hash);
                            specs.insert(spec.name.clone(), spec);
                        }
                        Err(e) => tracing::warn!("skipping {:?}: {}", path, e),
                    }
                }
            }
        }

        let events_verified = db.count_events().await.unwrap_or(0) as u64;
        let recent_vec = db.get_recent_events(100).await.unwrap_or_default();
        let recent = recent_vec.into_iter().collect();

        tracing::info!(
            "loaded {} spec(s), {} events in history",
            specs.len(),
            events_verified
        );

        Ok(Self {
            specs,
            spec_sources,
            spec_hashes,
            events_verified,
            recent,
            db,
        })
    }

    pub fn push_recent(&mut self, event: RecentEvent) {
        self.events_verified += 1;
        self.recent.push_front(event);
        if self.recent.len() > 100 {
            self.recent.pop_back();
        }
    }
}
