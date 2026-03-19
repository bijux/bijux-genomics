use bijux_dna_core::ids::StageId;

pub const STAGE_VALIDATE_READS: StageId = StageId::from_static("fastq.validate_reads");
pub const STAGE_PROFILE_READ_LENGTHS: StageId =
    StageId::from_static("fastq.profile_read_lengths");
pub const STAGE_DETECT_ADAPTERS: StageId = StageId::from_static("fastq.detect_adapters");
pub const STAGE_TRIM_TERMINAL_DAMAGE: StageId = StageId::from_static("fastq.trim_terminal_damage");
pub const STAGE_TRIM_POLYG_TAILS: StageId = StageId::from_static("fastq.trim_polyg_tails");
pub const STAGE_TRIM_READS: StageId = StageId::from_static("fastq.trim_reads");
pub const STAGE_FILTER_READS: StageId = StageId::from_static("fastq.filter_reads");
pub const STAGE_PROFILE_READS: StageId = StageId::from_static("fastq.profile_reads");
pub const STAGE_MERGE_PAIRS: StageId = StageId::from_static("fastq.merge_pairs");
pub const STAGE_REMOVE_DUPLICATES: StageId = StageId::from_static("fastq.remove_duplicates");
pub const STAGE_FILTER_LOW_COMPLEXITY: StageId = StageId::from_static("fastq.filter_low_complexity");
pub const STAGE_DEPLETE_HOST: StageId = StageId::from_static("fastq.deplete_host");
pub const STAGE_DEPLETE_REFERENCE_CONTAMINANTS: StageId = StageId::from_static("fastq.deplete_reference_contaminants");
pub const STAGE_CORRECT_ERRORS: StageId = StageId::from_static("fastq.correct_errors");
pub const STAGE_REPORT_QC: StageId = StageId::from_static("fastq.report_qc");
pub const STAGE_EXTRACT_UMIS: StageId = StageId::from_static("fastq.extract_umis");
pub const STAGE_PROFILE_OVERREPRESENTED_SEQUENCES: StageId =
    StageId::from_static("fastq.profile_overrepresented_sequences");
pub const STAGE_SCREEN_TAXONOMY: StageId = StageId::from_static("fastq.screen_taxonomy");
pub const STAGE_PREPARE_REFERENCE: StageId = StageId::from_static("fastq.prepare_reference");
pub const STAGE_DEPLETE_RRNA: StageId = StageId::from_static("fastq.deplete_rrna");
pub const STAGE_PRIMER_NORMALIZATION: StageId = StageId::from_static("fastq.primer_normalization");
pub const STAGE_CHIMERA_DETECTION: StageId = StageId::from_static("fastq.chimera_detection");
pub const STAGE_ASV_INFERENCE: StageId = StageId::from_static("fastq.asv_inference");
pub const STAGE_OTU_CLUSTERING: StageId = StageId::from_static("fastq.otu_clustering");
pub const STAGE_ABUNDANCE_NORMALIZATION: StageId =
    StageId::from_static("fastq.abundance_normalization");

pub const STAGE_PREFIX: &str = "fastq.";

pub const STAGES: [StageId; 25] = [
    STAGE_PREPARE_REFERENCE,
    STAGE_VALIDATE_READS,
    STAGE_PROFILE_READ_LENGTHS,
    STAGE_DETECT_ADAPTERS,
    STAGE_TRIM_TERMINAL_DAMAGE,
    STAGE_PRIMER_NORMALIZATION,
    STAGE_TRIM_POLYG_TAILS,
    STAGE_TRIM_READS,
    STAGE_FILTER_READS,
    STAGE_PROFILE_READS,
    STAGE_DEPLETE_RRNA,
    STAGE_MERGE_PAIRS,
    STAGE_REMOVE_DUPLICATES,
    STAGE_FILTER_LOW_COMPLEXITY,
    STAGE_DEPLETE_HOST,
    STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
    STAGE_CORRECT_ERRORS,
    STAGE_EXTRACT_UMIS,
    STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
    STAGE_CHIMERA_DETECTION,
    STAGE_ASV_INFERENCE,
    STAGE_OTU_CLUSTERING,
    STAGE_ABUNDANCE_NORMALIZATION,
    STAGE_SCREEN_TAXONOMY,
    STAGE_REPORT_QC,
];

#[must_use]
pub fn bench_dir_name(stage: &StageId) -> Option<&'static str> {
    if stage == &STAGE_VALIDATE_READS {
        Some("validate_reads")
    } else if stage == &STAGE_PROFILE_READ_LENGTHS {
        Some("profile_read_lengths")
    } else if stage == &STAGE_DETECT_ADAPTERS {
        Some("detect_adapters")
    } else if stage == &STAGE_TRIM_TERMINAL_DAMAGE {
        Some("trim_terminal_damage")
    } else if stage == &STAGE_TRIM_POLYG_TAILS {
        Some("trim_polyg_tails")
    } else if stage == &STAGE_TRIM_READS {
        Some("trim_reads")
    } else if stage == &STAGE_FILTER_READS {
        Some("filter_reads")
    } else if stage == &STAGE_PROFILE_READS {
        Some("profile_reads")
    } else if stage == &STAGE_DEPLETE_RRNA {
        Some("deplete_rrna")
    } else if stage == &STAGE_MERGE_PAIRS {
        Some("merge_pairs")
    } else if stage == &STAGE_REMOVE_DUPLICATES {
        Some("remove_duplicates")
    } else if stage == &STAGE_FILTER_LOW_COMPLEXITY {
        Some("filter_low_complexity")
    } else if stage == &STAGE_DEPLETE_HOST {
        Some("deplete_host")
    } else if stage == &STAGE_DEPLETE_REFERENCE_CONTAMINANTS {
        Some("deplete_reference_contaminants")
    } else if stage == &STAGE_CORRECT_ERRORS {
        Some("correct_errors")
    } else if stage == &STAGE_REPORT_QC {
        Some("report_qc")
    } else if stage == &STAGE_EXTRACT_UMIS {
        Some("extract_umis")
    } else if stage == &STAGE_PROFILE_OVERREPRESENTED_SEQUENCES {
        Some("profile_overrepresented_sequences")
    } else if stage == &STAGE_PRIMER_NORMALIZATION {
        Some("primer_normalization")
    } else if stage == &STAGE_CHIMERA_DETECTION {
        Some("chimera_detection")
    } else if stage == &STAGE_ASV_INFERENCE {
        Some("asv_inference")
    } else if stage == &STAGE_OTU_CLUSTERING {
        Some("otu_clustering")
    } else if stage == &STAGE_ABUNDANCE_NORMALIZATION {
        Some("abundance_normalization")
    } else if stage == &STAGE_SCREEN_TAXONOMY {
        Some("screen_taxonomy")
    } else if stage == &STAGE_PREPARE_REFERENCE {
        Some("prepare_reference")
    } else {
        None
    }
}
