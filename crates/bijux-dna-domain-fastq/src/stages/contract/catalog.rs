use super::super::{FastqStage, FastqStageContract};
use crate::types::FastqArtifactKind;

const PE: &[FastqArtifactKind] = &[FastqArtifactKind::PairedEnd];
const SE_OR_PE: &[FastqArtifactKind] =
    &[FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd];
const SE_PE_OR_MERGED: &[FastqArtifactKind] =
    &[FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd, FastqArtifactKind::Merged];
const STATS_ONLY: &[FastqArtifactKind] = &[FastqArtifactKind::StatsOnly];
const SE_OR_PE_OUT: &[FastqArtifactKind] =
    &[FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd];
const MERGED_OR_PE_OUT: &[FastqArtifactKind] =
    &[FastqArtifactKind::Merged, FastqArtifactKind::PairedEnd];
const REF_FASTA: &[FastqArtifactKind] = &[FastqArtifactKind::ReferenceFasta];
const REF_INDEX: &[FastqArtifactKind] = &[FastqArtifactKind::ReferenceIndex];
const AMPLICON_TABLE: &[FastqArtifactKind] = &[FastqArtifactKind::AmpliconTable];
const STATS_OR_TAXONOMY: &[FastqArtifactKind] =
    &[FastqArtifactKind::StatsOnly, FastqArtifactKind::TaxonomyMapping];
const AMPLICON_TABLE_OR_REPRESENTATIVES: &[FastqArtifactKind] =
    &[FastqArtifactKind::AmpliconTable, FastqArtifactKind::RepresentativeFasta];

fn single_end_transform_contract() -> FastqStageContract {
    FastqStageContract {
        input_kind: FastqArtifactKind::SingleEnd,
        output_kind: FastqArtifactKind::SingleEnd,
        accepted_input_kinds: SE_OR_PE,
        possible_output_kinds: SE_OR_PE_OUT,
        may_drop_reads: true,
        must_preserve_pairing: true,
        emits_fastq: true,
        preserves: &["read_order", "pairing_metadata"],
        may_drop: &["reads", "bases"],
        retention_definition: "reads_out / reads_in; bases_out / bases_in",
        retention_units: "reads,bases",
    }
}

fn stats_stage_contract() -> FastqStageContract {
    FastqStageContract {
        input_kind: FastqArtifactKind::SingleEnd,
        output_kind: FastqArtifactKind::StatsOnly,
        accepted_input_kinds: SE_PE_OR_MERGED,
        possible_output_kinds: STATS_ONLY,
        may_drop_reads: false,
        must_preserve_pairing: true,
        emits_fastq: false,
        preserves: &["reads", "pairs", "bases"],
        may_drop: &[],
        retention_definition: "reads_out == reads_in; bases_out == bases_in",
        retention_units: "reads,bases",
    }
}

fn taxonomy_stage_contract() -> FastqStageContract {
    FastqStageContract { possible_output_kinds: STATS_OR_TAXONOMY, ..stats_stage_contract() }
}

fn amplicon_stage_contract(stage_id: &str) -> FastqStageContract {
    FastqStageContract {
        input_kind: FastqArtifactKind::SingleEnd,
        output_kind: FastqArtifactKind::AmpliconTable,
        accepted_input_kinds: if stage_id == "fastq.normalize_abundance" {
            AMPLICON_TABLE
        } else {
            SE_PE_OR_MERGED
        },
        possible_output_kinds: if stage_id == "fastq.normalize_abundance" {
            AMPLICON_TABLE
        } else {
            AMPLICON_TABLE_OR_REPRESENTATIVES
        },
        may_drop_reads: false,
        must_preserve_pairing: false,
        emits_fastq: false,
        preserves: &["sample_ids", "abundance_units"],
        may_drop: &[],
        retention_definition: "non-empty table rows and stable sample identifiers",
        retention_units: "rows,samples",
    }
}

