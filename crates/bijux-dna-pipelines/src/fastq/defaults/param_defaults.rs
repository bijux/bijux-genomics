use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::defaults::{
    correct_defaults, detect_adapters_defaults, filter_defaults, merge_defaults,
    overrepresented_profile_defaults, qc_post_defaults, read_length_profile_defaults,
    screen_defaults, stats_defaults, trim_defaults, trim_polyg_tails_defaults,
    trim_terminal_damage_defaults, umi_defaults, validate_defaults,
};

use crate::DefaultParams;

pub(super) fn fastq_default_params(paired: bool) -> BTreeMap<StageId, DefaultParams> {
    let mut params = BTreeMap::new();
    params.insert(
        StageId::from_static("fastq.validate_reads"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.profile_reads"),
        DefaultParams::FastqStats(stats_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.profile_read_lengths"),
        DefaultParams::FastqReadLengthProfile(read_length_profile_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.correct_errors"),
        DefaultParams::FastqCorrect(correct_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.extract_umis"),
        DefaultParams::FastqUmi(umi_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.detect_adapters"),
        DefaultParams::FastqDetectAdapters(detect_adapters_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim_reads"),
        DefaultParams::FastqTrim(trim_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim_polyg_tails"),
        DefaultParams::FastqTrimPolygTails(trim_polyg_tails_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim_terminal_damage"),
        DefaultParams::FastqTrimTerminalDamage(trim_terminal_damage_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.filter_reads"),
        DefaultParams::FastqFilter(filter_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        DefaultParams::FastqOverrepresentedProfile(overrepresented_profile_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.report_qc"),
        DefaultParams::FastqQcPost(qc_post_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.merge_pairs"),
        DefaultParams::FastqMerge(merge_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.screen_taxonomy"),
        DefaultParams::FastqScreen(screen_defaults(paired)),
    );
    params
}
