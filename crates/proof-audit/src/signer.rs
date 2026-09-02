use sha2::{Digest, Sha256};

/// SHA-256 hash of spec source text. Returns lowercase hex.
pub fn hash_spec(source: &str) -> String {
    hex::encode(Sha256::digest(source.as_bytes()))
}

/// First 8 chars of the hash for display - same format as git short SHA.
pub fn short_hash(full: &str) -> &str {
    &full[..8.min(full.len())]
}
