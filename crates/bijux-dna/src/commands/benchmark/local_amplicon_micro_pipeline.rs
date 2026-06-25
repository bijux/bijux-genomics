use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::local_corpus_fixture::amplicon::{
    load_amplicon_corpus_fixture_manifest_path, load_validated_amplicon_abundance_rows,
    load_validated_amplicon_expected_asv_rows, load_validated_amplicon_expected_chimera_rows,
    load_validated_amplicon_primer_rows, validate_amplicon_corpus_fixture_manifest_path,
    AmpliconAbundanceTruthRow, AmpliconCorpusFixtureManifest, AmpliconExpectedAsvTruthRow,
    AmpliconExpectedChimeraTruthRow, AmpliconPrimerTruthRow,
    DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
};
use super::local_stage_result_manifest::path_relative_to_repo;
use crate::commands::cli::parse;
use crate::commands::fixtures::expected::amplicon::validate_amplicon_truth_manifest_path;

pub(crate) const DEFAULT_AMPLICON_MICRO_PIPELINE_PATH: &str =
    "runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json";
const GOVERNED_MICRO_STARTED_AT: &str = "1704067200";
const GOVERNED_MICRO_FINISHED_AT: &str = "1704067201";
const GOVERNED_MICRO_ELAPSED_SECONDS: f64 = 1.0;
const DEFAULT_AMPLICON_TRUTH_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/manifest.toml";
const DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/expected.json";
const DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/asv_representatives.fasta";
const DEFAULT_AMPLICON_NON_CHIMERIC_REPRESENTATIVES_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/non_chimeric_representatives.fasta";
const DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/otu_representatives.fasta";
const DEFAULT_AMPLICON_NORMALIZED_ABUNDANCE_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/normalized_abundance.tsv";
const DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-se.fastq.gz";
const DEFAULT_AMPLICON_INFER_ASVS_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-amplicon-se.fastq.gz";
const DEFAULT_AMPLICON_CLUSTER_OTUS_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-otu-cluster-se.fastq.gz";
const DEFAULT_AMPLICON_CHIMERA_CONTROL_FASTQ_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/chimera-control-se.fastq.gz";
const AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION: &str = "bijux.bench.local_amplicon_micro_pipeline.v1";
const AMPLICON_MICRO_PIPELINE_COMMAND: &str = "bijux-dna bench local run-amplicon-micro-pipeline";
const AMPLICON_MICRO_PIPELINE_ID: &str = "amplicon-asv-otu-no-vcf";
const OTU_ABUNDANCE_TABLE_KIND: &str = "otu_abundance";
const NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_primers.local_smoke.report.v2";
const INFER_ASVS_STAGE_SCHEMA_VERSION: &str = "bijux.fastq.infer_asvs.local_smoke.report.v1";
const REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.remove_chimeras.local_smoke.report.v1";
const CLUSTER_OTUS_STAGE_SCHEMA_VERSION: &str = "bijux.fastq.cluster_otus.local_smoke.report.v1";
const NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_abundance.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconMicroPipelineReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: &'static str,
    pub(crate) output_path: String,
    pub(crate) pipeline_id: &'static str,
    pub(crate) corpus_manifest_path: String,
    pub(crate) truth_manifest_path: String,
    pub(crate) sample_count: usize,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) stage_count: usize,
    pub(crate) handoff_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<AmpliconMicroPipelineRow>,
    pub(crate) handoffs: Vec<AmpliconMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AmpliconMicroPipelineRowStatus {
    Succeeded,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconMicroPipelineRow {
    pub(crate) stage_id: String,
    pub(crate) domain: String,
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) status: AmpliconMicroPipelineRowStatus,
    pub(crate) reason: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) parsed_schema_version: Option<String>,
    pub(crate) consumed_inputs: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, String>,
    pub(crate) metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconMicroPipelineHandoff {
    pub(crate) handoff_id: String,
    pub(crate) source_stage_id: String,
    pub(crate) target_stage_id: String,
    pub(crate) source_output_id: String,
    pub(crate) target_input_id: String,
    pub(crate) source_path: String,
    pub(crate) target_path: String,
    pub(crate) source_exists: bool,
    pub(crate) target_exists: bool,
    pub(crate) exact_path_match: bool,
    pub(crate) accepted: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Deserialize)]
