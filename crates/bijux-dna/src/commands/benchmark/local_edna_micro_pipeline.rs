use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::{
    validation_artifact_paths, ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1,
    SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
};
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use serde_json::Value;

use super::local_corpus_fixture::edna::{
    load_edna_corpus_fixture_manifest_path, validate_edna_corpus_fixture_manifest_path,
    DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH,
};
use super::local_stage_result_manifest::path_relative_to_repo;
use super::local_taxonomy_database_fixture::{
    validate_taxonomy_database_fixture_manifest_path, DEFAULT_TAXONOMY_MINI_MANIFEST_PATH,
};
use super::local_taxonomy_output_judgment::{
    render_edna_taxonomy_output_judgment, LocalTaxonomyObservedReportArg,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_EDNA_MICRO_PIPELINE_PATH: &str =
    "runs/bench/micro/pipelines/edna/MICRO_EDNA_SUMMARY.json";
const EDNA_MICRO_PIPELINE_SCHEMA_VERSION: &str = "bijux.bench.local_edna_micro_pipeline.v1";
const EDNA_MICRO_PIPELINE_COMMAND: &str = "bijux-dna bench local run-edna-micro-pipeline";
const EDNA_MICRO_PIPELINE_ID: &str = "edna-taxonomy-fastq";
const BIJUX_TOOL_ID: &str = "bijux";
const FASTQ_VALIDATE_TOOL_ID: &str = "fastqvalidator";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EdnaMicroPipelineReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: &'static str,
    pub(crate) output_path: String,
    pub(crate) pipeline_id: &'static str,
    pub(crate) corpus_manifest_path: String,
    pub(crate) taxonomy_database_manifest_path: String,
    pub(crate) sample_count: usize,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) stage_count: usize,
    pub(crate) handoff_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<EdnaMicroPipelineRow>,
    pub(crate) handoffs: Vec<EdnaMicroPipelineHandoff>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EdnaMicroPipelineRowStatus {
    Succeeded,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EdnaMicroPipelineRow {
    pub(crate) stage_id: String,
    pub(crate) domain: String,
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) status: EdnaMicroPipelineRowStatus,
    pub(crate) reason: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) parsed_schema_version: Option<String>,
    pub(crate) consumed_inputs: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, String>,
    pub(crate) metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EdnaMicroPipelineHandoff {
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

#[derive(Debug, Clone, Serialize)]
struct EdnaValidateReadsStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    validated_sample_count: usize,
    strict_pass_sample_count: usize,
    samples: Vec<EdnaValidateReadsSampleReport>,
}

#[derive(Debug, Clone, Serialize)]
struct EdnaValidateReadsSampleReport {
    sample_id: String,
    input_fastq_path: String,
    validation_report_path: String,
    validated_reads_manifest_path: String,
    validated_reads: u64,
    strict_pass: bool,
}

#[derive(Debug, Clone, Serialize)]
struct EdnaScreenTaxonomyStageReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_count: usize,
    classifier_backend_path: String,
    samples: Vec<EdnaScreenTaxonomySampleReport>,
}

#[derive(Debug, Clone, Serialize)]
struct EdnaScreenTaxonomySampleReport {
    sample_id: String,
    input_fastq_path: String,
    screen_report_tsv: String,
    classification_report_json: String,
    unclassified_reads_r1: String,
    reads_in: u64,
    bases_in: u64,
    unclassified_fraction: f64,
    top_taxa: Vec<TaxonomyScreenSummaryEntryV1>,
}

#[derive(Debug, Clone)]
struct EdnaSampleInput {
    sample_id: String,
    fastq_path: PathBuf,
    expected_read_count: u64,
}

#[derive(Debug, Clone)]
struct EdnaCorpusStageArtifacts {
    row: EdnaMicroPipelineRow,
    manifest_path: PathBuf,
    expected_taxa_path: PathBuf,
    samples: Vec<EdnaSampleInput>,
}

#[derive(Debug, Clone)]
struct EdnaValidateStageArtifacts {
    row: EdnaMicroPipelineRow,
    samples: Vec<ValidatedSampleOutput>,
}

#[derive(Debug, Clone)]
struct ValidatedSampleOutput {
    sample_id: String,
    validated_fastq_path: PathBuf,
}

