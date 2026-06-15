/// Canonical FASTQ preprocessing stage.
pub const FASTQ_PREPROCESS: &str = "fastq.preprocess";
/// Canonical FASTQ abundance-normalization stage.
pub const FASTQ_NORMALIZE_ABUNDANCE: &str = "fastq.normalize_abundance";
/// Canonical FASTQ ASV inference stage.
pub const FASTQ_INFER_ASVS: &str = "fastq.infer_asvs";
/// Canonical FASTQ chimera-removal stage.
pub const FASTQ_REMOVE_CHIMERAS: &str = "fastq.remove_chimeras";
/// Canonical FASTQ validation stage.
pub const FASTQ_VALIDATE_READS: &str = "fastq.validate_reads";
/// Alias used when FASTQ validation is positioned as a preflight stage.
pub const FASTQ_VALIDATE_PRE: &str = FASTQ_VALIDATE_READS;
/// Canonical FASTQ adapter-detection stage.
pub const FASTQ_DETECT_ADAPTERS: &str = "fastq.detect_adapters";
/// Canonical FASTQ premerge duplicate-signaling stage.
pub const FASTQ_DETECT_DUPLICATES_PREMERGE: &str = "fastq.detect_duplicates_premerge";
/// Canonical FASTQ prealignment library-complexity stage.
pub const FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN: &str =
    "fastq.estimate_library_complexity_prealign";
/// Canonical FASTQ trimming stage.
pub const FASTQ_TRIM: &str = "fastq.trim_reads";
/// Canonical FASTQ filtering stage.
pub const FASTQ_FILTER: &str = "fastq.filter_reads";
/// Canonical FASTQ host-depletion stage.
pub const FASTQ_DEPLETE_HOST: &str = "fastq.deplete_host";
/// Canonical FASTQ reference-contaminant depletion stage.
pub const FASTQ_DEPLETE_REFERENCE_CONTAMINANTS: &str = "fastq.deplete_reference_contaminants";
/// Canonical FASTQ duplicate-removal stage.
pub const FASTQ_DEDUPLICATE: &str = "fastq.remove_duplicates";
/// Canonical FASTQ low-complexity filtering stage.
pub const FASTQ_LOW_COMPLEXITY: &str = "fastq.filter_low_complexity";
/// Canonical FASTQ read-merging stage.
pub const FASTQ_MERGE: &str = "fastq.merge_pairs";
/// Canonical FASTQ error-correction stage.
pub const FASTQ_CORRECT: &str = "fastq.correct_errors";
/// Canonical FASTQ terminal-damage trimming stage.
pub const FASTQ_TRIM_TERMINAL_DAMAGE: &str = "fastq.trim_terminal_damage";
/// Canonical FASTQ read-length profiling stage.
pub const FASTQ_PROFILE_READ_LENGTHS: &str = "fastq.profile_read_lengths";
/// Canonical FASTQ OTU clustering stage.
pub const FASTQ_CLUSTER_OTUS: &str = "fastq.cluster_otus";
/// Canonical FASTQ overrepresented-sequence profiling stage.
pub const FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES: &str = "fastq.profile_overrepresented_sequences";
/// Canonical FASTQ poly-G trimming stage.
pub const FASTQ_TRIM_POLYG_TAILS: &str = "fastq.trim_polyg_tails";
/// Canonical FASTQ reference indexing stage.
pub const FASTQ_INDEX_REFERENCE: &str = "fastq.index_reference";
/// Canonical FASTQ primer-normalization stage.
pub const FASTQ_NORMALIZE_PRIMERS: &str = "fastq.normalize_primers";
/// Canonical FASTQ QC reporting stage.
pub const FASTQ_QC_POST: &str = "fastq.report_qc";
/// Canonical FASTQ rRNA depletion stage.
pub const FASTQ_DEPLETE_RRNA: &str = "fastq.deplete_rrna";
/// Canonical FASTQ UMI extraction stage.
pub const FASTQ_UMI: &str = "fastq.extract_umis";
/// Canonical FASTQ taxonomy-screening stage.
pub const FASTQ_SCREEN: &str = "fastq.screen_taxonomy";
/// Canonical FASTQ neutral read-profiling stage.
pub const FASTQ_STATS_NEUTRAL: &str = "fastq.profile_reads";
