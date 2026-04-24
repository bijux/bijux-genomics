//! Core helpers for v1.

pub use crate::v1::run::{run_dir, DryRunExecutor, Executor};
pub use bijux_dna_runtime::manifests::load_manifests;
pub use bijux_dna_runtime::run::{load_profile, new_run_id};

/// Return the current UTC instant in RFC 3339 format.
#[must_use]
pub fn current_utc_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Return the current UTC calendar date in ISO 8601 format.
#[must_use]
pub fn current_utc_date() -> String {
    chrono::Utc::now().date_naive().to_string()
}

/// Parse an RFC 3339 timestamp into a Unix timestamp in seconds.
#[must_use]
pub fn parse_rfc3339_timestamp_to_unix_seconds(raw: &str) -> Option<i64> {
    let normalized = raw.trim().replace('Z', "+00:00");
    if normalized.is_empty() {
        return None;
    }
    chrono::DateTime::parse_from_rfc3339(&normalized).ok().map(|value| value.timestamp())
}

#[cfg(test)]
mod tests {
    use super::parse_rfc3339_timestamp_to_unix_seconds;

    #[test]
    fn parse_rfc3339_timestamp_accepts_utc_z_suffix() {
        assert_eq!(
            parse_rfc3339_timestamp_to_unix_seconds("2026-03-30T12:34:56Z"),
            Some(1_774_874_096)
        );
    }

    #[test]
    fn parse_rfc3339_timestamp_rejects_blank_values() {
        assert_eq!(parse_rfc3339_timestamp_to_unix_seconds("   "), None);
    }
}