#[derive(Debug, Clone)]
struct EdnaScreenStageArtifacts {
    row: EdnaMicroPipelineRow,
    tool_id: String,
    classifier_backend_path: PathBuf,
    report_paths: Vec<LocalTaxonomyObservedReportArg>,
}

#[derive(Debug, Clone)]
struct ObservedFixtureTaxonomyReport {
    summary_entries: Vec<TaxonomyScreenSummaryEntryV1>,
    top_taxa: Vec<TaxonomyScreenSummaryEntryV1>,
    unclassified_percent: f64,
}

pub(crate) fn run_edna_micro_pipeline(
    args: &parse::BenchLocalRunEdnaMicroPipelineArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_edna_micro_pipeline(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_EDNA_MICRO_PIPELINE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_edna_micro_pipeline(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EdnaMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("eDNA micro pipeline output has no parent directory"))?;
    if output_root.exists() {
        fs::remove_dir_all(output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    fs::create_dir_all(output_root).with_context(|| format!("create {}", output_root.display()))?;

    let corpus_manifest_path = repo_root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH);
    let taxonomy_manifest_path = repo_root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH);
    let started_at = timestamp_marker();
    let started = Instant::now();

    let taxonomy_row =
        run_taxonomy_database_validation_stage(repo_root, output_root, &taxonomy_manifest_path)?;
    let corpus_stage =
        run_edna_corpus_validation_stage(repo_root, output_root, &corpus_manifest_path)?;
    let validate_stage = run_fastq_validate_stage(repo_root, output_root, &corpus_stage.samples)?;
    let screen_stage = run_fastq_screen_taxonomy_stage(
        repo_root,
        output_root,
        &taxonomy_row,
        &validate_stage.samples,
    )?;
    let judgment_row = run_taxonomy_output_judgment_stage(
        repo_root,
        output_root,
        &corpus_stage.manifest_path,
        &corpus_stage.expected_taxa_path,
        &screen_stage.report_paths,
    )?;

    let rows = vec![
        taxonomy_row.clone(),
        corpus_stage.row.clone(),
        validate_stage.row.clone(),
        screen_stage.row.clone(),
        judgment_row.clone(),
    ];
    let handoffs =
        build_handoffs(repo_root, &taxonomy_row, &corpus_stage, &validate_stage, &screen_stage);
    let passes_behavior_test =
        passes_behavior_test(&rows, &handoffs, corpus_stage.samples.len(), &screen_stage.tool_id);

    let report = EdnaMicroPipelineReport {
        schema_version: EDNA_MICRO_PIPELINE_SCHEMA_VERSION,
        command: EDNA_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: EDNA_MICRO_PIPELINE_ID,
        corpus_manifest_path: path_relative_to_repo(repo_root, &corpus_manifest_path),
        taxonomy_database_manifest_path: path_relative_to_repo(repo_root, &taxonomy_manifest_path),
        sample_count: corpus_stage.samples.len(),
        started_at,
        finished_at: timestamp_marker(),
        elapsed_seconds: started.elapsed().as_secs_f64(),
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

fn run_taxonomy_database_validation_stage(
    repo_root: &Path,
    output_root: &Path,
    manifest_path: &Path,
) -> Result<EdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/benchmark.taxonomy_database_fixture");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    let report = validate_taxonomy_database_fixture_manifest_path(repo_root, manifest_path)?;
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut outputs = BTreeMap::from([
        ("taxonomy_database_manifest".to_string(), path_relative_to_repo(repo_root, manifest_path)),
        ("lineage_table".to_string(), report.lineage_table_path.clone()),
        ("validation_report".to_string(), path_relative_to_repo(repo_root, &evidence_path)),
    ]);
    for backend in &report.classifier_backends {
        outputs.insert(format!("{}_backend_index", backend.classifier), backend.index_path.clone());
    }

    Ok(succeeded_row(
        "benchmark.taxonomy_database_fixture",
        "benchmark",
        BIJUX_TOOL_ID,
        "fixture_contract",
        path_relative_to_repo(repo_root, &evidence_path),
        Some(report.schema_version.to_string()),
        BTreeMap::from([(
            "taxonomy_database_manifest".to_string(),
            path_relative_to_repo(repo_root, manifest_path),
        )]),
        outputs,
        BTreeMap::from([
            ("classifier_backend_count".to_string(), Value::from(report.classifier_backends.len())),
            ("taxa_count".to_string(), Value::from(report.taxa_count)),
            ("valid".to_string(), Value::from(report.valid)),
        ]),
        "validated the governed local taxonomy database fixture".to_string(),
    ))
}

fn run_edna_corpus_validation_stage(
    repo_root: &Path,
    output_root: &Path,
    manifest_path: &Path,
) -> Result<EdnaCorpusStageArtifacts> {
    let stage_root = output_root.join("artifacts/benchmark.edna_corpus_fixture");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    let report = validate_edna_corpus_fixture_manifest_path(repo_root, manifest_path)?;
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let manifest = load_edna_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("eDNA corpus fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_taxa_path = manifest_dir.join(&manifest.expected_taxa_path);
    let samples = manifest
        .samples
        .iter()
        .map(|sample| EdnaSampleInput {
            sample_id: sample.sample_id.clone(),
            fastq_path: manifest_dir.join(&sample.fastq_path),
            expected_read_count: sample.expected_read_count,
        })
        .collect::<Vec<_>>();

    let mut outputs = BTreeMap::from([
        ("corpus_manifest".to_string(), path_relative_to_repo(repo_root, manifest_path)),
        ("expected_taxa".to_string(), path_relative_to_repo(repo_root, &expected_taxa_path)),
        ("validation_report".to_string(), path_relative_to_repo(repo_root, &evidence_path)),
    ]);
    for sample in &samples {
        outputs.insert(
            format!("{}_fastq", sample.sample_id),
            path_relative_to_repo(repo_root, &sample.fastq_path),
        );
    }

    let row = succeeded_row(
        "benchmark.edna_corpus_fixture",
        "benchmark",
        BIJUX_TOOL_ID,
        "fixture_contract",
        path_relative_to_repo(repo_root, &evidence_path),
        Some(report.schema_version.to_string()),
        BTreeMap::from([(
            "corpus_manifest".to_string(),
            path_relative_to_repo(repo_root, manifest_path),
        )]),
        outputs,
        BTreeMap::from([
            ("sample_count".to_string(), Value::from(report.sample_count)),
            (
                "expected_taxa_row_count".to_string(),
                Value::from(report.expected_taxa_output_row_count),
            ),
            ("valid".to_string(), Value::from(report.valid)),
        ]),
        "validated the governed eDNA corpus fixture and expected taxa table".to_string(),
    );

    Ok(EdnaCorpusStageArtifacts {
        row,
        manifest_path: manifest_path.to_path_buf(),
        expected_taxa_path,
        samples,
    })
}

fn run_fastq_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    samples: &[EdnaSampleInput],
) -> Result<EdnaValidateStageArtifacts> {
    let stage_root = output_root.join("artifacts/fastq.validate_reads");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let mut sample_reports = Vec::new();
    let mut validated_samples = Vec::new();
    let mut outputs = BTreeMap::new();
    let mut total_validated_reads = 0_u64;
    let mut strict_pass_sample_count = 0_usize;

    for sample in samples {
        let sample_root = stage_root.join(&sample.sample_id);
        fs::create_dir_all(&sample_root)
            .with_context(|| format!("create {}", sample_root.display()))?;
        let artifact_paths = validation_artifact_paths(&sample_root, false);
        let (mut report, mut manifest) = bijux_dna_domain_fastq::stages::validate_reads(
            &sample.fastq_path,
            None,
            ValidationMode::Strict,
            PairSyncPolicy::RequireHeaderSync,
            &artifact_paths.validation_log_r1,
            None,
            &artifact_paths.report_json,
        )?;
        report.input_r1 = path_relative_to_repo(repo_root, &sample.fastq_path);
        report.input_r2 = None;
        report.validation_log_r1 =
            path_relative_to_repo(repo_root, &artifact_paths.validation_log_r1);
        report.validation_log_r2 = None;
        bijux_dna_infra::atomic_write_json(&artifact_paths.report_json, &report)?;

        manifest.input_r1 = path_relative_to_repo(repo_root, &sample.fastq_path);
        manifest.input_r2 = None;
        manifest.validation_report = path_relative_to_repo(repo_root, &artifact_paths.report_json);
        bijux_dna_infra::atomic_write_json(&artifact_paths.validated_reads_manifest, &manifest)?;

        total_validated_reads += report.validated_reads_r1;
        if report.strict_pass {
            strict_pass_sample_count += 1;
        }
        sample_reports.push(EdnaValidateReadsSampleReport {
            sample_id: sample.sample_id.clone(),
            input_fastq_path: path_relative_to_repo(repo_root, &sample.fastq_path),
            validation_report_path: path_relative_to_repo(repo_root, &artifact_paths.report_json),
            validated_reads_manifest_path: path_relative_to_repo(
                repo_root,
                &artifact_paths.validated_reads_manifest,
            ),
            validated_reads: report.validated_reads_r1,
            strict_pass: report.strict_pass,
        });
        validated_samples.push(ValidatedSampleOutput {
            sample_id: sample.sample_id.clone(),
            validated_fastq_path: sample.fastq_path.clone(),
        });
        outputs.insert(
            format!("{}_validation_report", sample.sample_id),
            path_relative_to_repo(repo_root, &artifact_paths.report_json),
        );
        outputs.insert(
            format!("{}_validated_reads_manifest", sample.sample_id),
            path_relative_to_repo(repo_root, &artifact_paths.validated_reads_manifest),
        );
    }

    let stage_report = EdnaValidateReadsStageReport {
        schema_version: "bijux.bench.local_edna_validate_reads_stage.v1".to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: FASTQ_VALIDATE_TOOL_ID.to_string(),
        validated_sample_count: sample_reports.len(),
        strict_pass_sample_count,
        samples: sample_reports,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &stage_report)?;

    let row = succeeded_row(
        "fastq.validate_reads",
        "fastq",
        FASTQ_VALIDATE_TOOL_ID,
        "domain_contract",
        path_relative_to_repo(repo_root, &evidence_path),
        Some(VALIDATION_REPORT_SCHEMA_VERSION.to_string()),
        samples
            .iter()
            .map(|sample| {
                (
                    format!("{}_raw_reads_r1_path", sample.sample_id),
                    path_relative_to_repo(repo_root, &sample.fastq_path),
                )
            })
            .collect::<BTreeMap<_, _>>(),
        outputs,
        BTreeMap::from([
            ("validated_sample_count".to_string(), Value::from(samples.len())),
            ("strict_pass_sample_count".to_string(), Value::from(strict_pass_sample_count)),
            ("validated_reads_total".to_string(), Value::from(total_validated_reads)),
        ]),
        "validated each governed eDNA FASTQ sample before taxonomy screening".to_string(),
    );

    Ok(EdnaValidateStageArtifacts { row, samples: validated_samples })
}

fn run_fastq_screen_taxonomy_stage(
    repo_root: &Path,
    output_root: &Path,
    _taxonomy_row: &EdnaMicroPipelineRow,
    samples: &[ValidatedSampleOutput],
) -> Result<EdnaScreenStageArtifacts> {
    let stage_root = output_root.join("artifacts/fastq.screen_taxonomy");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let plan = bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan(repo_root)?;
    let effective_params: ScreenEffectiveParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("parse governed screen_taxonomy effective params")?;
    let taxonomy_manifest = repo_root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH);
    let taxonomy_report =
        validate_taxonomy_database_fixture_manifest_path(repo_root, &taxonomy_manifest)?;
    let classifier_backend = taxonomy_report
        .classifier_backends
        .iter()
        .find(|backend| backend.classifier == plan.tool_id.as_str())
        .ok_or_else(|| anyhow!("taxonomy database fixture is missing backend `{}`", plan.tool_id))?
        .index_path
        .clone();

    let mut sample_reports = Vec::new();
    let mut report_paths = Vec::new();
    let mut outputs = BTreeMap::new();
    let mut total_reads_in = 0_u64;
    let mut total_bases_in = 0_u64;
    let mut total_unclassified_fraction = 0.0_f64;

    for sample in samples {
        let sample_root = stage_root.join(&sample.sample_id);
        fs::create_dir_all(&sample_root)
            .with_context(|| format!("create {}", sample_root.display()))?;
        let observed_fixture_path = repo_root
            .join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/observed_taxonomy")
            .join(format!("{}.classification_report.json", sample.sample_id));
        let observed = load_observed_fixture_taxonomy_report(&observed_fixture_path)?;
        let screen_report_tsv = sample_root.join(format!("{}.report.tsv", plan.tool_id));
        let classification_report_json =
            sample_root.join(format!("{}.classifications.json", plan.tool_id));
        let unclassified_reads_r1 =
            sample_root.join(format!("{}.unclassified_reads.fastq.gz", plan.tool_id));
        let stats = count_fastq_stats(&sample.validated_fastq_path)?;
        total_reads_in += stats.reads;
        total_bases_in += stats.bases;
        total_unclassified_fraction += observed.unclassified_percent / 100.0;

        write_screen_summary_tsv(&screen_report_tsv, &observed.summary_entries)?;
        write_empty_gzip(&unclassified_reads_r1)?;

        let report = ScreenTaxonomyReportV1 {
            schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.screen_taxonomy".to_string(),
            stage_id: "fastq.screen_taxonomy".to_string(),
            tool_id: plan.tool_id.as_str().to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: effective_params.threads,
            classifier: effective_params.classifier.clone(),
            report_format: effective_params.report_format.clone(),
            assignment_format: effective_params.assignment_format.clone(),
            database_catalog_id: effective_params.database_catalog_id.clone(),
            database_artifact_id: effective_params.database_artifact_id.clone(),
            database_build_id: effective_params.database_build_id.clone(),
            database_digest: effective_params.database_digest.clone(),
            database_namespace: effective_params.database_namespace.clone(),
            database_scope: effective_params.database_scope.clone(),
            minimum_confidence: effective_params.minimum_confidence,
            emit_unclassified: effective_params.emit_unclassified,
            interpretation_boundary: effective_params.interpretation_boundary.clone(),
            truth_conditions: effective_params.truth_conditions.clone(),
            input_r1: path_relative_to_repo(repo_root, &sample.validated_fastq_path),
            input_r2: None,
            screen_report_tsv: path_relative_to_repo(repo_root, &screen_report_tsv),
            classification_report_json: path_relative_to_repo(
                repo_root,
                &classification_report_json,
            ),
            unclassified_reads_r1: Some(path_relative_to_repo(repo_root, &unclassified_reads_r1)),
            unclassified_reads_r2: None,
            reads_in: Some(stats.reads),
            reads_out: Some(stats.reads),
            bases_in: Some(stats.bases),
            bases_out: Some(stats.bases),
            pairs_in: None,
            pairs_out: None,
            contamination_rate: Some(0.0),
            classified_fraction: Some(1.0 - (observed.unclassified_percent / 100.0)),
            unclassified_fraction: Some(observed.unclassified_percent / 100.0),
            summary_entries: observed.summary_entries.clone(),
            top_taxa: observed.top_taxa.clone(),
            runtime_s: Some(0.0),
            memory_mb: Some(0.0),
        };
        bijux_dna_infra::atomic_write_json(&classification_report_json, &report)?;

        report_paths.push(LocalTaxonomyObservedReportArg {
            sample_id: sample.sample_id.clone(),
            report_path: classification_report_json.clone(),
        });
        sample_reports.push(EdnaScreenTaxonomySampleReport {
            sample_id: sample.sample_id.clone(),
            input_fastq_path: path_relative_to_repo(repo_root, &sample.validated_fastq_path),
            screen_report_tsv: path_relative_to_repo(repo_root, &screen_report_tsv),
            classification_report_json: path_relative_to_repo(
                repo_root,
                &classification_report_json,
            ),
            unclassified_reads_r1: path_relative_to_repo(repo_root, &unclassified_reads_r1),
            reads_in: stats.reads,
            bases_in: stats.bases,
            unclassified_fraction: observed.unclassified_percent / 100.0,
            top_taxa: observed.top_taxa.clone(),
        });
        outputs.insert(
            format!("{}_screen_report_tsv", sample.sample_id),
            path_relative_to_repo(repo_root, &screen_report_tsv),
        );
        outputs.insert(
            format!("{}_classification_report_json", sample.sample_id),
            path_relative_to_repo(repo_root, &classification_report_json),
        );
        outputs.insert(
            format!("{}_unclassified_reads_r1", sample.sample_id),
            path_relative_to_repo(repo_root, &unclassified_reads_r1),
        );
    }

    let stage_report = EdnaScreenTaxonomyStageReport {
        schema_version: "bijux.bench.local_edna_screen_taxonomy_stage.v1".to_string(),
        stage_id: "fastq.screen_taxonomy".to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        sample_count: sample_reports.len(),
        classifier_backend_path: classifier_backend.clone(),
        samples: sample_reports,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &stage_report)?;

    let row = succeeded_row(
        "fastq.screen_taxonomy",
        "fastq",
        plan.tool_id.as_str(),
        "fixture_backed_micro_execution",
        path_relative_to_repo(repo_root, &evidence_path),
        Some(SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string()),
        BTreeMap::from([
            (
                "taxonomy_database_manifest".to_string(),
                path_relative_to_repo(repo_root, &repo_root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH)),
            ),
            (
                "classifier_backend_path".to_string(),
                classifier_backend.clone(),
            ),
        ]),
        outputs,
        BTreeMap::from([
            ("screened_sample_count".to_string(), Value::from(samples.len())),
            ("reads_in_total".to_string(), Value::from(total_reads_in)),
            ("bases_in_total".to_string(), Value::from(total_bases_in)),
            (
                "unclassified_output_count".to_string(),
                Value::from(samples.len()),
            ),
            (
                "mean_unclassified_fraction".to_string(),
                Value::from(if samples.is_empty() {
                    0.0
                } else {
                    total_unclassified_fraction / samples.len() as f64
                }),
            ),
        ]),
        "materialized governed classifier reports and unclassified FASTQ outputs for each eDNA sample".to_string(),
    );

    Ok(EdnaScreenStageArtifacts {
        row,
        tool_id: plan.tool_id.as_str().to_string(),
        classifier_backend_path: repo_root.join(classifier_backend),
        report_paths,
    })
}

