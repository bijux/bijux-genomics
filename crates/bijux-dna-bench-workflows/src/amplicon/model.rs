use bijux_dna_bench_corpus::amplicon::AmpliconAbundanceTruthRow;
use bijux_dna_bench_corpus::amplicon::AmpliconExpectedAsvTruthRow;
use bijux_dna_bench_corpus::amplicon::AmpliconExpectedChimeraTruthRow;
use bijux_dna_bench_corpus::amplicon::AmpliconPrimerTruthRow;
use serde::Deserialize;
use serde::Serialize;

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
// Owner: bijux-dna-bench-workflows
// amplicon workflow wire and internal artifact contracts.
pub const DEFAULT_AMPLICON_MICRO_PIPELINE_PATH: &str =
    "artifacts/benchmark-smoke/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json";
pub(crate) const GOVERNED_MICRO_STARTED_AT: &str = "1704067200";
pub(crate) const GOVERNED_MICRO_FINISHED_AT: &str = "1704067201";
pub(crate) const GOVERNED_MICRO_ELAPSED_SECONDS: f64 = 1.0;
pub(crate) const DEFAULT_AMPLICON_TRUTH_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/manifest.toml";
pub(crate) const DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/expected.json";
pub(crate) const DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/asv_representatives.fasta";
pub(crate) const DEFAULT_AMPLICON_NON_CHIMERIC_REPRESENTATIVES_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/non_chimeric_representatives.fasta";
pub(crate) const DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/otu_representatives.fasta";
pub(crate) const DEFAULT_AMPLICON_NORMALIZED_ABUNDANCE_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/normalized_abundance.tsv";
pub(crate) const DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-se.fastq.gz";
pub(crate) const DEFAULT_AMPLICON_INFER_ASVS_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-amplicon-se.fastq.gz";
pub(crate) const DEFAULT_AMPLICON_CLUSTER_OTUS_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-otu-cluster-se.fastq.gz";
pub(crate) const DEFAULT_AMPLICON_CHIMERA_CONTROL_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/chimera-control-se.fastq.gz";
pub(crate) const AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION: &str =
    "bijux.bench.local_amplicon_micro_pipeline.v1";
pub(crate) const AMPLICON_MICRO_PIPELINE_COMMAND: &str =
    "bijux-dna bench local run-amplicon-micro-pipeline";
