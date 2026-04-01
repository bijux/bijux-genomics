use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::defaults::{
    overrepresented_profile_defaults, qc_post_defaults, read_length_profile_defaults,
    screen_defaults, stats_defaults,
};

use crate::DefaultParams;

pub(super) fn fastq_analysis_params(paired: bool) -> BTreeMap<StageId, DefaultParams> {
    BTreeMap::from([
        (
            StageId::from_static("fastq.profile_reads"),
            DefaultParams::FastqStats(stats_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.profile_read_lengths"),
            DefaultParams::FastqReadLengthProfile(read_length_profile_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.profile_overrepresented_sequences"),
            DefaultParams::FastqOverrepresentedProfile(overrepresented_profile_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.report_qc"),
            DefaultParams::FastqQcPost(qc_post_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            DefaultParams::FastqScreen(screen_defaults(paired)),
        ),
    ])
}