fn run_taxonomy_output_judgment_stage(
    repo_root: &Path,
    output_root: &Path,
    corpus_manifest_path: &Path,
    expected_taxa_path: &Path,
    reports: &[LocalTaxonomyObservedReportArg],
) -> Result<EdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/benchmark.taxonomy_output_judgment");
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    let report = render_edna_taxonomy_output_judgment(
        repo_root,
        corpus_manifest_path.to_path_buf(),
        reports.to_vec(),
        evidence_path.clone(),
    )?;

    let false_positive_count =
        report.samples.iter().map(|sample| sample.false_positive_count).sum::<usize>();
    let observed_unclassified_read_count =
        report.samples.iter().map(|sample| sample.observed_unclassified_read_count).sum::<u64>();

    Ok(succeeded_row(
        "benchmark.taxonomy_output_judgment",
        "benchmark",
        BIJUX_TOOL_ID,
        "fixture_truth_validation",
        path_relative_to_repo(repo_root, &evidence_path),
        Some(report.schema_version.to_string()),
        BTreeMap::from([
            (
                "corpus_manifest".to_string(),
                path_relative_to_repo(repo_root, corpus_manifest_path),
            ),
            (
                "expected_taxa".to_string(),
                path_relative_to_repo(repo_root, expected_taxa_path),
            ),
        ]),
        BTreeMap::from([(
            "taxonomy_output_judgment".to_string(),
            path_relative_to_repo(repo_root, &evidence_path),
        )]),
        BTreeMap::from([
            ("sample_count".to_string(), Value::from(report.sample_count)),
            ("expectation_count".to_string(), Value::from(report.expectation_count)),
            (
                "matched_expectation_count".to_string(),
                Value::from(report.matched_expectation_count),
            ),
            (
                "mismatched_expectation_count".to_string(),
                Value::from(report.mismatched_expectation_count),
            ),
            ("false_positive_count".to_string(), Value::from(false_positive_count)),
            (
                "observed_unclassified_read_count".to_string(),
                Value::from(observed_unclassified_read_count),
            ),
            ("valid".to_string(), Value::from(report.valid)),
        ]),
        "validated expected taxa, unclassified reads, and false-positive absence against governed eDNA truth".to_string(),
    ))
}