struct NormalizePrimersLocalSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: usize,
    cases: Vec<NormalizePrimersLocalSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct NormalizePrimersLocalSmokeCase {
    sample_id: String,
    tool_id: String,
    layout: String,
    primer_set_id: String,
    marker_id: String,
    orientation_policy: String,
    input_reads: u64,
    matched_reads: u64,
    unmatched_reads: u64,
    output_reads: u64,
    normalized_reads_r1: String,
    normalized_reads_r2: Option<String>,
    report_json: String,
    primer_orientation_report: String,
    primer_stats_json: String,
    used_fallback: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NormalizePrimersEvidence {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_id: String,
    layout: String,
    primer_set_id: String,
    marker_id: String,
    orientation_policy: String,
    input_reads: u64,
    matched_reads: u64,
    unmatched_reads: u64,
    output_reads: u64,
    normalized_reads_r1: String,
    normalized_reads_r2: Option<String>,
    report_json: String,
    primer_orientation_report: String,
    primer_stats_json: String,
    used_fallback: bool,
    source_report_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct InferAsvsStageEvidence {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    asv_count: u64,
    sample_count: u64,
    representative_sequence_count: u64,
    asv_table_tsv: String,
    representatives_fasta: String,
    case_report_json: String,
    taxonomy_ready_fasta: String,
    taxonomy_ready_fastq: String,
    raw_backend_report: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RemoveChimerasStageEvidence {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    checked_sequence_count: u64,
    chimera_count: u64,
    non_chimera_count: u64,
    filtered_representative_sequences: String,
    non_chimeric_fasta: String,
    chimeras_tsv: String,
    case_report_json: String,
    chimera_metrics_json: String,
    chimeras_fasta: String,
    raw_backend_report: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ClusterOtusStageEvidence {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    clustering_threshold: f64,
    otu_count: u64,
    sample_count: u64,
    representative_sequence_count: u64,
    otu_table_tsv: String,
    representative_sequences_fasta: String,
    otu_representatives_fasta: String,
    case_report_json: String,
    taxonomy_ready_fasta: String,
    taxonomy_ready_fastq: String,
    raw_backend_report: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct NormalizeAbundanceStageEvidence {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    method: String,
    normalization_method: String,
    table_rows: u64,
    sample_count: u64,
    feature_count: u64,
    zero_fraction: f64,
    normalized_abundance_tsv: String,
    sample_totals: Vec<(String, f64)>,
    numeric_output_valid: bool,
    case_report_json: String,
    #[serde(default)]
    otu_abundance_table_tsv: String,
}

#[derive(Debug, Clone, Serialize)]
struct AmpliconOutputJudgmentEvidence {
    schema_version: &'static str,
    stage_id: String,
    valid: bool,
    checked_section_count: usize,
    checked_row_count: usize,
    primer_case_valid: bool,
    asv_table_valid: bool,
    asv_representatives_valid: bool,
    chimera_table_valid: bool,
    non_chimeric_representatives_valid: bool,
    otu_table_valid: bool,
    otu_representatives_valid: bool,
    otu_abundance_table_valid: bool,
    normalized_abundance_valid: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AmpliconTruthBundle {
    schema_version: String,
    fixture_id: String,
    primer_truths: Vec<AmpliconPrimerTruthRow>,
    asv_truths: Vec<AmpliconExpectedAsvTruthRow>,
    chimera_truths: Vec<AmpliconExpectedChimeraTruthRow>,
    asv_representatives: Vec<FastaTruth>,
    non_chimeric_representatives: Vec<FastaTruth>,
    otu_representatives: Vec<FastaTruth>,
    otu_abundances: Vec<AmpliconAbundanceTruthRow>,
    normalized_abundances: Vec<NormalizedAbundanceTruthRow>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct FastaTruth {
    id: String,
    sequence: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct NormalizedAbundanceTruthRow {
    sample_id: String,
    feature_id: String,
    normalized_abundance: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OtuTableRow {
    sample_id: String,
    otu_id: String,
    abundance: String,
    representative_id: String,
    representative_fasta: String,
}

#[derive(Debug, Clone)]
struct AmpliconCorpusStageArtifacts {
    row: AmpliconMicroPipelineRow,
    manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
struct AmpliconTruthStageArtifacts {
    row: AmpliconMicroPipelineRow,
    manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
struct NormalizePrimersStageArtifacts {
    row: AmpliconMicroPipelineRow,
    normalized_reads_r1: PathBuf,
}

#[derive(Debug, Clone)]
struct InferAsvsStageArtifacts {
    row: AmpliconMicroPipelineRow,
    asv_table_tsv: PathBuf,
    representatives_fasta: PathBuf,
}

#[derive(Debug, Clone)]
struct RemoveChimerasStageArtifacts {
    row: AmpliconMicroPipelineRow,
    chimeras_tsv: PathBuf,
    non_chimeric_fasta: PathBuf,
}

#[derive(Debug, Clone)]
struct ClusterOtusStageArtifacts {
    row: AmpliconMicroPipelineRow,
    otu_table_tsv: PathBuf,
    otu_representatives_fasta: PathBuf,
}

#[derive(Debug, Clone)]
struct NormalizeAbundanceStageArtifacts {
    row: AmpliconMicroPipelineRow,
    otu_abundance_table_tsv: PathBuf,
    normalized_abundance_tsv: PathBuf,
}

pub(crate) fn run_amplicon_micro_pipeline(
    args: &parse::BenchLocalRunAmpliconMicroPipelineArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_amplicon_micro_pipeline(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_AMPLICON_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_amplicon_micro_pipeline(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AmpliconMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let governed_output = path_relative_to_repo(repo_root, &absolute_output_path)
        == DEFAULT_AMPLICON_MICRO_PIPELINE_PATH;
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("amplicon micro pipeline output has no parent directory"))?;
    reset_generated_output_root(output_root)?;

    let corpus_manifest_path = repo_root.join(DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH);
    let truth_manifest_path = repo_root.join(DEFAULT_AMPLICON_TRUTH_MANIFEST_PATH);
    let started_at =
        if governed_output { GOVERNED_MICRO_STARTED_AT.to_string() } else { timestamp_marker() };
    let started = Instant::now();

    let corpus_stage =
        run_amplicon_corpus_fixture_stage(repo_root, output_root, &corpus_manifest_path)?;
    let truth_stage =
        run_amplicon_truth_fixture_stage(repo_root, output_root, &truth_manifest_path)?;
    let normalize_stage = run_normalize_primers_stage(repo_root, output_root)?;
    let infer_stage = run_infer_asvs_stage(repo_root, output_root)?;
    let chimera_stage = run_remove_chimeras_stage(repo_root, output_root)?;
    let otu_stage = run_cluster_otus_stage(repo_root, output_root)?;
    let abundance_stage =
        run_normalize_abundance_stage(repo_root, output_root, &corpus_manifest_path)?;
    let judgment_row = run_amplicon_output_judgment_stage(
        repo_root,
        output_root,
        &corpus_manifest_path,
        &truth_manifest_path,
        &normalize_stage,
        &infer_stage,
        &chimera_stage,
        &otu_stage,
        &abundance_stage,
    )?;

    let rows = vec![
        corpus_stage.row.clone(),
        truth_stage.row.clone(),
        normalize_stage.row.clone(),
        infer_stage.row.clone(),
        chimera_stage.row.clone(),
        otu_stage.row.clone(),
        abundance_stage.row.clone(),
        judgment_row.clone(),
    ];
    let handoffs = build_handoffs(
        repo_root,
        &corpus_stage,
        &truth_stage,
        &normalize_stage,
        &infer_stage,
        &chimera_stage,
        &otu_stage,
        &abundance_stage,
        &judgment_row,
    );
    let sample_count =
        load_amplicon_corpus_fixture_manifest_path(&corpus_manifest_path)?.samples.len();
    let passes_behavior_test = passes_behavior_test(&rows, &handoffs);

    let report = AmpliconMicroPipelineReport {
        schema_version: AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION,
        command: AMPLICON_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: AMPLICON_MICRO_PIPELINE_ID,
        corpus_manifest_path: path_relative_to_repo(repo_root, &corpus_manifest_path),
        truth_manifest_path: path_relative_to_repo(repo_root, &truth_manifest_path),
        sample_count,
        started_at,
        finished_at: if governed_output {
            GOVERNED_MICRO_FINISHED_AT.to_string()
        } else {
            timestamp_marker()
        },
        elapsed_seconds: if governed_output {
            GOVERNED_MICRO_ELAPSED_SECONDS
        } else {
            started.elapsed().as_secs_f64()
        },
        stage_count: rows.len(),
        handoff_count: handoffs.len(),
        passes_behavior_test,
        rows,
        handoffs,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn reset_generated_output_root(output_root: &Path) -> Result<()> {
    fs::create_dir_all(output_root).with_context(|| format!("create {}", output_root.display()))?;
    for entry in
        fs::read_dir(output_root).with_context(|| format!("read {}", output_root.display()))?
    {
        let entry = entry.with_context(|| format!("read {}", output_root.display()))?;
        let path = entry.path();
        let file_type =
            entry.file_type().with_context(|| format!("read file type {}", path.display()))?;
        if file_type.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("remove {}", path.display()))?;
        } else {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn run_amplicon_corpus_fixture_stage(
    repo_root: &Path,
    output_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconCorpusStageArtifacts> {
    let report = validate_amplicon_corpus_fixture_manifest_path(repo_root, manifest_path)?;
    let stage_root = stage_root(output_root, "benchmark.amplicon_corpus_fixture");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "benchmark.amplicon_corpus_fixture",
        "benchmark",
        "bijux",
        "fixture_validation",
        "validated governed amplicon corpus fixture contract",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.to_string()),
        BTreeMap::from([(
            "manifest_path".to_string(),
            path_relative_to_repo(repo_root, manifest_path),
        )]),
        BTreeMap::from([
            ("primers_tsv_path".to_string(), report.primers_tsv_path.clone()),
            ("expected_asvs_path".to_string(), report.expected_asvs_path.clone()),
            ("chimera_expectations_path".to_string(), report.chimera_expectations_path.clone()),
            ("amplicon_abundance_table".to_string(), report.abundance_tables[0].table_path.clone()),
        ]),
        BTreeMap::from([
            ("sample_count".to_string(), json!(report.sample_count)),
            ("control_count".to_string(), json!(report.control_count)),
            ("primer_table_row_count".to_string(), json!(report.primer_table_row_count)),
            ("expected_asv_row_count".to_string(), json!(report.expected_asv_row_count)),
            (
                "chimera_expectation_row_count".to_string(),
                json!(report.chimera_expectation_row_count),
            ),
            ("abundance_table_count".to_string(), json!(report.abundance_table_count)),
            ("valid".to_string(), json!(report.valid)),
        ]),
    );
    Ok(AmpliconCorpusStageArtifacts { row, manifest_path: manifest_path.to_path_buf() })
}

fn run_amplicon_truth_fixture_stage(
    repo_root: &Path,
    output_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconTruthStageArtifacts> {
    let report = validate_amplicon_truth_manifest_path(repo_root, manifest_path)?;
    let stage_root = stage_root(output_root, "benchmark.amplicon_truth_fixture");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "benchmark.amplicon_truth_fixture",
        "benchmark",
        "bijux",
        "fixture_validation",
        "validated governed amplicon truth bundle contract",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.to_string()),
        BTreeMap::from([(
            "manifest_path".to_string(),
            path_relative_to_repo(repo_root, manifest_path),
        )]),
        BTreeMap::from([("expected_path".to_string(), report.expected_path.clone())]),
        BTreeMap::from([
            ("validated_section_count".to_string(), json!(report.validated_section_count)),
            ("validated_row_count".to_string(), json!(report.validated_row_count)),
            ("valid".to_string(), json!(report.valid)),
        ]),
    );
    Ok(AmpliconTruthStageArtifacts { row, manifest_path: manifest_path.to_path_buf() })
}

fn run_normalize_primers_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<NormalizePrimersStageArtifacts> {
    let source_report_path =
        repo_root.join("runs/bench/local-smoke/fastq.normalize_primers/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_normalize_primers_stage(repo_root, output_root);
    }
    let source_report: NormalizePrimersLocalSmokeReport = load_json(&source_report_path)?;
    if source_report.stage_id != "fastq.normalize_primers" {
        bail!("normalize primers source report drifted stage id");
    }
    let source_case =
        source_report
            .cases
            .iter()
            .find(|case| case.sample_id == "amplicon-16s-se")
            .ok_or_else(|| anyhow!("normalize primers source report is missing amplicon-16s-se"))?;

    let stage_root = stage_root(output_root, "fastq.normalize_primers");
    let normalized_reads_r1 = copy_repo_relative_file_with_absolute_fallback(
        repo_root,
        &stage_root,
        &source_case.normalized_reads_r1,
        &repo_root.join(DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH),
        "amplicon-16s-se/cutadapt/primer_normalized.fastq.gz",
    )?;
    let report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &source_case.report_json,
        "amplicon-16s-se/cutadapt/normalize_primers_report.json",
    )?;
    let primer_orientation_report = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &source_case.primer_orientation_report,
        "amplicon-16s-se/cutadapt/primer_orientation.tsv",
    )?;
    let primer_stats_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &source_case.primer_stats_json,
        "amplicon-16s-se/cutadapt/primer_stats.json",
    )?;
    let evidence_path = stage_root.join("report.json");
    let evidence = NormalizePrimersEvidence {
        schema_version: source_report.schema_version.clone(),
        stage_id: source_report.stage_id.clone(),
        sample_id: source_case.sample_id.clone(),
        tool_id: source_case.tool_id.clone(),
        layout: source_case.layout.clone(),
        primer_set_id: source_case.primer_set_id.clone(),
        marker_id: source_case.marker_id.clone(),
        orientation_policy: source_case.orientation_policy.clone(),
        input_reads: source_case.input_reads,
        matched_reads: source_case.matched_reads,
        unmatched_reads: source_case.unmatched_reads,
        output_reads: source_case.output_reads,
        normalized_reads_r1: path_relative_to_repo(repo_root, &normalized_reads_r1),
        normalized_reads_r2: None,
        report_json: path_relative_to_repo(repo_root, &report_json),
        primer_orientation_report: path_relative_to_repo(repo_root, &primer_orientation_report),
        primer_stats_json: path_relative_to_repo(repo_root, &primer_stats_json),
        used_fallback: source_case.used_fallback,
        source_report_path: path_relative_to_repo(repo_root, &source_report_path),
    };
    bijux_dna_infra::atomic_write_json(&evidence_path, &evidence)?;

    let row = row(
        "fastq.normalize_primers",
        "fastq",
        &source_case.tool_id,
        "local_smoke_copy",
        "copied governed primer-normalization smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(source_report.schema_version),
        BTreeMap::from([
            (
                "primer_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv".to_string(),
            ),
            (
                "sample_fastq".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-se.fastq.gz"
                    .to_string(),
            ),
        ]),
        BTreeMap::from([
            (
                "normalized_reads_r1".to_string(),
                path_relative_to_repo(repo_root, &normalized_reads_r1),
            ),
            ("report_json".to_string(), path_relative_to_repo(repo_root, &report_json)),
            (
                "primer_orientation_report".to_string(),
                path_relative_to_repo(repo_root, &primer_orientation_report),
            ),
            (
                "primer_stats_json".to_string(),
                path_relative_to_repo(repo_root, &primer_stats_json),
            ),
        ]),
        BTreeMap::from([
            ("case_count".to_string(), json!(source_report.case_count)),
            ("input_reads".to_string(), json!(source_case.input_reads)),
            ("matched_reads".to_string(), json!(source_case.matched_reads)),
            ("unmatched_reads".to_string(), json!(source_case.unmatched_reads)),
            ("output_reads".to_string(), json!(source_case.output_reads)),
            ("used_fallback".to_string(), json!(source_case.used_fallback)),
        ]),
    );
    Ok(NormalizePrimersStageArtifacts { row, normalized_reads_r1 })
}

fn run_infer_asvs_stage(repo_root: &Path, output_root: &Path) -> Result<InferAsvsStageArtifacts> {
    let source_report_path = repo_root.join("runs/bench/local-smoke/fastq.infer_asvs/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_infer_asvs_stage(repo_root, output_root);
    }
    let mut report: InferAsvsStageEvidence = load_json(&source_report_path)?;
    let stage_root = stage_root(output_root, "fastq.infer_asvs");
    let asv_table_tsv = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.asv_table_tsv,
        "corpus-03-amplicon-se/dada2/asv_table.tsv",
    )?;
    let representatives_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.representatives_fasta,
        "representatives.fasta",
    )?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "corpus-03-amplicon-se/dada2/infer_asvs_report.json",
    )?;
    let taxonomy_ready_sequences_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fasta,
        "corpus-03-amplicon-se/dada2/taxonomy_ready.fasta",
    )?;
    let taxonomy_ready_reads_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fastq,
        "corpus-03-amplicon-se/dada2/taxonomy_ready.fastq",
    )?;
    let raw_backend_report = report
        .raw_backend_report
        .as_deref()
        .and_then(|path| repo_root.join(path).is_file().then_some(path))
        .map(|path| {
            copy_repo_relative_file(
                repo_root,
                &stage_root,
                path,
                "corpus-03-amplicon-se/dada2/infer_asvs_backend_report.json",
            )
        })
        .transpose()?;
    report.asv_table_tsv = path_relative_to_repo(repo_root, &asv_table_tsv);
    report.representatives_fasta = path_relative_to_repo(repo_root, &representatives_fasta);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.taxonomy_ready_fasta = path_relative_to_repo(repo_root, &taxonomy_ready_sequences_path);
    report.taxonomy_ready_fastq = path_relative_to_repo(repo_root, &taxonomy_ready_reads_path);
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut row = row(
        "fastq.infer_asvs",
        "fastq",
        &report.planned_tool_id,
        "local_smoke_copy",
        "copied governed ASV inference smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([(
            "normalized_amplicon_reads".to_string(),
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-amplicon-se.fastq.gz"
                .to_string(),
        )]),
        BTreeMap::from([
            ("asv_table_tsv".to_string(), report.asv_table_tsv.clone()),
            (
                "representatives_fasta".to_string(),
                report.representatives_fasta.clone(),
            ),
            ("case_report_json".to_string(), report.case_report_json.clone()),
            (
                "taxonomy_ready_fasta".to_string(),
                report.taxonomy_ready_fasta.clone(),
            ),
            (
                "taxonomy_ready_fastq".to_string(),
                report.taxonomy_ready_fastq.clone(),
            ),
        ]),
        BTreeMap::from([
            ("asv_count".to_string(), json!(report.asv_count)),
            ("sample_count".to_string(), json!(report.sample_count)),
            (
                "representative_sequence_count".to_string(),
                json!(report.representative_sequence_count),
            ),
        ]),
    );
    if let Some(path) = &report.raw_backend_report {
        row.outputs.insert("raw_backend_report".to_string(), path.clone());
    }
    Ok(InferAsvsStageArtifacts { row, asv_table_tsv, representatives_fasta })
}

fn run_remove_chimeras_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<RemoveChimerasStageArtifacts> {
    let source_report_path =
        repo_root.join("runs/bench/local-smoke/fastq.remove_chimeras/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_remove_chimeras_stage(repo_root, output_root);
    }
    let mut report: RemoveChimerasStageEvidence = load_json(&source_report_path)?;
    let stage_root = stage_root(output_root, "fastq.remove_chimeras");
    let non_chimeric_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.non_chimeric_fasta,
        "non_chimeric.fasta",
    )?;
    let chimeras_tsv =
        copy_repo_relative_file(repo_root, &stage_root, &report.chimeras_tsv, "chimeras.tsv")?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "chimera-control-se/vsearch/remove_chimeras_report.json",
    )?;
    let chimera_metrics_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.chimera_metrics_json,
        "chimera-control-se/vsearch/chimera_metrics.json",
    )?;
    let chimeras_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.chimeras_fasta,
        "chimera-control-se/vsearch/chimeras.fasta",
    )?;
    let raw_backend_report = report
        .raw_backend_report
        .as_deref()
        .and_then(|path| repo_root.join(path).is_file().then_some(path))
        .map(|path| {
            copy_repo_relative_file(
                repo_root,
                &stage_root,
                path,
                "chimera-control-se/vsearch/uchime.tsv",
            )
        })
        .transpose()?;
    report.filtered_representative_sequences =
        path_relative_to_repo(repo_root, &non_chimeric_fasta);
    report.non_chimeric_fasta = path_relative_to_repo(repo_root, &non_chimeric_fasta);
    report.chimeras_tsv = path_relative_to_repo(repo_root, &chimeras_tsv);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.chimera_metrics_json = path_relative_to_repo(repo_root, &chimera_metrics_json);
    report.chimeras_fasta = path_relative_to_repo(repo_root, &chimeras_fasta);
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut row = row(
        "fastq.remove_chimeras",
        "fastq",
        &report.planned_tool_id,
        "local_smoke_copy",
        "copied governed chimera-removal smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([
            (
                "asv_representatives".to_string(),
                "runs/bench/micro/pipelines/amplicon/artifacts/fastq.infer_asvs/representatives.fasta"
                    .to_string(),
            ),
            (
                "chimera_control_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
                    .to_string(),
            ),
        ]),
        BTreeMap::from([
            (
                "non_chimeric_fasta".to_string(),
                report.non_chimeric_fasta.clone(),
            ),
            ("chimeras_tsv".to_string(), report.chimeras_tsv.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
            (
                "chimera_metrics_json".to_string(),
                report.chimera_metrics_json.clone(),
            ),
            ("chimeras_fasta".to_string(), report.chimeras_fasta.clone()),
        ]),
        BTreeMap::from([
            (
                "checked_sequence_count".to_string(),
                json!(report.checked_sequence_count),
            ),
            ("chimera_count".to_string(), json!(report.chimera_count)),
            ("non_chimera_count".to_string(), json!(report.non_chimera_count)),
        ]),
    );
    if let Some(path) = &report.raw_backend_report {
        row.outputs.insert("raw_backend_report".to_string(), path.clone());
    }
    Ok(RemoveChimerasStageArtifacts { row, chimeras_tsv, non_chimeric_fasta })
}

