use bijux_dna_domain_fastq::{STAGE_EXTRACT_UMIS, STAGE_MERGE_PAIRS};

pub(crate) fn apply_layout_branching(mut stages: Vec<String>, paired: bool) -> Vec<String> {
    if paired {
        return stages;
    }
    // Single-end runs must not schedule paired-only stages.
    stages.retain(|stage| {
        stage != STAGE_MERGE_PAIRS.as_str() && stage != STAGE_EXTRACT_UMIS.as_str()
    });
    stages
}