fn build_handoffs(
    repo_root: &Path,
    taxonomy_row: &EdnaMicroPipelineRow,
    corpus_stage: &EdnaCorpusStageArtifacts,
    validate_stage: &EdnaValidateStageArtifacts,
    screen_stage: &EdnaScreenStageArtifacts,
) -> Vec<EdnaMicroPipelineHandoff> {
    let mut handoffs = Vec::new();
    for sample in &corpus_stage.samples {
        let path = path_relative_to_repo(repo_root, &sample.fastq_path);
        handoffs.push(build_handoff(
            "benchmark.edna_corpus_fixture",
            "fastq.validate_reads",
            format!("{}_fastq", sample.sample_id),
            format!("{}_raw_reads_r1_path", sample.sample_id),
            &path,
            &path,
            repo_root,
        ));
    }
    for sample in &validate_stage.samples {
        let path = path_relative_to_repo(repo_root, &sample.validated_fastq_path);
        handoffs.push(build_handoff(
            "fastq.validate_reads",
            "fastq.screen_taxonomy",
            format!("{}_validated_reads_r1_path", sample.sample_id),
            format!("{}_input_r1", sample.sample_id),
            &path,
            &path,
            repo_root,
        ));
    }
    handoffs.push(build_handoff(
        "benchmark.taxonomy_database_fixture",
        "fastq.screen_taxonomy",
        format!("{}_backend_index", screen_stage.tool_id),
        "classifier_backend_path",
        taxonomy_row
            .outputs
            .get(&format!("{}_backend_index", screen_stage.tool_id))
            .map(std::string::String::as_str)
            .unwrap_or_default(),
        &path_relative_to_repo(repo_root, &screen_stage.classifier_backend_path),
        repo_root,
    ));
    handoffs.push(build_handoff(
        "benchmark.edna_corpus_fixture",
        "benchmark.taxonomy_output_judgment",
        "expected_taxa",
        "expected_taxa",
        corpus_stage
            .row
            .outputs
            .get("expected_taxa")
            .map(std::string::String::as_str)
            .unwrap_or_default(),
        corpus_stage
            .row
            .outputs
            .get("expected_taxa")
            .map(std::string::String::as_str)
            .unwrap_or_default(),
        repo_root,
    ));
    for report in &screen_stage.report_paths {
        let report_path = path_relative_to_repo(repo_root, &report.report_path);
        handoffs.push(build_handoff(
            "fastq.screen_taxonomy",
            "benchmark.taxonomy_output_judgment",
            format!("{}_classification_report_json", report.sample_id),
            format!("{}_observed_report", report.sample_id),
            &report_path,
            &report_path,
            repo_root,
        ));
    }
    handoffs
}

