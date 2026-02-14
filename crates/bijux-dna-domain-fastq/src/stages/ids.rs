use bijux_dna_core::ids::StageId;

pub const STAGE_VALIDATE_PRE: StageId = StageId::from_static("fastq.validate_pre");
pub const STAGE_LENGTH_DISTRIBUTION_PRE: StageId =
    StageId::from_static("fastq.length_distribution_pre");
pub const STAGE_DETECT_ADAPTERS: StageId = StageId::from_static("fastq.detect_adapters");
pub const STAGE_POLYG_TAILING: StageId = StageId::from_static("fastq.polyg_tailing");
pub const STAGE_TRIM: StageId = StageId::from_static("fastq.trim");
pub const STAGE_FILTER: StageId = StageId::from_static("fastq.filter");
pub const STAGE_STATS_NEUTRAL: StageId = StageId::from_static("fastq.stats_neutral");
pub const STAGE_MERGE: StageId = StageId::from_static("fastq.merge");
pub const STAGE_DEDUPLICATE: StageId = StageId::from_static("fastq.deduplicate");
pub const STAGE_LOW_COMPLEXITY: StageId = StageId::from_static("fastq.low_complexity");
pub const STAGE_HOST_DEPLETION: StageId = StageId::from_static("fastq.host_depletion");
pub const STAGE_CONTAMINANT_SCREEN: StageId = StageId::from_static("fastq.contaminant_screen");
pub const STAGE_CORRECT: StageId = StageId::from_static("fastq.correct");
pub const STAGE_QC_POST: StageId = StageId::from_static("fastq.qc_post");
pub const STAGE_UMI: StageId = StageId::from_static("fastq.umi");
pub const STAGE_OVERREPRESENTED_SEQUENCES: StageId =
    StageId::from_static("fastq.overrepresented_sequences");
pub const STAGE_SCREEN: StageId = StageId::from_static("fastq.screen");
pub const STAGE_PREPROCESS: StageId = StageId::from_static("fastq.preprocess");
pub const STAGE_PREPARE_REFERENCE: StageId = StageId::from_static("fastq.prepare_reference");
pub const STAGE_RRNA: StageId = StageId::from_static("fastq.rrna");
pub const STAGE_PRIMER_NORMALIZATION: StageId = StageId::from_static("fastq.primer_normalization");
pub const STAGE_CHIMERA_DETECTION: StageId = StageId::from_static("fastq.chimera_detection");
pub const STAGE_ASV_INFERENCE: StageId = StageId::from_static("fastq.asv_inference");
pub const STAGE_OTU_CLUSTERING: StageId = StageId::from_static("fastq.otu_clustering");
pub const STAGE_ABUNDANCE_NORMALIZATION: StageId =
    StageId::from_static("fastq.abundance_normalization");

pub const STAGE_PREFIX: &str = "fastq.";

pub const STAGES: [StageId; 24] = [
    STAGE_PREPARE_REFERENCE,
    STAGE_VALIDATE_PRE,
    STAGE_LENGTH_DISTRIBUTION_PRE,
    STAGE_DETECT_ADAPTERS,
    STAGE_PRIMER_NORMALIZATION,
    STAGE_POLYG_TAILING,
    STAGE_TRIM,
    STAGE_FILTER,
    STAGE_STATS_NEUTRAL,
    STAGE_RRNA,
    STAGE_MERGE,
    STAGE_DEDUPLICATE,
    STAGE_LOW_COMPLEXITY,
    STAGE_HOST_DEPLETION,
    STAGE_CONTAMINANT_SCREEN,
    STAGE_CORRECT,
    STAGE_UMI,
    STAGE_OVERREPRESENTED_SEQUENCES,
    STAGE_CHIMERA_DETECTION,
    STAGE_ASV_INFERENCE,
    STAGE_OTU_CLUSTERING,
    STAGE_ABUNDANCE_NORMALIZATION,
    STAGE_SCREEN,
    STAGE_QC_POST,
];

#[must_use]
pub fn bench_dir_name(stage: &StageId) -> Option<&'static str> {
    if stage == &STAGE_VALIDATE_PRE {
        Some("validate_pre")
    } else if stage == &STAGE_LENGTH_DISTRIBUTION_PRE {
        Some("length_distribution_pre")
    } else if stage == &STAGE_DETECT_ADAPTERS {
        Some("detect_adapters")
    } else if stage == &STAGE_POLYG_TAILING {
        Some("polyg_tailing")
    } else if stage == &STAGE_TRIM {
        Some("trim")
    } else if stage == &STAGE_FILTER {
        Some("filter")
    } else if stage == &STAGE_STATS_NEUTRAL {
        Some("stats")
    } else if stage == &STAGE_RRNA {
        Some("rrna")
    } else if stage == &STAGE_MERGE {
        Some("merge")
    } else if stage == &STAGE_DEDUPLICATE {
        Some("deduplicate")
    } else if stage == &STAGE_LOW_COMPLEXITY {
        Some("low_complexity")
    } else if stage == &STAGE_HOST_DEPLETION {
        Some("host_depletion")
    } else if stage == &STAGE_CONTAMINANT_SCREEN {
        Some("contaminant_screen")
    } else if stage == &STAGE_CORRECT {
        Some("correct")
    } else if stage == &STAGE_QC_POST {
        Some("qc_post")
    } else if stage == &STAGE_UMI {
        Some("umi")
    } else if stage == &STAGE_OVERREPRESENTED_SEQUENCES {
        Some("overrepresented_sequences")
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
    } else if stage == &STAGE_SCREEN {
        Some("screen")
    } else if stage == &STAGE_PREPROCESS {
        Some("preprocess")
    } else if stage == &STAGE_PREPARE_REFERENCE {
        Some("prepare_reference")
    } else {
        None
    }
}
