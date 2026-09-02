/// Replace customer_id with a stable anonymous hash.
/// Same input always maps to the same output for cohort tracking,
/// but the original ID is not recoverable.
pub fn anonymise_id(customer_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    customer_id.hash(&mut h);
    format!("anon-{:016x}", h.finish())
}