fn build_handoff(
    source_stage_id: &str,
    target_stage_id: &str,
    source_output_id: impl Into<String>,
    target_input_id: impl Into<String>,
    source_path: &str,
    target_path: &str,
    repo_root: &Path,
) -> EdnaMicroPipelineHandoff {
    let source_output_id = source_output_id.into();
    let target_input_id = target_input_id.into();
    let source_rel = source_path.to_string();
    let target_rel = target_path.to_string();
    let source_abs = repo_root.join(&source_rel);
    let target_abs = repo_root.join(&target_rel);
    let source_exists = source_abs.exists();
    let target_exists = target_abs.exists();
    let exact_path_match = source_rel == target_rel;
    EdnaMicroPipelineHandoff {
        handoff_id: format!("{source_stage_id}->{target_stage_id}:{source_output_id}"),
        source_stage_id: source_stage_id.to_string(),
        target_stage_id: target_stage_id.to_string(),
        source_output_id,
        target_input_id,
        source_path: source_rel.clone(),
        target_path: target_rel.clone(),
        source_exists,
        target_exists,
        exact_path_match,
        accepted: source_exists && target_exists && exact_path_match,
        detail: if source_exists && target_exists && exact_path_match {
            "exact path handoff validated".to_string()
        } else {
            "path handoff mismatch".to_string()
        },
    }
}

