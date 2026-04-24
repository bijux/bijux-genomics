use std::collections::BTreeSet;

use crate::PipelineProfile;
use bijux_dna_core::prelude::id_catalog;

pub(super) const CORE_FASTQ_STAGES: [&str; 5] = [
    id_catalog::FASTQ_VALIDATE_PRE,
    id_catalog::FASTQ_DETECT_ADAPTERS,
    id_catalog::FASTQ_TRIM,
    id_catalog::FASTQ_FILTER,
    id_catalog::FASTQ_QC_POST,
];

pub(super) fn stage_set(profile: &PipelineProfile) -> BTreeSet<&str> {
    profile.capabilities.required_stages.iter().map(String::as_str).collect()
}
