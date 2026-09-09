//! Adapt shared count conversion errors to command diagnostics.

use anyhow::Result;
use bijux_dna_core::prelude::numeric;

pub(crate) fn checked_f64_from_u64(value: u64, context: &str) -> Result<f64> {
    Ok(numeric::checked_f64_from_u64(value, context)?)
}

pub(crate) fn checked_f64_from_usize(value: usize, context: &str) -> Result<f64> {
    Ok(numeric::checked_f64_from_usize(value, context)?)
}

pub(crate) fn rounded_f64_to_u64(value: f64, context: &str) -> Result<u64> {
    Ok(numeric::rounded_f64_to_u64(value, context)?)
}
