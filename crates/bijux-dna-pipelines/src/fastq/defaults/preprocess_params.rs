use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::defaults::{
    correct_defaults, detect_adapters_defaults, filter_defaults, merge_defaults, trim_defaults,
    trim_polyg_tails_defaults, trim_terminal_damage_defaults, umi_defaults, validate_defaults,
};

use crate::DefaultParams;

pub(super) fn fastq_preprocess_params(paired: bool) -> BTreeMap<StageId, DefaultParams> {
    BTreeMap::from([
        (
            StageId::from_static("fastq.validate_reads"),
            DefaultParams::FastqValidate(validate_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.correct_errors"),
            DefaultParams::FastqCorrect(correct_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.extract_umis"),
            DefaultParams::FastqUmi(umi_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.detect_adapters"),
            DefaultParams::FastqDetectAdapters(detect_adapters_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.trim_reads"),
            DefaultParams::FastqTrim(trim_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.trim_polyg_tails"),
            DefaultParams::FastqTrimPolygTails(trim_polyg_tails_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.trim_terminal_damage"),
            DefaultParams::FastqTrimTerminalDamage(trim_terminal_damage_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.filter_reads"),
            DefaultParams::FastqFilter(filter_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.merge_pairs"),
            DefaultParams::FastqMerge(merge_defaults(paired)),
        ),
    ])
}