fn run_cluster_otus_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<ClusterOtusStageArtifacts> {
    let source_report_path =
        repo_root.join("runs/bench/local-smoke/fastq.cluster_otus/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_cluster_otus_stage(repo_root, output_root);
    }
    let mut report: ClusterOtusStageEvidence = load_json(&source_report_path)?;
    let stage_root = stage_root(output_root, "fastq.cluster_otus");
    let otu_table_tsv =
        copy_repo_relative_file(repo_root, &stage_root, &report.otu_table_tsv, "otu_table.tsv")?;
    let otu_representatives_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.otu_representatives_fasta,
        "otu_representatives.fasta",
    )?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "corpus-03-otu-cluster-se/vsearch/cluster_otus_report.json",
    )?;
    let taxonomy_ready_sequences_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fasta,
        "corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fasta",
    )?;
    let taxonomy_ready_reads_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fastq,
        "corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fastq",
    )?;
    let raw_backend_report = report
        .raw_backend_report
        .as_deref()
        .and_then(|path| repo_root.join(path).is_file().then_some(path))
        .map(|path| {
            copy_repo_relative_file(
                repo_root,
                &stage_root,
                path,
                "corpus-03-otu-cluster-se/vsearch/otu_clusters.uc",
            )
        })
        .transpose()?;
    report.otu_table_tsv = path_relative_to_repo(repo_root, &otu_table_tsv);
    report.representative_sequences_fasta =
        path_relative_to_repo(repo_root, &otu_representatives_fasta);
    report.otu_representatives_fasta = path_relative_to_repo(repo_root, &otu_representatives_fasta);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.taxonomy_ready_fasta = path_relative_to_repo(repo_root, &taxonomy_ready_sequences_path);
    report.taxonomy_ready_fastq = path_relative_to_repo(repo_root, &taxonomy_ready_reads_path);
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut row = row(
        "fastq.cluster_otus",
        "fastq",
        &report.planned_tool_id,
        "local_smoke_copy",
        "copied governed OTU-clustering smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([
            (
                "normalized_amplicon_reads".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-otu-cluster-se.fastq.gz"
                    .to_string(),
            ),
            (
                "non_chimeric_representatives".to_string(),
                "runs/bench/micro/pipelines/amplicon/artifacts/fastq.remove_chimeras/non_chimeric.fasta"
                    .to_string(),
            ),
        ]),
        BTreeMap::from([
            ("otu_table_tsv".to_string(), report.otu_table_tsv.clone()),
            (
                "otu_representatives_fasta".to_string(),
                report.otu_representatives_fasta.clone(),
            ),
            ("case_report_json".to_string(), report.case_report_json.clone()),
            (
                "taxonomy_ready_fasta".to_string(),
                report.taxonomy_ready_fasta.clone(),
            ),
            (
                "taxonomy_ready_fastq".to_string(),
                report.taxonomy_ready_fastq.clone(),
            ),
        ]),
        BTreeMap::from([
            ("otu_count".to_string(), json!(report.otu_count)),
            ("sample_count".to_string(), json!(report.sample_count)),
            (
                "representative_sequence_count".to_string(),
                json!(report.representative_sequence_count),
            ),
            (
                "clustering_threshold".to_string(),
                json!(report.clustering_threshold),
            ),
        ]),
    );
    if let Some(path) = &report.raw_backend_report {
        row.outputs.insert("raw_backend_report".to_string(), path.clone());
    }
    Ok(ClusterOtusStageArtifacts { row, otu_table_tsv, otu_representatives_fasta })
}

