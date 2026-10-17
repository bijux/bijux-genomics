//! Pure rejection decisions for the native local read-filter envelope.
//! Callers retain record acquisition, paired retention and report publication.

use crate::params::filter::FilterEffectiveParams;

/// Reason a read is rejected by the native local filter rule.
///
/// Stability: stable domain contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadFilterRejection {
    /// The count of uppercase or lowercase N bytes exceeds the selected limit.
    TooManyN,
    /// The byte-transition percentage is below the selected complexity threshold.
    LowComplexity,
}

/// Decide whether a sequence is rejected by the native local read-filter rule.
///
/// `max_n_count` takes precedence over `max_n`; the N-count rule runs before
/// complexity. `low_complexity_threshold` takes precedence over `entropy_threshold`.
/// Complexity is the percentage of adjacent bytes that differ, with zero for
/// zero or one byte. It is a local transition proxy, not Shannon entropy.
/// Thresholds retain direct floating-point comparison behavior, including NaN.
/// Other parameters remain metadata for this rule. This function performs no IO.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn local_read_filter_rejection(
    sequence: &str,
    effective_params: &FilterEffectiveParams,
) -> Option<ReadFilterRejection> {
    let max_n_count = effective_params.max_n_count.or(effective_params.max_n);
    if let Some(limit) = max_n_count {
        let n_count = sequence.bytes().filter(|base| matches!(*base, b'N' | b'n')).count() as u32;
        if n_count > limit {
            return Some(ReadFilterRejection::TooManyN);
        }
    }

    if let Some(threshold) =
        effective_params.low_complexity_threshold.or(effective_params.entropy_threshold)
    {
        let complexity = local_complexity_score(sequence);
        if complexity < threshold {
            return Some(ReadFilterRejection::LowComplexity);
        }
    }

    None
}

#[allow(clippy::cast_precision_loss)]
fn local_complexity_score(sequence: &str) -> f64 {
    let bytes = sequence.as_bytes();
    if bytes.len() <= 1 {
        return 0.0;
    }
    let transitions = bytes.windows(2).filter(|window| window[0] != window[1]).count() as f64;
    (transitions / (bytes.len() as f64 - 1.0)) * 100.0
}

#[cfg(test)]
mod tests {
    use super::{local_read_filter_rejection, ReadFilterRejection};
    use crate::params::{filter::FilterEffectiveParams, DamageMode, PairedMode};

    #[test]
    fn local_rejection_preserves_alias_precedence_and_transition_boundaries() {
        use ReadFilterRejection::{LowComplexity, TooManyN};

        let params = FilterEffectiveParams {
            paired_mode: PairedMode::SingleEnd,
            threads: 1,
            max_n: None,
            max_n_fraction: None,
            max_n_count: None,
            low_complexity_threshold: None,
            entropy_threshold: None,
            contaminant_db: None,
            n_policy: None,
            polyx_policy: None,
            damage_mode: None,
        };
        // Label, sequence, count limit, legacy count, complexity, legacy complexity, rejection.
        let cases = [
            ("unset", "NnNN", None, None, None, None, None),
            ("count equality", "Nn", Some(2), None, None, None, None),
            ("count excess", "Nn", Some(1), None, None, None, Some(TooManyN)),
            ("count preferred", "NN", Some(3), Some(0), None, None, None),
            ("count stricter", "NN", Some(0), Some(3), None, None, Some(TooManyN)),
            ("legacy count", "N", None, Some(0), None, None, Some(TooManyN)),
            ("N first", "NNNN", Some(0), None, Some(100.0), None, Some(TooManyN)),
            ("complexity preferred", "AAAA", None, None, Some(0.0), Some(100.0), None),
            (
                "complexity stricter",
                "ACGT",
                None,
                None,
                Some(101.0),
                Some(0.0),
                Some(LowComplexity),
            ),
            ("legacy complexity", "AAAA", None, None, None, Some(50.0), Some(LowComplexity)),
            ("transition equality", "AAC", None, None, Some(50.0), None, None),
            ("transition lower", "AAAC", None, None, Some(50.0), None, Some(LowComplexity)),
            ("alternating", "ATATATAT", None, None, Some(100.0), None, None),
            ("grouped", "AAAATTTT", None, None, Some(100.0), None, Some(LowComplexity)),
            ("empty equality", "", None, None, Some(0.0), None, None),
            ("empty lower", "", None, None, Some(0.1), None, Some(LowComplexity)),
            ("single equality", "A", None, None, Some(0.0), None, None),
            ("single lower", "A", None, None, Some(0.1), None, Some(LowComplexity)),
            ("UTF-8 bytes", "éé", None, None, Some(100.0), None, None),
            ("case bytes", "AaAa", None, None, Some(100.0), None, None),
            ("NaN preferred", "AAAA", None, None, Some(f64::NAN), Some(100.0), None),
            ("NaN legacy", "AAAA", None, None, None, Some(f64::NAN), None),
            ("infinite upper", "ACGT", None, None, Some(f64::INFINITY), None, Some(LowComplexity)),
            ("infinite lower", "AAAA", None, None, Some(f64::NEG_INFINITY), None, None),
            ("negative zero", "AAAA", None, None, Some(-0.0), None, None),
        ];
        for (label, sequence, max_n_count, max_n, complexity, entropy, expected) in cases {
            let effective_params = FilterEffectiveParams {
                max_n_count,
                max_n,
                low_complexity_threshold: complexity,
                entropy_threshold: entropy,
                ..params.clone()
            };
            assert_eq!(
                local_read_filter_rejection(sequence, &effective_params),
                expected,
                "{label}"
            );
        }

        let metadata_only = FilterEffectiveParams {
            paired_mode: PairedMode::PairedEnd,
            threads: 64,
            max_n_fraction: Some(0.0),
            contaminant_db: Some("not-acquired-by-the-decision".to_string()),
            n_policy: Some("keep".to_string()),
            polyx_policy: Some("trim".to_string()),
            damage_mode: Some(DamageMode::Ancient),
            ..params
        };
        assert_eq!(local_read_filter_rejection("NNNN", &metadata_only), None);
    }
}
