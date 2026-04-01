#[must_use]
pub fn snapshot_name(bucket: &str, test_name: &str) -> String {
    let pkg =
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string());
    format!("{pkg}__{bucket}__{test_name}")
}