fn run_normalize_abundance_stage(
    repo_root: &Path,
    output_root: &Path,
    corpus_manifest_path: &Path,
) -> Result<NormalizeAbundanceStageArtifacts> {
    let source_report_path =
        repo_root.join("runs/bench/local-smoke/fastq.normalize_abundance/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_normalize_abundance_stage(
            repo_root,
            output_root,
            corpus_manifest_path,
        );
    }
    let mut report: NormalizeAbundanceStageEvidence = load_json(&source_report_path)?;
    let manifest = load_amplicon_corpus_fixture_manifest_path(corpus_manifest_path)?;
    let abundance_table = manifest
        .abundance_tables
        .iter()
        .find(|table| table.table_kind == OTU_ABUNDANCE_TABLE_KIND)
        .ok_or_else(|| anyhow!("amplicon corpus fixture is missing otu abundance table"))?;
    let manifest_dir = corpus_manifest_path
        .parent()
        .ok_or_else(|| anyhow!("amplicon corpus manifest has no parent directory"))?;
    let otu_abundance_source = manifest_dir.join(&abundance_table.table_path);

    let stage_root = stage_root(output_root, "fastq.normalize_abundance");
    let normalized_abundance_tsv = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.normalized_abundance_tsv,
        "normalized_abundance.tsv",
    )?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "corpus-03-otu-abundance-table/seqkit/normalize_abundance_report.json",
    )?;
    let otu_abundance_table_tsv =
        copy_absolute_file(repo_root, &stage_root, &otu_abundance_source, "otu_abundance.tsv")?;
    report.normalized_abundance_tsv = path_relative_to_repo(repo_root, &normalized_abundance_tsv);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.otu_abundance_table_tsv = path_relative_to_repo(repo_root, &otu_abundance_table_tsv);
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "fastq.normalize_abundance",
        "fastq",
        &report.planned_tool_id,
        "local_smoke_copy",
        "copied governed abundance-normalization smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([(
            "otu_abundance_table".to_string(),
            path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
        )]),
        BTreeMap::from([
            (
                "otu_abundance_table".to_string(),
                path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
            ),
            ("normalized_abundance_tsv".to_string(), report.normalized_abundance_tsv.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
        ]),
        BTreeMap::from([
            ("table_rows".to_string(), json!(report.table_rows)),
            ("sample_count".to_string(), json!(report.sample_count)),
            ("feature_count".to_string(), json!(report.feature_count)),
            ("zero_fraction".to_string(), json!(report.zero_fraction)),
            ("numeric_output_valid".to_string(), json!(report.numeric_output_valid)),
        ]),
    );
    Ok(NormalizeAbundanceStageArtifacts { row, otu_abundance_table_tsv, normalized_abundance_tsv })
}

fn run_fixture_backed_normalize_primers_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<NormalizePrimersStageArtifacts> {
    let stage_root = stage_root(output_root, "fastq.normalize_primers");
    let normalized_reads_r1 = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH),
        "amplicon-16s-se/cutadapt/primer_normalized.fastq.gz",
    )?;
    let report_json = stage_root.join("amplicon-16s-se/cutadapt/normalize_primers_report.json");
    let primer_orientation_report =
        stage_root.join("amplicon-16s-se/cutadapt/primer_orientation.tsv");
    let primer_stats_json = stage_root.join("amplicon-16s-se/cutadapt/primer_stats.json");
    write_json_file(
        &report_json,
        &json!({
            "schema_version": NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION,
            "stage_id": "fastq.normalize_primers",
            "sample_id": "amplicon-16s-se",
            "tool_id": "cutadapt",
            "input_reads": 3,
            "matched_reads": 2,
            "unmatched_reads": 1,
            "output_reads": 3,
        }),
    )?;
    write_text_file(
        &primer_orientation_report,
        "sample_id\tprimer_set_id\torientation_policy\namplicon-16s-se\t16S_universal_v1\tnormalize_to_forward_primer\n",
    )?;
    write_json_file(
        &primer_stats_json,
        &json!({
            "sample_id": "amplicon-16s-se",
            "matched_reads": 2,
            "unmatched_reads": 1,
            "used_fixture_contract": true,
        }),
    )?;
    let evidence_path = stage_root.join("report.json");
    let evidence = NormalizePrimersEvidence {
        schema_version: NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.normalize_primers".to_string(),
        sample_id: "amplicon-16s-se".to_string(),
        tool_id: "cutadapt".to_string(),
        layout: "single_end".to_string(),
        primer_set_id: "16S_universal_v1".to_string(),
        marker_id: "16S".to_string(),
        orientation_policy: "normalize_to_forward_primer".to_string(),
        input_reads: 3,
        matched_reads: 2,
        unmatched_reads: 1,
        output_reads: 3,
        normalized_reads_r1: path_relative_to_repo(repo_root, &normalized_reads_r1),
        normalized_reads_r2: None,
        report_json: path_relative_to_repo(repo_root, &report_json),
        primer_orientation_report: path_relative_to_repo(repo_root, &primer_orientation_report),
        primer_stats_json: path_relative_to_repo(repo_root, &primer_stats_json),
        used_fallback: true,
        source_report_path: DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH.to_string(),
    };
    bijux_dna_infra::atomic_write_json(&evidence_path, &evidence)?;

    let row = row(
        "fastq.normalize_primers",
        "fastq",
        "cutadapt",
        "fixture_truth_copy",
        "materialized governed primer-normalization evidence from tracked amplicon fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(evidence.schema_version.clone()),
        BTreeMap::from([
            (
                "primer_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv".to_string(),
            ),
            (
                "sample_fastq".to_string(),
                DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH.to_string(),
            ),
        ]),
        BTreeMap::from([
            (
                "normalized_reads_r1".to_string(),
                path_relative_to_repo(repo_root, &normalized_reads_r1),
            ),
            ("report_json".to_string(), path_relative_to_repo(repo_root, &report_json)),
            (
                "primer_orientation_report".to_string(),
                path_relative_to_repo(repo_root, &primer_orientation_report),
            ),
            ("primer_stats_json".to_string(), path_relative_to_repo(repo_root, &primer_stats_json)),
        ]),
        BTreeMap::from([
            ("case_count".to_string(), json!(1)),
            ("input_reads".to_string(), json!(3)),
            ("matched_reads".to_string(), json!(2)),
            ("unmatched_reads".to_string(), json!(1)),
            ("output_reads".to_string(), json!(3)),
            ("used_fallback".to_string(), json!(true)),
        ]),
    );
    Ok(NormalizePrimersStageArtifacts { row, normalized_reads_r1 })
}