#[must_use]
pub fn contract_for_stage(stage_id: &str) -> Option<FastqStageContract> {
    match stage_id {
        "fastq.trim_reads"
        | "fastq.trim_terminal_damage"
        | "fastq.filter_reads"
        | "fastq.normalize_primers"
        | "fastq.remove_chimeras"
        | "fastq.remove_duplicates"
        | "fastq.filter_low_complexity"
        | "fastq.trim_polyg_tails"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna" => Some(single_end_transform_contract()),
        "fastq.merge_pairs" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::PairedEnd,
            output_kind: FastqArtifactKind::Merged,
            accepted_input_kinds: PE,
            possible_output_kinds: MERGED_OR_PE_OUT,
            may_drop_reads: true,
            must_preserve_pairing: false,
            emits_fastq: true,
            preserves: &["pairing_metadata"],
            may_drop: &["reads", "pairs", "bases"],
            retention_definition: "reads_merged + reads_unmerged <= min(reads_r1, reads_r2)",
            retention_units: "reads,pairs,bases",
        }),
        "fastq.correct_errors" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::SingleEnd,
            output_kind: FastqArtifactKind::SingleEnd,
            accepted_input_kinds: SE_OR_PE,
            possible_output_kinds: SE_OR_PE_OUT,
            may_drop_reads: false,
            must_preserve_pairing: true,
            emits_fastq: true,
            preserves: &["reads", "pairs", "bases", "pairing_metadata"],
            may_drop: &[],
            retention_definition: "reads_out == reads_in; bases_out <= bases_in",
            retention_units: "reads,bases",
        }),
        "fastq.extract_umis" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::PairedEnd,
            output_kind: FastqArtifactKind::PairedEnd,
            accepted_input_kinds: PE,
            possible_output_kinds: PE,
            may_drop_reads: false,
            must_preserve_pairing: true,
            emits_fastq: true,
            preserves: &["reads", "pairs", "bases", "pairing_metadata"],
            may_drop: &[],
            retention_definition: "reads_out == reads_in; bases_out == bases_in",
            retention_units: "reads,bases",
        }),
        "fastq.validate_reads"
        | "fastq.profile_read_lengths"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.detect_adapters"
        | "fastq.detect_duplicates_premerge"
        | "fastq.estimate_library_complexity_prealign"
        | "fastq.profile_reads"
        | "fastq.report_qc" => Some(stats_stage_contract()),
        "fastq.screen_taxonomy" => Some(taxonomy_stage_contract()),
        "fastq.index_reference" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::ReferenceFasta,
            output_kind: FastqArtifactKind::ReferenceIndex,
            accepted_input_kinds: REF_FASTA,
            possible_output_kinds: REF_INDEX,
            may_drop_reads: false,
            must_preserve_pairing: false,
            emits_fastq: false,
            preserves: &["reference_sequence_records"],
            may_drop: &[],
            retention_definition: "indexed_reference_entries_out == reference_entries_in",
            retention_units: "reference_entries",
        }),
        "fastq.infer_asvs" | "fastq.cluster_otus" | "fastq.normalize_abundance" => {
            Some(amplicon_stage_contract(stage_id))
        }
        _ => None,
    }
}

#[must_use]
pub(crate) fn stage_for_id(stage_id: &str) -> Option<FastqStage> {
    match stage_id {
        "fastq.index_reference" => Some(FastqStage::PrepareReference),
        "fastq.validate_reads" => Some(FastqStage::ValidateReads),
        "fastq.profile_read_lengths" => Some(FastqStage::ProfileReadLengths),
        "fastq.detect_adapters" => Some(FastqStage::DetectAdapters),
        "fastq.detect_duplicates_premerge" => Some(FastqStage::Deduplicate),
        "fastq.estimate_library_complexity_prealign" => Some(FastqStage::LowComplexity),
        "fastq.trim_terminal_damage" => Some(FastqStage::DamageAwarePretrim),
        "fastq.normalize_primers" => Some(FastqStage::PrimerNormalization),
        "fastq.trim_polyg_tails" => Some(FastqStage::PolygTailing),
        "fastq.trim_reads" => Some(FastqStage::Trim),
        "fastq.filter_reads" => Some(FastqStage::Filter),
        "fastq.profile_reads" => Some(FastqStage::ProfileReads),
        "fastq.deplete_rrna" => Some(FastqStage::Rrna),
        "fastq.merge_pairs" => Some(FastqStage::Merge),
        "fastq.remove_duplicates" => Some(FastqStage::Deduplicate),
        "fastq.filter_low_complexity" => Some(FastqStage::LowComplexity),
        "fastq.deplete_host" => Some(FastqStage::HostDepletion),
        "fastq.deplete_reference_contaminants" => Some(FastqStage::ContaminantScreen),
        "fastq.correct_errors" => Some(FastqStage::Correct),
        "fastq.extract_umis" => Some(FastqStage::Umi),
        "fastq.profile_overrepresented_sequences" => {
            Some(FastqStage::ProfileOverrepresentedSequences)
        }
        "fastq.report_qc" => Some(FastqStage::ReportQc),
        "fastq.screen_taxonomy" => Some(FastqStage::Screen),
        "fastq.remove_chimeras" => Some(FastqStage::ChimeraDetection),
        "fastq.infer_asvs" => Some(FastqStage::AsvInference),
        "fastq.cluster_otus" => Some(FastqStage::OtuClustering),
        "fastq.normalize_abundance" => Some(FastqStage::AbundanceNormalization),
        _ => None,
    }
}