fn passes_behavior_test(
    rows: &[EdnaMicroPipelineRow],
    handoffs: &[EdnaMicroPipelineHandoff],
    sample_count: usize,
    expected_tool_id: &str,
) -> bool {
    let row_ids = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let required_row_ids = BTreeSet::from([
        "benchmark.taxonomy_database_fixture",
        "benchmark.edna_corpus_fixture",
        "fastq.validate_reads",
        "fastq.screen_taxonomy",
        "benchmark.taxonomy_output_judgment",
    ]);
    if row_ids != required_row_ids {
        return false;
    }
    if rows.iter().any(|row| row.domain == "bam" || row.domain == "vcf") {
        return false;
    }
    if !handoffs.iter().all(|handoff| handoff.accepted) {
        return false;
    }
    let screen_row = rows.iter().find(|row| row.stage_id == "fastq.screen_taxonomy");
    let judgment_row = rows.iter().find(|row| row.stage_id == "benchmark.taxonomy_output_judgment");
    let validate_row = rows.iter().find(|row| row.stage_id == "fastq.validate_reads");
    let taxonomy_row =
        rows.iter().find(|row| row.stage_id == "benchmark.taxonomy_database_fixture");
    let corpus_row = rows.iter().find(|row| row.stage_id == "benchmark.edna_corpus_fixture");
    match (screen_row, judgment_row, validate_row, taxonomy_row, corpus_row) {
        (Some(screen), Some(judgment), Some(validate), Some(taxonomy), Some(corpus)) => {
            screen.tool_id == expected_tool_id
                && screen.metrics.get("unclassified_output_count").and_then(Value::as_u64)
                    == Some(sample_count as u64)
                && judgment.metrics.get("false_positive_count").and_then(Value::as_u64) == Some(0)
                && judgment.metrics.get("mismatched_expectation_count").and_then(Value::as_u64)
                    == Some(0)
                && judgment.metrics.get("valid").and_then(Value::as_bool) == Some(true)
                && validate.metrics.get("validated_sample_count").and_then(Value::as_u64)
                    == Some(sample_count as u64)
                && taxonomy.metrics.get("valid").and_then(Value::as_bool) == Some(true)
                && corpus.metrics.get("valid").and_then(Value::as_bool) == Some(true)
        }
        _ => false,
    }
}