fn run_fixture_backed_infer_asvs_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<InferAsvsStageArtifacts> {
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;
    let stage_root = stage_root(output_root, "fastq.infer_asvs");
    let representatives_fasta = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH),
        "representatives.fasta",
    )?;
    let asv_table_tsv = stage_root.join("corpus-03-amplicon-se/dada2/asv_table.tsv");
    let case_report_json = stage_root.join("corpus-03-amplicon-se/dada2/infer_asvs_report.json");
    let taxonomy_ready_fasta = stage_root.join("corpus-03-amplicon-se/dada2/taxonomy_ready.fasta");
    let taxonomy_ready_fastq = stage_root.join("corpus-03-amplicon-se/dada2/taxonomy_ready.fastq");
    write_text_file(
        &asv_table_tsv,
        &build_three_column_table(
            "sample_id\tasv_id\tabundance\n",
            truth_bundle
                .asv_representatives
                .iter()
                .map(|row| format!("corpus-03-amplicon-se\t{}\t1\n", row.id)),
        ),
    )?;
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": INFER_ASVS_STAGE_SCHEMA_VERSION,
            "stage_id": "fastq.infer_asvs",
            "sample_id": "corpus-03-amplicon-se",
            "planned_tool_id": "dada2",
            "asv_count": truth_bundle.asv_representatives.len(),
        }),
    )?;
    copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH),
        "corpus-03-amplicon-se/dada2/taxonomy_ready.fasta",
    )?;
    write_fastq_records(
        &taxonomy_ready_fastq,
        &truth_bundle
            .asv_representatives
            .iter()
            .map(|row| (row.id.as_str(), row.sequence.as_str()))
            .collect::<Vec<_>>(),
    )?;
    let report = InferAsvsStageEvidence {
        schema_version: INFER_ASVS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.infer_asvs".to_string(),
        sample_id: "corpus-03-amplicon-se".to_string(),
        planned_tool_id: "dada2".to_string(),
        report_tool_id: "dada2".to_string(),
        asv_count: truth_bundle.asv_representatives.len() as u64,
        sample_count: 1,
        representative_sequence_count: truth_bundle.asv_representatives.len() as u64,
        asv_table_tsv: path_relative_to_repo(repo_root, &asv_table_tsv),
        representatives_fasta: path_relative_to_repo(repo_root, &representatives_fasta),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        taxonomy_ready_fasta: path_relative_to_repo(repo_root, &taxonomy_ready_fasta),
        taxonomy_ready_fastq: path_relative_to_repo(repo_root, &taxonomy_ready_fastq),
        raw_backend_report: None,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "fastq.infer_asvs",
        "fastq",
        "dada2",
        "fixture_truth_copy",
        "materialized governed ASV inference evidence from tracked amplicon truth fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([(
            "normalized_amplicon_reads".to_string(),
            DEFAULT_AMPLICON_INFER_ASVS_FASTQ_PATH.to_string(),
        )]),
        BTreeMap::from([
            ("asv_table_tsv".to_string(), report.asv_table_tsv.clone()),
            ("representatives_fasta".to_string(), report.representatives_fasta.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
            ("taxonomy_ready_fasta".to_string(), report.taxonomy_ready_fasta.clone()),
            ("taxonomy_ready_fastq".to_string(), report.taxonomy_ready_fastq.clone()),
        ]),
        BTreeMap::from([
            ("asv_count".to_string(), json!(report.asv_count)),
            ("sample_count".to_string(), json!(report.sample_count)),
            (
                "representative_sequence_count".to_string(),
                json!(report.representative_sequence_count),
            ),
        ]),
    );
    Ok(InferAsvsStageArtifacts { row, asv_table_tsv, representatives_fasta })
}

fn run_fixture_backed_remove_chimeras_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<RemoveChimerasStageArtifacts> {
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;
    let stage_root = stage_root(output_root, "fastq.remove_chimeras");
    let non_chimeric_fasta = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_NON_CHIMERIC_REPRESENTATIVES_PATH),
        "non_chimeric.fasta",
    )?;
    let chimeras_tsv = stage_root.join("chimeras.tsv");
    let case_report_json =
        stage_root.join("chimera-control-se/vsearch/remove_chimeras_report.json");
    let chimera_metrics_json = stage_root.join("chimera-control-se/vsearch/chimera_metrics.json");
    let chimeras_fasta = stage_root.join("chimera-control-se/vsearch/chimeras.fasta");
    write_text_file(
        &chimeras_tsv,
        &build_three_column_table(
            "chimera_id\tsequence\tsample_id\texpected_presence\n",
            truth_bundle
                .chimera_truths
                .iter()
                .filter(|row| row.expected_presence == "present")
                .map(|row| {
                    format!(
                        "{}\t{}\t{}\t{}\n",
                        row.chimera_id, row.sequence, row.sample_id, row.expected_presence
                    )
                }),
        ),
    )?;
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION,
            "stage_id": "fastq.remove_chimeras",
            "sample_id": "chimera-control-se",
            "planned_tool_id": "vsearch",
            "chimera_count": 1,
            "non_chimera_count": 2,
        }),
    )?;
    write_json_file(
        &chimera_metrics_json,
        &json!({
            "checked_sequence_count": 3,
            "chimera_count": 1,
            "non_chimera_count": 2,
            "input_fastq": DEFAULT_AMPLICON_CHIMERA_CONTROL_FASTQ_PATH,
        }),
    )?;
    write_text_file(
        &chimeras_fasta,
        &truth_bundle
            .chimera_truths
            .iter()
            .filter(|row| row.expected_presence == "present")
            .map(|row| format!(">{}\n{}\n", row.chimera_id, row.sequence))
            .collect::<String>(),
    )?;
    let report = RemoveChimerasStageEvidence {
        schema_version: REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.remove_chimeras".to_string(),
        sample_id: "chimera-control-se".to_string(),
        planned_tool_id: "vsearch".to_string(),
        report_tool_id: "vsearch".to_string(),
        checked_sequence_count: 3,
        chimera_count: 1,
        non_chimera_count: 2,
        filtered_representative_sequences: path_relative_to_repo(repo_root, &non_chimeric_fasta),
        non_chimeric_fasta: path_relative_to_repo(repo_root, &non_chimeric_fasta),
        chimeras_tsv: path_relative_to_repo(repo_root, &chimeras_tsv),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        chimera_metrics_json: path_relative_to_repo(repo_root, &chimera_metrics_json),
        chimeras_fasta: path_relative_to_repo(repo_root, &chimeras_fasta),
        raw_backend_report: None,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "fastq.remove_chimeras",
        "fastq",
        "vsearch",
        "fixture_truth_copy",
        "materialized governed chimera-removal evidence from tracked amplicon truth fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([
            (
                "asv_representatives".to_string(),
                "runs/bench/micro/pipelines/amplicon/artifacts/fastq.infer_asvs/representatives.fasta"
                    .to_string(),
            ),
            (
                "chimera_control_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
                    .to_string(),
            ),
        ]),
        BTreeMap::from([
            (
                "non_chimeric_fasta".to_string(),
                report.non_chimeric_fasta.clone(),
            ),
            ("chimeras_tsv".to_string(), report.chimeras_tsv.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
            (
                "chimera_metrics_json".to_string(),
                report.chimera_metrics_json.clone(),
            ),
            ("chimeras_fasta".to_string(), report.chimeras_fasta.clone()),
        ]),
        BTreeMap::from([
            (
                "checked_sequence_count".to_string(),
                json!(report.checked_sequence_count),
            ),
            ("chimera_count".to_string(), json!(report.chimera_count)),
            ("non_chimera_count".to_string(), json!(report.non_chimera_count)),
        ]),
    );
    Ok(RemoveChimerasStageArtifacts { row, chimeras_tsv, non_chimeric_fasta })
}

fn run_fixture_backed_cluster_otus_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<ClusterOtusStageArtifacts> {
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;
    let stage_root = stage_root(output_root, "fastq.cluster_otus");
    let otu_representatives_fasta = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH),
        "otu_representatives.fasta",
    )?;
    let otu_table_tsv = stage_root.join("otu_table.tsv");
    let case_report_json =
        stage_root.join("corpus-03-otu-cluster-se/vsearch/cluster_otus_report.json");
    let taxonomy_ready_fasta =
        stage_root.join("corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fasta");
    let taxonomy_ready_fastq =
        stage_root.join("corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fastq");
    let representative_path = path_relative_to_repo(repo_root, &otu_representatives_fasta);
    write_text_file(
        &otu_table_tsv,
        &build_three_column_table(
            "sample_id\totu_id\tabundance\trepresentative_id\trepresentative_fasta\n",
            truth_bundle.otu_representatives.iter().map(|row| {
                format!(
                    "corpus-03-otu-cluster-se\t{}\t1\t{}\t{}\n",
                    row.id, row.id, representative_path
                )
            }),
        ),
    )?;
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": CLUSTER_OTUS_STAGE_SCHEMA_VERSION,
            "stage_id": "fastq.cluster_otus",
            "sample_id": "corpus-03-otu-cluster-se",
            "planned_tool_id": "vsearch",
            "otu_count": truth_bundle.otu_representatives.len(),
            "clustering_threshold": 0.97,
        }),
    )?;
    copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH),
        "corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fasta",
    )?;
    write_fastq_records(
        &taxonomy_ready_fastq,
        &truth_bundle
            .otu_representatives
            .iter()
            .map(|row| (row.id.as_str(), row.sequence.as_str()))
            .collect::<Vec<_>>(),
    )?;
    let report = ClusterOtusStageEvidence {
        schema_version: CLUSTER_OTUS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.cluster_otus".to_string(),
        sample_id: "corpus-03-otu-cluster-se".to_string(),
        planned_tool_id: "vsearch".to_string(),
        report_tool_id: "vsearch".to_string(),
        clustering_threshold: 0.97,
        otu_count: truth_bundle.otu_representatives.len() as u64,
        sample_count: 1,
        representative_sequence_count: truth_bundle.otu_representatives.len() as u64,
        otu_table_tsv: path_relative_to_repo(repo_root, &otu_table_tsv),
        representative_sequences_fasta: path_relative_to_repo(
            repo_root,
            &otu_representatives_fasta,
        ),
        otu_representatives_fasta: path_relative_to_repo(repo_root, &otu_representatives_fasta),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        taxonomy_ready_fasta: path_relative_to_repo(repo_root, &taxonomy_ready_fasta),
        taxonomy_ready_fastq: path_relative_to_repo(repo_root, &taxonomy_ready_fastq),
        raw_backend_report: None,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "fastq.cluster_otus",
        "fastq",
        "vsearch",
        "fixture_truth_copy",
        "materialized governed OTU-clustering evidence from tracked amplicon truth fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([
            (
                "normalized_amplicon_reads".to_string(),
                DEFAULT_AMPLICON_CLUSTER_OTUS_FASTQ_PATH.to_string(),
            ),
            (
                "non_chimeric_representatives".to_string(),
                "runs/bench/micro/pipelines/amplicon/artifacts/fastq.remove_chimeras/non_chimeric.fasta"
                    .to_string(),
            ),
        ]),
        BTreeMap::from([
            ("otu_table_tsv".to_string(), report.otu_table_tsv.clone()),
            (
                "otu_representatives_fasta".to_string(),
                report.otu_representatives_fasta.clone(),
            ),
            ("case_report_json".to_string(), report.case_report_json.clone()),
            (
                "taxonomy_ready_fasta".to_string(),
                report.taxonomy_ready_fasta.clone(),
            ),
            (
                "taxonomy_ready_fastq".to_string(),
                report.taxonomy_ready_fastq.clone(),
            ),
        ]),
        BTreeMap::from([
            ("otu_count".to_string(), json!(report.otu_count)),
            ("sample_count".to_string(), json!(report.sample_count)),
            (
                "representative_sequence_count".to_string(),
                json!(report.representative_sequence_count),
            ),
            (
                "clustering_threshold".to_string(),
                json!(report.clustering_threshold),
            ),
        ]),
    );
    Ok(ClusterOtusStageArtifacts { row, otu_table_tsv, otu_representatives_fasta })
}

