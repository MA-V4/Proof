// Phase 5 deliverable.
use chrono::{DateTime, Utc};
use proof_dsl::ast::ProductSpec;
use proof_verify::Divergence;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub specs:           HashMap<String, ProductSpec>,
    pub divergences:     Vec<Divergence>,
    pub events_verified: u64,
    pub recent:          VecDeque<RecentEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentEvent {
    pub customer_id: String,
    pub spec_name:   String,
    pub event_type:  String,
    pub ok:          bool,
    pub timestamp:   DateTime<Utc>,
}

impl AppState {
    pub fn load_from_dir(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        let mut specs = HashMap::new();

        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path.extension().map_or(false, |e| e == "proof") {
                    let src = std::fs::read_to_string(&path)?;
                    match proof_dsl::parse(&src) {
                        Ok(spec) => {
                            tracing::info!("loaded spec: {}", spec.name);
                            specs.insert(spec.name.clone(), spec);
                        }
                        Err(e) => tracing::warn!("skipping {:?}: {}", path, e),
                    }
                }
            }
        }

        tracing::info!("loaded {} spec(s)", specs.len());
        Ok(Self {
            specs,
            divergences:     Vec::new(),
            events_verified: 0,
            recent:          VecDeque::new(),
        })
    }

    pub fn push_event(&mut self, event: RecentEvent) {
        self.events_verified += 1;
        self.recent.push_front(event);
        if self.recent.len() > 100 { self.recent.pop_back(); }
    }
}