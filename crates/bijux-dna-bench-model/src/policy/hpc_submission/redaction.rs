//! Select explicit and conventional sensitive campaign values without acquiring environment state.
use std::collections::BTreeMap;

/// Preserve the campaign redaction key rules, minimum value length and deterministic unique order.
#[must_use]
pub fn select_hpc_redaction_needles(
    env_map: &BTreeMap<String, String>,
    explicit_keys: &[String],
) -> Vec<String> {
    let mut needles = Vec::new();
    for (key, value) in env_map {
        if value.is_empty() {
            continue;
        }
        let uppercase = key.to_ascii_uppercase();
        let by_name = uppercase.contains("SECRET")
            || uppercase.contains("TOKEN")
            || uppercase.contains("PASSWORD")
            || uppercase.contains("KEY")
            || uppercase.contains("ACCOUNT")
            || uppercase.contains("PROJECT");
        let explicitly = explicit_keys.iter().any(|k| k == key);
        if (by_name || explicitly) && value.len() >= 4 {
            needles.push(value.clone());
        }
    }
    needles.sort();
    needles.dedup();
    needles
}
