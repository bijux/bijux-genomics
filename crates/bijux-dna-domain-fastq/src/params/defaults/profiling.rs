use super::super::stats::{
    FastqOverrepresentedProfileParams, FastqReadLengthProfileParams, FastqStatsParams,
    OVERREPRESENTED_PROFILE_SCHEMA_VERSION, READ_LENGTH_PROFILE_SCHEMA_VERSION,
    STATS_SCHEMA_VERSION,
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