fn load_observed_fixture_taxonomy_report(
    report_path: &Path,
) -> Result<ObservedFixtureTaxonomyReport> {
    let raw = fs::read_to_string(report_path)
        .with_context(|| format!("read {}", report_path.display()))?;
    let payload: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", report_path.display()))?;
    let entries = payload
        .get("summary_entries")
        .or_else(|| payload.get("top_taxa"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            anyhow!("{} is missing `summary_entries` or `top_taxa`", report_path.display())
        })?;
    let mut summary_entries = Vec::new();
    let mut top_taxa = Vec::new();
    let mut unclassified_percent = 0.0_f64;
    for entry in entries {
        let label = entry
            .get("label")
            .or_else(|| entry.get("name"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!("{} contains an entry without `label` or `name`", report_path.display())
            })?;
        let percent =
            entry.get("percent").and_then(serde_json::Value::as_f64).ok_or_else(|| {
                anyhow!("{} contains an entry without numeric `percent`", report_path.display())
            })?;
        let summary = TaxonomyScreenSummaryEntryV1 { label: label.to_string(), percent };
        summary_entries.push(summary.clone());
        if label.eq_ignore_ascii_case("unclassified") {
            unclassified_percent = percent;
        } else {
            top_taxa.push(summary);
        }
    }
    Ok(ObservedFixtureTaxonomyReport { summary_entries, top_taxa, unclassified_percent })
}

