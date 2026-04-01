use bijux_dna_core::ids::StageId;

use crate::EffectiveDefaults;

pub(super) fn apply(defaults: &mut EffectiveDefaults) {
    defaults.rationales.insert(
        StageId::from_static("fastq.trim_reads"),
        "aDNA preset: short-read preserving trim with strict adapter handling".to_string(),
    );
    defaults.rationales.insert(
        StageId::from_static("fastq.merge_pairs"),
        "aDNA preset: aggressive overlap merge/collapse for fragmented paired-end reads"
            .to_string(),
    );
    defaults.rationales.insert(
        StageId::from_static("fastq.detect_adapters"),
        "aDNA preset: stricter adapter detection depth for short fragments".to_string(),
    );
}
