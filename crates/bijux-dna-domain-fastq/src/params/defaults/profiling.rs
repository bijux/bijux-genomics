use super::super::stats::{
    FastqEstimateLibraryComplexityPrealignParams, FastqOverrepresentedProfileParams,
    FastqReadLengthProfileParams, FastqStatsParams,
    ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SCHEMA_VERSION, OVERREPRESENTED_PROFILE_SCHEMA_VERSION,
    READ_LENGTH_PROFILE_SCHEMA_VERSION, STATS_SCHEMA_VERSION,
};
use super::shared::paired_mode;

#[must_use]
pub fn stats_defaults(paired: bool) -> FastqStatsParams {
    FastqStatsParams {
        schema_version: STATS_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 2,
    }
}

#[must_use]
pub fn read_length_profile_defaults(paired: bool) -> FastqReadLengthProfileParams {
    FastqReadLengthProfileParams {
        schema_version: READ_LENGTH_PROFILE_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 2,
        histogram_bins: 100,
    }
}

#[must_use]
pub fn overrepresented_profile_defaults(paired: bool) -> FastqOverrepresentedProfileParams {
    FastqOverrepresentedProfileParams {
        schema_version: OVERREPRESENTED_PROFILE_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 2,
        top_k: 50,
    }
}

#[must_use]
pub fn estimate_library_complexity_prealign_defaults(
    paired: bool,
) -> FastqEstimateLibraryComplexityPrealignParams {
    FastqEstimateLibraryComplexityPrealignParams {
        schema_version: ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        complexity_policy: "prealign_kmer".to_string(),
        estimate_method: "kmer_redundancy".to_string(),
        advisory_only: true,
        modifies_reads: false,
        kmer_size: Some(31),
    }
}
