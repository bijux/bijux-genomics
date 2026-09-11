use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(super) fn required_string_field(
    payload: &serde_json::Value,
    key: &str,
    path: &Path,
) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{key}`", path.display()))
}

pub(super) fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-12
}