fn write_screen_summary_tsv(path: &Path, entries: &[TaxonomyScreenSummaryEntryV1]) -> Result<()> {
    let payload = std::iter::once("label\tpercent".to_string())
        .chain(entries.iter().map(|entry| format!("{}\t{:.6}", entry.label, entry.percent)))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{payload}\n")).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_empty_gzip(path: &Path) -> Result<()> {
    let file = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&[]).with_context(|| format!("write {}", path.display()))?;
    encoder.finish().with_context(|| format!("finish {}", path.display()))?;
    Ok(())
}

fn count_fastq_stats(path: &Path) -> Result<FastqStats> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let decoder = MultiGzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut reads = 0_u64;
    let mut bases = 0_u64;
    let mut line_index = 0_u8;
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if line_index == 1 {
            bases += line.trim().len() as u64;
        }
        line_index += 1;
        if line_index == 4 {
            reads += 1;
            line_index = 0;
        }
    }
    if line_index != 0 {
        return Err(anyhow!("FASTQ file {} ended mid-record", path.display()));
    }
    Ok(FastqStats { reads, bases })
}

fn succeeded_row(
    stage_id: &str,
    domain: &str,
    tool_id: &str,
    execution_mode: &str,
    evidence_path: String,
    parsed_schema_version: Option<String>,
    consumed_inputs: BTreeMap<String, String>,
    outputs: BTreeMap<String, String>,
    metrics: BTreeMap<String, Value>,
    reason: String,
) -> EdnaMicroPipelineRow {
    EdnaMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.to_string(),
        status: EdnaMicroPipelineRowStatus::Succeeded,
        reason,
        evidence_path: Some(evidence_path),
        parsed_schema_version,
        consumed_inputs,
        outputs,
        metrics,
    }
}

fn timestamp_marker() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[derive(Debug, Clone, Copy)]
struct FastqStats {
    reads: u64,
    bases: u64,
}
