use crate::commands::cli::parse::{FastqTrimArgs, FastqValidateArgs};

#[must_use]
pub fn is_bench_requested_trim(args: &FastqTrimArgs) -> bool {
    args.sample_id.is_some() && args.r1.is_some() && args.out.is_some()
}

#[must_use]
pub fn is_bench_requested_validate(args: &FastqValidateArgs) -> bool {
    args.sample_id.is_some() && args.r1.is_some() && args.out.is_some()
}
