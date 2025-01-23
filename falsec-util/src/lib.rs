use std::hash::{DefaultHasher, Hash, Hasher};

pub fn string_id(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