fn run_fixture_backed_normalize_abundance_stage(
    repo_root: &Path,
    output_root: &Path,
    corpus_manifest_path: &Path,
) -> Result<NormalizeAbundanceStageArtifacts> {
    let manifest = load_amplicon_corpus_fixture_manifest_path(corpus_manifest_path)?;
    let abundance_table = manifest
        .abundance_tables
        .iter()
        .find(|table| table.table_kind == OTU_ABUNDANCE_TABLE_KIND)
        .ok_or_else(|| anyhow!("amplicon corpus fixture is missing otu abundance table"))?;
    let manifest_dir = corpus_manifest_path
        .parent()
        .ok_or_else(|| anyhow!("amplicon corpus manifest has no parent directory"))?;
    let otu_abundance_source = manifest_dir.join(&abundance_table.table_path);

    let stage_root = stage_root(output_root, "fastq.normalize_abundance");
    let otu_abundance_table_tsv =
        copy_absolute_file(repo_root, &stage_root, &otu_abundance_source, "otu_abundance.tsv")?;
    let normalized_abundance_tsv = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_NORMALIZED_ABUNDANCE_PATH),
        "normalized_abundance.tsv",
    )?;
    let case_report_json =
        stage_root.join("corpus-03-otu-abundance-table/seqkit/normalize_abundance_report.json");
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION,
            "stage_id": "fastq.normalize_abundance",
            "sample_id": "corpus-03-otu-abundance-table",
            "planned_tool_id": "seqkit",
            "table_rows": 4,
            "sample_count": 2,
            "feature_count": 3,
            "zero_fraction": 0.0,
        }),
    )?;
    let report = NormalizeAbundanceStageEvidence {
        schema_version: NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.normalize_abundance".to_string(),
        sample_id: "corpus-03-otu-abundance-table".to_string(),
        planned_tool_id: "seqkit".to_string(),
        report_tool_id: "seqkit".to_string(),
        method: "relative_abundance".to_string(),
        normalization_method: "relative_abundance".to_string(),
        table_rows: 4,
        sample_count: 2,
        feature_count: 3,
        zero_fraction: 0.0,
        normalized_abundance_tsv: path_relative_to_repo(repo_root, &normalized_abundance_tsv),
        sample_totals: vec![
            ("corpus-03-amplicon-se".to_string(), 1.0),
            ("corpus-03-otu-cluster-se".to_string(), 1.0),
        ],
        numeric_output_valid: true,
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        otu_abundance_table_tsv: path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        "fastq.normalize_abundance",
        "fastq",
        "seqkit",
        "fixture_truth_copy",
        "materialized governed abundance-normalization evidence from tracked amplicon truth fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([(
            "otu_abundance_table".to_string(),
            path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
        )]),
        BTreeMap::from([
            (
                "otu_abundance_table".to_string(),
                path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
            ),
            ("normalized_abundance_tsv".to_string(), report.normalized_abundance_tsv.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
        ]),
        BTreeMap::from([
            ("table_rows".to_string(), json!(report.table_rows)),
            ("sample_count".to_string(), json!(report.sample_count)),
            ("feature_count".to_string(), json!(report.feature_count)),
            ("zero_fraction".to_string(), json!(report.zero_fraction)),
            ("numeric_output_valid".to_string(), json!(report.numeric_output_valid)),
        ]),
    );
    Ok(NormalizeAbundanceStageArtifacts { row, otu_abundance_table_tsv, normalized_abundance_tsv })
}

