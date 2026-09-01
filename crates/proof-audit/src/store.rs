use crate::log::AuditEntry;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct AuditStore {
    path: PathBuf,
}

impl AuditStore {
    pub fn open(path: impl AsRef<Path>) -> Self {
        Self { path: path.as_ref().to_owned() }
    }

    pub fn load(&self) -> Vec<AuditEntry> {
        let Ok(raw) = std::fs::read_to_string(&self.path) else { return Vec::new() };
        let content = raw.strip_prefix('\u{FEFF}').unwrap_or(&raw);
        content.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }

    pub fn append(&self, entry: &AuditEntry) {
        if let Ok(json) = serde_json::to_string(entry) {
            let _ = std::fs::OpenOptions::new()
                .create(true).append(true)
                .open(&self.path)
                .and_then(|mut f| writeln!(f, "{}", json));
        }
    }
}