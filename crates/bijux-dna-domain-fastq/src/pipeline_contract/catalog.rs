use bijux_dna_core::ids::StageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FastqPipelineMode {
    Shotgun,
    Amplicon,
}

impl FastqPipelineMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shotgun => "shotgun",
            Self::Amplicon => "amplicon",
        }
    }
}

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_polyg_tails"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.correct_errors"),
        StageId::from_static("fastq.extract_umis"),
        StageId::from_static("fastq.index_reference"),
        StageId::from_static("fastq.deplete_host"),
        StageId::from_static("fastq.deplete_reference_contaminants"),
        StageId::from_static("fastq.deplete_rrna"),
        StageId::from_static("fastq.merge_pairs"),
        StageId::from_static("fastq.remove_duplicates"),
        StageId::from_static("fastq.filter_low_complexity"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.screen_taxonomy"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn canonical_amplicon_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.normalize_primers"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.correct_errors"),
        StageId::from_static("fastq.extract_umis"),
        StageId::from_static("fastq.remove_chimeras"),
        StageId::from_static("fastq.cluster_otus"),
        StageId::from_static("fastq.normalize_abundance"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.screen_taxonomy"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn default_shotgun_preprocess_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_polyg_tails"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn default_amplicon_preprocess_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.normalize_primers"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.remove_chimeras"),
        StageId::from_static("fastq.cluster_otus"),
        StageId::from_static("fastq.normalize_abundance"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    vec![
        (
            StageId::from_static("fastq.merge_pairs"),
            vec![
                StageId::from_static("fastq.trim_reads"),
                StageId::from_static("fastq.filter_reads"),
            ],
        ),
        (
            StageId::from_static("fastq.correct_errors"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.extract_umis"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.report_qc"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
    ]
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(StageId, StageId)> {
    vec![
        (
            StageId::from_static("fastq.validate_reads"),
            StageId::from_static("fastq.merge_pairs"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.trim_reads"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.filter_reads"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.merge_pairs"),
        ),
        (
            StageId::from_static("fastq.infer_asvs"),
            StageId::from_static("fastq.cluster_otus"),
        ),
        (
            StageId::from_static("fastq.cluster_otus"),
            StageId::from_static("fastq.infer_asvs"),
        ),
    ]
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    match stage_id.as_str() {
        "fastq.validate_reads"
        | "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.filter_reads"
        | "fastq.normalize_primers"
        | "fastq.remove_chimeras"
        | "fastq.cluster_otus"
        | "fastq.normalize_abundance"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.report_qc" => Some(StageCriticality::Essential),
        "fastq.index_reference"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.filter_low_complexity"
        | "fastq.correct_errors"
        | "fastq.extract_umis"
        | "fastq.screen_taxonomy" => Some(StageCriticality::Optional),
        "fastq.infer_asvs" => Some(StageCriticality::Experimental),
        _ => None,
    }
}