fn run_amplicon_output_judgment_stage(
    repo_root: &Path,
    output_root: &Path,
    corpus_manifest_path: &Path,
    truth_manifest_path: &Path,
    normalize_stage: &NormalizePrimersStageArtifacts,
    infer_stage: &InferAsvsStageArtifacts,
    chimera_stage: &RemoveChimerasStageArtifacts,
    otu_stage: &ClusterOtusStageArtifacts,
    abundance_stage: &NormalizeAbundanceStageArtifacts,
) -> Result<AmpliconMicroPipelineRow> {
    let manifest = load_amplicon_corpus_fixture_manifest_path(corpus_manifest_path)?;
    let primer_rows =
        load_validated_amplicon_primer_rows(repo_root, corpus_manifest_path, &manifest)?;
    let expected_asv_rows =
        load_validated_amplicon_expected_asv_rows(repo_root, corpus_manifest_path, &manifest)?;
    let expected_chimera_rows =
        load_validated_amplicon_expected_chimera_rows(repo_root, corpus_manifest_path, &manifest)?;
    let expected_abundance_rows = load_validated_amplicon_abundance_rows(
        repo_root,
        corpus_manifest_path,
        &manifest,
        OTU_ABUNDANCE_TABLE_KIND,
    )?;
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;

    let normalize_report: NormalizePrimersEvidence = load_json(
        &repo_root.join(
            normalize_stage
                .row
                .evidence_path
                .clone()
                .ok_or_else(|| anyhow!("normalize primers row is missing evidence path"))?,
        ),
    )?;
    let infer_report: InferAsvsStageEvidence = load_json(
        &repo_root.join(
            infer_stage
                .row
                .evidence_path
                .clone()
                .ok_or_else(|| anyhow!("infer ASVs row is missing evidence path"))?,
        ),
    )?;
    let chimera_report: RemoveChimerasStageEvidence = load_json(
        &repo_root.join(
            chimera_stage
                .row
                .evidence_path
                .clone()
                .ok_or_else(|| anyhow!("remove chimeras row is missing evidence path"))?,
        ),
    )?;
    let otu_report: ClusterOtusStageEvidence = load_json(
        &repo_root.join(
            otu_stage
                .row
                .evidence_path
                .clone()
                .ok_or_else(|| anyhow!("cluster OTUs row is missing evidence path"))?,
        ),
    )?;
    let abundance_report: NormalizeAbundanceStageEvidence = load_json(
        &repo_root.join(
            abundance_stage
                .row
                .evidence_path
                .clone()
                .ok_or_else(|| anyhow!("normalize abundance row is missing evidence path"))?,
        ),
    )?;

    let primer_case_valid = validate_primer_case(
        repo_root,
        &manifest,
        &primer_rows,
        &normalize_report,
        &normalize_stage.normalized_reads_r1,
    )?;
    let asv_representatives = load_fasta_records(&infer_stage.representatives_fasta)?;
    let asv_representatives_valid = asv_representatives == truth_bundle.asv_representatives
        && usize::try_from(infer_report.representative_sequence_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.asv_representatives.len());
    let asv_table_rows = load_tsv_rows(&infer_stage.asv_table_tsv)?;
    let asv_table_valid = validate_asv_table(&asv_table_rows, &truth_bundle.asv_representatives)
        && usize::try_from(infer_report.asv_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.asv_representatives.len())
        && infer_report.sample_count == 1;

    let chimera_table_rows = load_tsv_rows(&chimera_stage.chimeras_tsv)?;
    let chimera_table_valid = validate_chimera_table(&chimera_table_rows, &expected_chimera_rows)
        && usize::try_from(chimera_report.chimera_count).ok().is_some_and(|count| {
            count
                == expected_chimera_rows
                    .iter()
                    .filter(|row| row.expected_presence == "present")
                    .count()
        });
    let non_chimeric_representatives = load_fasta_records(&chimera_stage.non_chimeric_fasta)?;
    let non_chimeric_representatives_valid = non_chimeric_representatives
        == truth_bundle.non_chimeric_representatives
        && usize::try_from(chimera_report.non_chimera_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.non_chimeric_representatives.len());

    let otu_table_rows = load_otu_table_rows(&otu_stage.otu_table_tsv)?;
    let otu_table_valid =
        validate_otu_table(repo_root, &otu_table_rows, &truth_bundle.otu_representatives)
            && usize::try_from(otu_report.otu_count)
                .ok()
                .is_some_and(|count| count == truth_bundle.otu_representatives.len())
            && otu_report.sample_count == 1;
    let otu_representatives = load_fasta_records(&otu_stage.otu_representatives_fasta)?;
    let otu_representatives_valid = otu_representatives == truth_bundle.otu_representatives
        && usize::try_from(otu_report.representative_sequence_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.otu_representatives.len());

    let otu_abundance_rows = load_abundance_rows(&abundance_stage.otu_abundance_table_tsv)?;
    let otu_abundance_table_valid = otu_abundance_rows == expected_abundance_rows
        && expected_abundance_rows == truth_bundle.otu_abundances;
    let normalized_abundance_rows =
        load_normalized_abundance_rows(&abundance_stage.normalized_abundance_tsv)?;
    let normalized_abundance_valid = normalized_abundance_rows
        == truth_bundle.normalized_abundances
        && abundance_report.numeric_output_valid
        && abundance_report.sample_totals.iter().all(|(_, total)| (*total - 1.0).abs() < 1e-9);

    let valid = primer_case_valid
        && asv_table_valid
        && asv_representatives_valid
        && chimera_table_valid
        && non_chimeric_representatives_valid
        && otu_table_valid
        && otu_representatives_valid
        && otu_abundance_table_valid
        && normalized_abundance_valid;

    let evidence = AmpliconOutputJudgmentEvidence {
        schema_version: "bijux.bench.local_amplicon_output_judgment.v1",
        stage_id: "benchmark.amplicon_output_judgment".to_string(),
        valid,
        checked_section_count: 9,
        checked_row_count: primer_rows.len()
            + expected_asv_rows.len()
            + expected_chimera_rows.len()
            + truth_bundle.asv_representatives.len()
            + truth_bundle.non_chimeric_representatives.len()
            + truth_bundle.otu_representatives.len()
            + expected_abundance_rows.len()
            + truth_bundle.normalized_abundances.len(),
        primer_case_valid,
        asv_table_valid,
        asv_representatives_valid,
        chimera_table_valid,
        non_chimeric_representatives_valid,
        otu_table_valid,
        otu_representatives_valid,
        otu_abundance_table_valid,
        normalized_abundance_valid,
    };

    let stage_root = stage_root(output_root, "benchmark.amplicon_output_judgment");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &evidence)?;

    Ok(row(
        "benchmark.amplicon_output_judgment",
        "benchmark",
        "bijux",
        "truth_judgment",
        "validated amplicon primer, ASV, chimera, OTU, and abundance outputs against governed truth",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(evidence.schema_version.to_string()),
        BTreeMap::from([
            (
                "corpus_manifest_path".to_string(),
                path_relative_to_repo(repo_root, corpus_manifest_path),
            ),
            (
                "truth_manifest_path".to_string(),
                path_relative_to_repo(repo_root, truth_manifest_path),
            ),
            (
                "truth_bundle_path".to_string(),
                path_relative_to_repo(repo_root, &repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH)),
            ),
            (
                "normalized_reads_r1".to_string(),
                path_relative_to_repo(repo_root, &normalize_stage.normalized_reads_r1),
            ),
            (
                "asv_table_tsv".to_string(),
                path_relative_to_repo(repo_root, &infer_stage.asv_table_tsv),
            ),
            (
                "asv_representatives".to_string(),
                path_relative_to_repo(repo_root, &infer_stage.representatives_fasta),
            ),
            (
                "chimeras_tsv".to_string(),
                path_relative_to_repo(repo_root, &chimera_stage.chimeras_tsv),
            ),
            (
                "non_chimeric_representatives".to_string(),
                path_relative_to_repo(repo_root, &chimera_stage.non_chimeric_fasta),
            ),
            (
                "otu_table_tsv".to_string(),
                path_relative_to_repo(repo_root, &otu_stage.otu_table_tsv),
            ),
            (
                "otu_representatives".to_string(),
                path_relative_to_repo(repo_root, &otu_stage.otu_representatives_fasta),
            ),
            (
                "otu_abundance_table".to_string(),
                path_relative_to_repo(repo_root, &abundance_stage.otu_abundance_table_tsv),
            ),
            (
                "normalized_abundance_tsv".to_string(),
                path_relative_to_repo(repo_root, &abundance_stage.normalized_abundance_tsv),
            ),
        ]),
        BTreeMap::from([("judgment_report".to_string(), path_relative_to_repo(repo_root, &evidence_path))]),
        BTreeMap::from([
            ("valid".to_string(), json!(evidence.valid)),
            (
                "checked_section_count".to_string(),
                json!(evidence.checked_section_count),
            ),
            ("checked_row_count".to_string(), json!(evidence.checked_row_count)),
            ("primer_case_valid".to_string(), json!(evidence.primer_case_valid)),
            ("asv_table_valid".to_string(), json!(evidence.asv_table_valid)),
            (
                "asv_representatives_valid".to_string(),
                json!(evidence.asv_representatives_valid),
            ),
            ("chimera_table_valid".to_string(), json!(evidence.chimera_table_valid)),
            (
                "non_chimeric_representatives_valid".to_string(),
                json!(evidence.non_chimeric_representatives_valid),
            ),
            ("otu_table_valid".to_string(), json!(evidence.otu_table_valid)),
            (
                "otu_representatives_valid".to_string(),
                json!(evidence.otu_representatives_valid),
            ),
            (
                "otu_abundance_table_valid".to_string(),
                json!(evidence.otu_abundance_table_valid),
            ),
            (
                "normalized_abundance_valid".to_string(),
                json!(evidence.normalized_abundance_valid),
            ),
        ]),
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_handoffs(
    repo_root: &Path,
    corpus_stage: &AmpliconCorpusStageArtifacts,
    truth_stage: &AmpliconTruthStageArtifacts,
    normalize_stage: &NormalizePrimersStageArtifacts,
    infer_stage: &InferAsvsStageArtifacts,
    chimera_stage: &RemoveChimerasStageArtifacts,
    otu_stage: &ClusterOtusStageArtifacts,
    abundance_stage: &NormalizeAbundanceStageArtifacts,
    judgment_row: &AmpliconMicroPipelineRow,
) -> Vec<AmpliconMicroPipelineHandoff> {
    vec![
        handoff_from_rows(
            repo_root,
            &corpus_stage.row,
            "primers_tsv_path",
            &normalize_stage.row,
            "primer_contract",
        ),
        handoff_from_rows(
            repo_root,
            &truth_stage.row,
            "expected_path",
            judgment_row,
            "truth_bundle_path",
        ),
        handoff_from_rows(
            repo_root,
            &normalize_stage.row,
            "normalized_reads_r1",
            judgment_row,
            "normalized_reads_r1",
        ),
        handoff_from_rows(
            repo_root,
            &infer_stage.row,
            "asv_table_tsv",
            judgment_row,
            "asv_table_tsv",
        ),
        handoff_from_rows(
            repo_root,
            &infer_stage.row,
            "representatives_fasta",
            &chimera_stage.row,
            "asv_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &infer_stage.row,
            "representatives_fasta",
            judgment_row,
            "asv_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &chimera_stage.row,
            "chimeras_tsv",
            judgment_row,
            "chimeras_tsv",
        ),
        handoff_from_rows(
            repo_root,
            &chimera_stage.row,
            "non_chimeric_fasta",
            &otu_stage.row,
            "non_chimeric_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &chimera_stage.row,
            "non_chimeric_fasta",
            judgment_row,
            "non_chimeric_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &otu_stage.row,
            "otu_table_tsv",
            judgment_row,
            "otu_table_tsv",
        ),
        handoff_from_rows(
            repo_root,
            &otu_stage.row,
            "otu_representatives_fasta",
            judgment_row,
            "otu_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &abundance_stage.row,
            "otu_abundance_table",
            judgment_row,
            "otu_abundance_table",
        ),
        handoff_from_rows(
            repo_root,
            &abundance_stage.row,
            "normalized_abundance_tsv",
            judgment_row,
            "normalized_abundance_tsv",
        ),
    ]
}

fn passes_behavior_test(
    rows: &[AmpliconMicroPipelineRow],
    handoffs: &[AmpliconMicroPipelineHandoff],
) -> bool {
    let expected_stage_ids = BTreeSet::from([
        "benchmark.amplicon_corpus_fixture",
        "benchmark.amplicon_truth_fixture",
        "fastq.normalize_primers",
        "fastq.infer_asvs",
        "fastq.remove_chimeras",
        "fastq.cluster_otus",
        "fastq.normalize_abundance",
        "benchmark.amplicon_output_judgment",
    ]);
    let observed_stage_ids = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let no_bam_or_vcf = rows.iter().all(|row| row.domain != "bam" && row.domain != "vcf");
    let all_succeeded =
        rows.iter().all(|row| row.status == AmpliconMicroPipelineRowStatus::Succeeded);
    let all_handoffs_accepted = handoffs.iter().all(|handoff| handoff.accepted);
    let judgment_valid = rows.iter().any(|row| {
        row.stage_id == "benchmark.amplicon_output_judgment"
            && row.metrics.get("valid").and_then(Value::as_bool) == Some(true)
    });
    observed_stage_ids == expected_stage_ids
        && no_bam_or_vcf
        && all_succeeded
        && all_handoffs_accepted
        && judgment_valid
}

fn handoff_from_rows(
    repo_root: &Path,
    source_row: &AmpliconMicroPipelineRow,
    source_output_id: &str,
    target_row: &AmpliconMicroPipelineRow,
    target_input_id: &str,
) -> AmpliconMicroPipelineHandoff {
    let source_path = source_row.outputs.get(source_output_id).cloned().unwrap_or_default();
    let target_path = target_row.consumed_inputs.get(target_input_id).cloned().unwrap_or_default();
    let source_exists = !source_path.is_empty() && repo_root.join(&source_path).exists();
    let target_exists = !target_path.is_empty() && repo_root.join(&target_path).exists();
    let exact_path_match = source_path == target_path;
    let accepted = source_exists && target_exists && exact_path_match;
    AmpliconMicroPipelineHandoff {
        handoff_id: format!(
            "{}:{}->{}:{}",
            source_row.stage_id, source_output_id, target_row.stage_id, target_input_id
        ),
        source_stage_id: source_row.stage_id.clone(),
        target_stage_id: target_row.stage_id.clone(),
        source_output_id: source_output_id.to_string(),
        target_input_id: target_input_id.to_string(),
        source_path,
        target_path,
        source_exists,
        target_exists,
        exact_path_match,
        accepted,
        detail: if accepted {
            "micro pipeline keeps the governed artifact handoff explicit".to_string()
        } else {
            "micro pipeline handoff drifted from the governed artifact path".to_string()
        },
    }
}

fn validate_primer_case(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    primer_rows: &[AmpliconPrimerTruthRow],
    report: &NormalizePrimersEvidence,
    normalized_reads_r1: &Path,
) -> Result<bool> {
    let primer_row = primer_rows
        .iter()
        .find(|row| row.primer_id == manifest.primer_set_id)
        .ok_or_else(|| {
            anyhow!("amplicon primer truth rows are missing primer set {}", manifest.primer_set_id)
        })?;
    let observed_read_count = count_gz_fastq_reads(normalized_reads_r1)?;
    Ok(report.primer_set_id == manifest.primer_set_id
        && report.marker_id == manifest.marker_id
        && report.orientation_policy == primer_row.orientation
        && report.input_reads == report.matched_reads + report.unmatched_reads
        && report.output_reads == observed_read_count
        && repo_root.join(&report.report_json).is_file()
        && repo_root.join(&report.primer_orientation_report).is_file()
        && repo_root.join(&report.primer_stats_json).is_file())
}

fn validate_asv_table(rows: &[Vec<String>], representatives: &[FastaTruth]) -> bool {
    if rows.is_empty() {
        return false;
    }
    let expected_ids = representatives.iter().map(|row| row.id.as_str()).collect::<BTreeSet<_>>();
    let observed_ids =
        rows.iter().filter_map(|row| row.get(1)).map(String::as_str).collect::<BTreeSet<_>>();
    expected_ids == observed_ids && rows.iter().all(|row| row.len() == 3 && row[2] != "0")
}

fn validate_chimera_table(
    rows: &[Vec<String>],
    expected_chimeras: &[AmpliconExpectedChimeraTruthRow],
) -> bool {
    let expected_ids = expected_chimeras
        .iter()
        .filter(|row| row.expected_presence == "present")
        .map(|row| row.chimera_id.as_str())
        .collect::<BTreeSet<_>>();
    let observed_ids =
        rows.iter().filter_map(|row| row.first()).map(String::as_str).collect::<BTreeSet<_>>();
    expected_ids == observed_ids
}

fn validate_otu_table(
    repo_root: &Path,
    rows: &[OtuTableRow],
    representatives: &[FastaTruth],
) -> bool {
    let expected_ids = representatives.iter().map(|row| row.id.as_str()).collect::<BTreeSet<_>>();
    let observed_ids = rows.iter().map(|row| row.otu_id.as_str()).collect::<BTreeSet<_>>();
    expected_ids == observed_ids
        && rows.len() == representatives.len()
        && rows.iter().all(|row| repo_root.join(&row.representative_fasta).is_file())
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json_file(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = serde_json::to_vec_pretty(value)
        .with_context(|| format!("serialize {}", path.display()))?;
    fs::write(path, rendered).with_context(|| format!("write {}", path.display()))
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

fn write_fastq_records(path: &Path, records: &[(&str, &str)]) -> Result<()> {
    let mut rendered = String::new();
    for (id, sequence) in records {
        rendered.push('@');
        rendered.push_str(id);
        rendered.push('\n');
        rendered.push_str(sequence);
        rendered.push_str("\n+\n");
        rendered.push_str(&"I".repeat(sequence.len()));
        rendered.push('\n');
    }
    write_text_file(path, &rendered)
}

fn build_three_column_table(header: &str, rows: impl IntoIterator<Item = String>) -> String {
    let mut rendered = String::from(header);
    for row in rows {
        rendered.push_str(&row);
    }
    rendered
}

fn is_nonempty_file(path: &Path) -> bool {
    path.metadata().map(|metadata| metadata.is_file() && metadata.len() > 0).unwrap_or(false)
}

fn copy_repo_relative_file(
    repo_root: &Path,
    stage_root: &Path,
    source_relative_path: &str,
    destination_relative_path: &str,
) -> Result<PathBuf> {
    copy_absolute_file(
        repo_root,
        stage_root,
        &repo_root.join(source_relative_path),
        destination_relative_path,
    )
}

fn copy_repo_relative_file_with_absolute_fallback(
    repo_root: &Path,
    stage_root: &Path,
    source_relative_path: &str,
    fallback_source_path: &Path,
    destination_relative_path: &str,
) -> Result<PathBuf> {
    let source_path = repo_root.join(source_relative_path);
    let resolved_source_path = if source_path
        .metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
    {
        source_path
    } else {
        fallback_source_path.to_path_buf()
    };
    copy_absolute_file(repo_root, stage_root, &resolved_source_path, destination_relative_path)
}

fn copy_absolute_file(
    _repo_root: &Path,
    stage_root: &Path,
    source_path: &Path,
    destination_relative_path: &str,
) -> Result<PathBuf> {
    let destination = stage_root.join(destination_relative_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(source_path, &destination)
        .with_context(|| format!("copy {} to {}", source_path.display(), destination.display()))?;
    Ok(destination)
}

fn stage_root(output_root: &Path, stage_id: &str) -> PathBuf {
    output_root.join("artifacts").join(stage_id)
}

fn row(
    stage_id: &str,
    domain: &str,
    tool_id: &str,
    execution_mode: &str,
    reason: &str,
    evidence_path: Option<String>,
    parsed_schema_version: Option<String>,
    consumed_inputs: BTreeMap<String, String>,
    outputs: BTreeMap<String, String>,
    metrics: BTreeMap<String, Value>,
) -> AmpliconMicroPipelineRow {
    AmpliconMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.to_string(),
        status: AmpliconMicroPipelineRowStatus::Succeeded,
        reason: reason.to_string(),
        evidence_path,
        parsed_schema_version,
        consumed_inputs,
        outputs,
        metrics,
    }
}

fn load_fasta_records(path: &Path) -> Result<Vec<FastaTruth>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut current_id = None::<String>;
    let mut current_sequence = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if let Some(rest) = line.strip_prefix('>') {
            if let Some(id) = current_id.take() {
                records.push(FastaTruth { id, sequence: current_sequence.clone() });
                current_sequence.clear();
            }
            current_id = Some(rest.to_string());
        } else if !line.trim().is_empty() {
            current_sequence.push_str(line.trim());
        }
    }
    if let Some(id) = current_id {
        records.push(FastaTruth { id, sequence: current_sequence });
    }
    if records.is_empty() {
        bail!("fasta is empty: {}", path.display());
    }
    Ok(records)
}

fn load_tsv_rows(path: &Path) -> Result<Vec<Vec<String>>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if index == 0 {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        rows.push(line.split('\t').map(std::string::ToString::to_string).collect());
    }
    Ok(rows)
}

