//! postprocess execution timing.
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) fn timestamp_marker() -> anyhow::Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}