pub(crate) const AMPLICON_MICRO_PIPELINE_ID: &str = "amplicon-asv-otu-no-vcf";
pub(crate) const OTU_ABUNDANCE_TABLE_KIND: &str = "otu_abundance";
pub(crate) const NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_primers.local_smoke.report.v2";
pub(crate) const INFER_ASVS_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.infer_asvs.local_smoke.report.v1";
pub(crate) const REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.remove_chimeras.local_smoke.report.v1";
pub(crate) const CLUSTER_OTUS_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.cluster_otus.local_smoke.report.v1";
pub(crate) const NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_abundance.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconMicroPipelineReport {
    pub schema_version: &'static str,
    pub command: &'static str,
    pub output_path: String,
    pub pipeline_id: &'static str,
    pub corpus_manifest_path: String,
    pub truth_manifest_path: String,
    pub sample_count: usize,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub stage_count: usize,
    pub handoff_count: usize,
    pub passes_behavior_test: bool,
    pub rows: Vec<AmpliconMicroPipelineRow>,
    pub handoffs: Vec<AmpliconMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AmpliconMicroPipelineRowStatus {
    Succeeded,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconMicroPipelineRow {
    pub stage_id: String,
    pub domain: String,
    pub tool_id: String,
    pub execution_mode: String,
    pub status: AmpliconMicroPipelineRowStatus,
    pub reason: String,
    pub evidence_path: Option<String>,
    pub parsed_schema_version: Option<String>,
    pub consumed_inputs: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, String>,
    pub metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
// Keep independent artifact observations as separate wire fields.
pub struct AmpliconMicroPipelineHandoff {
    pub handoff_id: String,
    pub source_stage_id: String,
    pub target_stage_id: String,
    pub source_output_id: String,
    pub target_input_id: String,
    pub source_path: String,
    pub target_path: String,
    pub source_exists: bool,
    pub target_exists: bool,
    pub exact_path_match: bool,
    pub accepted: bool,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NormalizePrimersLocalSmokeReport {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) case_count: usize,
    pub(crate) cases: Vec<NormalizePrimersLocalSmokeCase>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NormalizePrimersLocalSmokeCase {
    pub(crate) sample_id: String,
    pub(crate) tool_id: String,
    pub(crate) layout: String,
    pub(crate) primer_set_id: String,
    pub(crate) marker_id: String,
    pub(crate) orientation_policy: String,
    pub(crate) input_reads: u64,
    pub(crate) matched_reads: u64,
    pub(crate) unmatched_reads: u64,
    pub(crate) output_reads: u64,
    pub(crate) normalized_reads_r1: String,
    pub(crate) normalized_reads_r2: Option<String>,
    pub(crate) report_json: String,
    pub(crate) primer_orientation_report: String,
    pub(crate) primer_stats_json: String,
    pub(crate) used_fallback: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct NormalizePrimersEvidence {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) sample_id: String,
    pub(crate) tool_id: String,
    pub(crate) layout: String,
    pub(crate) primer_set_id: String,
    pub(crate) marker_id: String,
    pub(crate) orientation_policy: String,
    pub(crate) input_reads: u64,
    pub(crate) matched_reads: u64,
    pub(crate) unmatched_reads: u64,
    pub(crate) output_reads: u64,
    pub(crate) normalized_reads_r1: String,
    pub(crate) normalized_reads_r2: Option<String>,
    pub(crate) report_json: String,
    pub(crate) primer_orientation_report: String,
    pub(crate) primer_stats_json: String,
    pub(crate) used_fallback: bool,
    pub(crate) source_report_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct InferAsvsStageEvidence {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) sample_id: String,
    pub(crate) planned_tool_id: String,
    pub(crate) report_tool_id: String,
    pub(crate) asv_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) representative_sequence_count: u64,
    pub(crate) asv_table_tsv: String,
    pub(crate) representatives_fasta: String,
    pub(crate) case_report_json: String,
    pub(crate) taxonomy_ready_fasta: String,
    pub(crate) taxonomy_ready_fastq: String,
    pub(crate) raw_backend_report: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct RemoveChimerasStageEvidence {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) sample_id: String,
    pub(crate) planned_tool_id: String,
    pub(crate) report_tool_id: String,
    pub(crate) checked_sequence_count: u64,
    pub(crate) chimera_count: u64,
    pub(crate) non_chimera_count: u64,
    pub(crate) filtered_representative_sequences: String,
    pub(crate) non_chimeric_fasta: String,
    pub(crate) chimeras_tsv: String,
    pub(crate) case_report_json: String,
    pub(crate) chimera_metrics_json: String,
    pub(crate) chimeras_fasta: String,
    pub(crate) raw_backend_report: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ClusterOtusStageEvidence {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) sample_id: String,
    pub(crate) planned_tool_id: String,
    pub(crate) report_tool_id: String,
    pub(crate) clustering_threshold: f64,
    pub(crate) otu_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) representative_sequence_count: u64,
    pub(crate) otu_table_tsv: String,
    pub(crate) representative_sequences_fasta: String,
    pub(crate) otu_representatives_fasta: String,
    pub(crate) case_report_json: String,
    pub(crate) taxonomy_ready_fasta: String,
    pub(crate) taxonomy_ready_fastq: String,
    pub(crate) raw_backend_report: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct NormalizeAbundanceStageEvidence {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) sample_id: String,
    pub(crate) planned_tool_id: String,
    pub(crate) report_tool_id: String,
    pub(crate) method: String,
    pub(crate) normalization_method: String,
    pub(crate) table_rows: u64,
    pub(crate) sample_count: u64,
    pub(crate) feature_count: u64,
    pub(crate) zero_fraction: f64,
    pub(crate) normalized_abundance_tsv: String,
    pub(crate) sample_totals: Vec<(String, f64)>,
    pub(crate) numeric_output_valid: bool,
    pub(crate) case_report_json: String,
    #[serde(default)]
    pub(crate) otu_abundance_table_tsv: String,
}

#[derive(Debug, Clone, Serialize)]
// Keep independent artifact observations as separate wire fields.
pub(crate) struct AmpliconOutputJudgmentEvidence {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) valid: bool,
    pub(crate) checked_section_count: usize,
    pub(crate) checked_row_count: usize,
    pub(crate) primer_case_valid: bool,
    pub(crate) asv_table_valid: bool,
    pub(crate) asv_representatives_valid: bool,
    pub(crate) chimera_table_valid: bool,
    pub(crate) non_chimeric_representatives_valid: bool,
    pub(crate) otu_table_valid: bool,
    pub(crate) otu_representatives_valid: bool,
    pub(crate) otu_abundance_table_valid: bool,
    pub(crate) normalized_abundance_valid: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconTruthBundle {
    pub(crate) schema_version: String,
    pub(crate) fixture_id: String,
    pub(crate) primer_truths: Vec<AmpliconPrimerTruthRow>,
    pub(crate) asv_truths: Vec<AmpliconExpectedAsvTruthRow>,
    pub(crate) chimera_truths: Vec<AmpliconExpectedChimeraTruthRow>,
    pub(crate) asv_representatives: Vec<FastaTruth>,
    pub(crate) non_chimeric_representatives: Vec<FastaTruth>,
    pub(crate) otu_representatives: Vec<FastaTruth>,
    pub(crate) otu_abundances: Vec<AmpliconAbundanceTruthRow>,
    pub(crate) normalized_abundances: Vec<NormalizedAbundanceTruthRow>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct FastaTruth {
    pub(crate) id: String,
    pub(crate) sequence: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct NormalizedAbundanceTruthRow {
    pub(crate) sample_id: String,
    pub(crate) feature_id: String,
    pub(crate) normalized_abundance: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OtuTableRow {
    pub(crate) sample_id: String,
    pub(crate) otu_id: String,
    pub(crate) abundance: String,
    pub(crate) representative_id: String,
    pub(crate) representative_fasta: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AmpliconCorpusStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct AmpliconTruthStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct NormalizePrimersStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) normalized_reads_r1: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct InferAsvsStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) asv_table_tsv: PathBuf,
    pub(crate) representatives_fasta: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct RemoveChimerasStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) chimeras_tsv: PathBuf,
    pub(crate) non_chimeric_fasta: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct ClusterOtusStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) otu_table_tsv: PathBuf,
    pub(crate) otu_representatives_fasta: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct NormalizeAbundanceStageArtifacts {
    pub(crate) row: AmpliconMicroPipelineRow,
    pub(crate) otu_abundance_table_tsv: PathBuf,
    pub(crate) normalized_abundance_tsv: PathBuf,
}