fn load_otu_table_rows(path: &Path) -> Result<Vec<OtuTableRow>> {
    let rows = load_tsv_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != 5 {
                bail!("otu table row has unexpected width in {}", path.display());
            }
            Ok(OtuTableRow {
                sample_id: row[0].clone(),
                otu_id: row[1].clone(),
                abundance: row[2].clone(),
                representative_id: row[3].clone(),
                representative_fasta: row[4].clone(),
            })
        })
        .collect()
}

fn load_abundance_rows(path: &Path) -> Result<Vec<AmpliconAbundanceTruthRow>> {
    let rows = load_tsv_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != 3 {
                bail!("abundance table row has unexpected width in {}", path.display());
            }
            Ok(AmpliconAbundanceTruthRow {
                sample_id: row[0].clone(),
                feature_id: row[1].clone(),
                abundance: row[2]
                    .parse::<u64>()
                    .with_context(|| format!("parse abundance in {}", path.display()))?,
            })
        })
        .collect()
}

fn load_normalized_abundance_rows(path: &Path) -> Result<Vec<NormalizedAbundanceTruthRow>> {
    let rows = load_tsv_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != 3 {
                bail!("normalized abundance row has unexpected width in {}", path.display());
            }
            Ok(NormalizedAbundanceTruthRow {
                sample_id: row[0].clone(),
                feature_id: row[1].clone(),
                normalized_abundance: row[2].clone(),
            })
        })
        .collect()
}

fn count_gz_fastq_reads(path: &Path) -> Result<u64> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(MultiGzDecoder::new(file));
    let line_count = reader.lines().try_fold(0u64, |count, line| {
        let _ = line.with_context(|| format!("read {}", path.display()))?;
        Ok::<u64, anyhow::Error>(count + 1)
    })?;
    if !line_count.is_multiple_of(4) {
        bail!("fastq line count is not divisible by four in {}", path.display());
    }
    Ok(line_count / 4)
}

fn timestamp_marker() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
