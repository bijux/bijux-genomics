use anyhow::{anyhow, Context, Result};

pub(crate) fn checked_f64_from_u64(value: u64, context: &str) -> Result<f64> {
    value
        .to_string()
        .parse::<f64>()
        .with_context(|| format!("{context} exceeds the supported floating-point range"))
}

pub(crate) fn checked_f64_from_usize(value: usize, context: &str) -> Result<f64> {
    let value = u64::try_from(value)
        .map_err(|_| anyhow!("{context} exceeds the supported integer range"))?;
    checked_f64_from_u64(value, context)
}

pub(crate) fn rounded_f64_to_u64(value: f64, context: &str) -> Result<u64> {
    if !value.is_finite() {
        return Err(anyhow!("{context} must be finite"));
    }
    if value < 0.0 {
        return Err(anyhow!("{context} must be nonnegative"));
    }

    format!("{value:.0}")
        .parse::<u64>()
        .with_context(|| format!("{context} exceeds the supported integer range"))
}
