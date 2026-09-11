use super::CorpusRunError;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
/// Format an explicit publication instant using the run contract's RFC 3339 UTC representation.
///
/// # Errors
/// Rejects instants before the Unix epoch or outside the supported calendar range.
pub fn format_generated_at_utc(instant: SystemTime) -> Result<String, CorpusRunError> {
    let elapsed = instant
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CorpusRunError::invalid("resolve benchmark timestamp".into()))?;
    let seconds = i64::try_from(elapsed.as_secs()).map_err(|_| {
        CorpusRunError::invalid("benchmark timestamp is outside the supported range".into())
    })?;
    let timestamp =
        DateTime::<Utc>::from_timestamp(seconds, elapsed.subsec_nanos()).ok_or_else(|| {
            CorpusRunError::invalid("benchmark timestamp is outside the supported range".into())
        })?;
    Ok(timestamp.to_rfc3339_opts(SecondsFormat::Nanos, true))
}
pub(super) fn validate_utc_timestamp(value: &str, label: &str) -> Result<(), CorpusRunError> {
    let parsed = DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .map_err(|_| CorpusRunError::invalid(format!("{label} timestamp is not RFC 3339")))?;
    if parsed.offset().local_minus_utc() != 0 {
        return Err(CorpusRunError::invalid(format!("{label} timestamp is not UTC")));
    }
    Ok(())
}
