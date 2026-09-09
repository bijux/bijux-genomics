//! IO-independent numeric conversions shared by request and evidence adapters.

use thiserror::Error;

/// A count conversion rejected an observation or exceeded its numeric range.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct NumericConversionError {
    message: String,
}

type Result<T> = std::result::Result<T, NumericConversionError>;

/// Convert an unsigned count to floating point using the established decimal conversion.
/// Counts above the exact integer range of f64 may be rounded.
///
/// # Errors
/// Returns a contextual error if the decimal count cannot be represented.
pub fn checked_f64_from_u64(value: u64, context: &str) -> Result<f64> {
    value.to_string().parse::<f64>().map_err(|_| NumericConversionError {
        message: format!("{context} exceeds the supported floating-point range"),
    })
}

/// Convert a platform-sized count through the unsigned count representation.
/// Counts above the exact integer range of f64 may be rounded.
///
/// # Errors
/// Rejects values outside the supported integer or floating-point range.
pub fn checked_f64_from_usize(value: usize, context: &str) -> Result<f64> {
    let value = u64::try_from(value).map_err(|_| NumericConversionError {
        message: format!("{context} exceeds the supported integer range"),
    })?;
    checked_f64_from_u64(value, context)
}

/// Round a finite non-negative observation to an unsigned count.
/// Decimal formatting determines ties; the resulting integer must fit in u64.
///
/// # Errors
/// Rejects non-finite, negative, or out-of-range observations.
pub fn rounded_f64_to_u64(value: f64, context: &str) -> Result<u64> {
    if !value.is_finite() {
        return Err(NumericConversionError { message: format!("{context} must be finite") });
    }
    if value < 0.0 {
        return Err(NumericConversionError { message: format!("{context} must be nonnegative") });
    }

    format!("{value:.0}").parse::<u64>().map_err(|_| NumericConversionError {
        message: format!("{context} exceeds the supported integer range"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_conversions_preserve_zero_small_and_large_inputs() -> Result<()> {
        assert_eq!(checked_f64_from_u64(0, "count")?.to_bits(), 0.0_f64.to_bits());
        assert_eq!(checked_f64_from_usize(17, "count")?.to_bits(), 17.0_f64.to_bits());
        assert_eq!(
            checked_f64_from_u64(u64::MAX, "count")?.to_bits(),
            18_446_744_073_709_551_616.0_f64.to_bits()
        );
        Ok(())
    }

    #[test]
    fn observed_counts_preserve_decimal_rounding() -> Result<()> {
        for (value, expected) in [(0.0, 0), (0.5, 0), (1.5, 2), (2.5, 2), (3.6, 4)] {
            assert_eq!(rounded_f64_to_u64(value, "reads")?, expected);
        }
        Ok(())
    }

    #[test]
    fn observed_counts_reject_invalid_or_overflowing_numbers() {
        for value in
            [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 18_446_744_073_709_551_616.0]
        {
            assert!(rounded_f64_to_u64(value, "reads").is_err());
        }
    }
}
