use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{run_pca_stage, PcaStageParams};
use serde::Serialize;

use super::local_corpus_fixture::vcf::{
    load_sample_metadata, load_vcf_corpus_fixture_manifest_path, SampleMetadataRow,
    DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_PCA_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.pca";
const LOCAL_VCF_PCA_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_pca_smoke.v1";
const LOCAL_VCF_PCA_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-pca-smoke";
const GOVERNED_VCF_PCA_STAGE_ID: &str = "vcf.pca";
const GOVERNED_VCF_PCA_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_PCA_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_PCA_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_PCA_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
const DEFAULT_INPUT_VCF_NAME: &str = "pca_input.vcf";
const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
const DEFAULT_INPUT_POPULATION_METADATA_NAME: &str = "population_metadata.tsv";
const DEFAULT_INPUT_POPULATION_LABELS_NAME: &str = "population_labels.json";
const DEFAULT_OUTPUT_TSV_NAME: &str = "pca.tsv";
const DEFAULT_OUTPUT_JSON_NAME: &str = "pca.json";
const DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME: &str = "source_eigenvec.tsv";
const DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME: &str = "source_eigenval.tsv";
const DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME: &str = "source_pca_manifest.json";
const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const GOVERNED_PCA_COMPONENT_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfPcaSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PopulationMetadataRow {
    population_id: String,
    population_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPcaSmokeRow {
    pub(crate) sample_id: String,
    pub(crate) population_id: String,
    pub(crate) population_label: String,
    pub(crate) sex: String,
    pub(crate) pc1: f64,
    pub(crate) pc2: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPcaSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) fixture_manifest_path: String,
    pub(crate) input_vcf_path: String,
    pub(crate) sample_metadata_path: String,
    pub(crate) population_metadata_path: String,
    pub(crate) population_labels_manifest_path: String,
    pub(crate) output_root: String,
    pub(crate) pca_tsv_path: String,
    pub(crate) pca_json_path: String,
    pub(crate) source_eigenvec_path: String,
    pub(crate) source_eigenval_path: String,
    pub(crate) source_pca_manifest_path: String,
    pub(crate) source_logs_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) variant_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) excluded_samples: Vec<String>,
    pub(crate) unexpected_samples: Vec<String>,
    pub(crate) eigenvalues: Vec<f64>,
    pub(crate) rows: Vec<LocalVcfPcaSmokeRow>,
}

pub(crate) fn run_vcf_pca_smoke(args: &parse::BenchLocalRunVcfPcaSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_pca_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.pca_json_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_pca_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPcaSmokeReport> {
    let contract = resolve_governed_vcf_pca_smoke_contract(tool_id)?;
    let fixture_manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let fixture_manifest = load_vcf_corpus_fixture_manifest_path(&fixture_manifest_path)?;
    let fixture_dir = fixture_manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", fixture_manifest_path.display())
    })?;
    let input_vcf_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.multisample_vcf_path);
    let sample_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.sample_metadata_path);
    let population_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.population_metadata_path);
    let expected_samples = fixture_manifest.expected_multisample_sample_ids.clone();
    if expected_samples.is_empty() {
        bail!("governed VCF PCA smoke fixture must declare multisample cohort sample ids");
    }

    let sample_metadata = load_sample_metadata(&sample_metadata_source)?;
    let metadata_by_sample = build_metadata_by_sample(&sample_metadata, &expected_samples)?;
    let population_labels = load_population_labels(&population_metadata_source)?;

    let published_output_root = repo_root.join(DEFAULT_VCF_PCA_SMOKE_ROOT).join(&contract.tool_id);
    let staging_parent = published_output_root.parent().ok_or_else(|| {
        anyhow!("VCF PCA smoke root has no parent: {}", published_output_root.display())
    })?;
    let staging_dir =
        bijux_dna_infra::temp_dir_in(staging_parent, &format!("{}-staging-", contract.tool_id))
            .with_context(|| format!("create staging directory in {}", staging_parent.display()))?;
    let output_root = staging_dir.path().to_path_buf();
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    fs::create_dir_all(&input_root).with_context(|| format!("create {}", input_root.display()))?;
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    fs::copy(&input_vcf_source, &input_vcf_path).with_context(|| {
        format!("copy {} to {}", input_vcf_source.display(), input_vcf_path.display())
    })?;
    let sample_metadata_path = input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    fs::copy(&sample_metadata_source, &sample_metadata_path).with_context(|| {
        format!("copy {} to {}", sample_metadata_source.display(), sample_metadata_path.display())
    })?;
    let population_metadata_path = input_root.join(DEFAULT_INPUT_POPULATION_METADATA_NAME);
    fs::copy(&population_metadata_source, &population_metadata_path).with_context(|| {
        format!(
            "copy {} to {}",
            population_metadata_source.display(),
            population_metadata_path.display()
        )
    })?;
    let population_labels_manifest_path = input_root.join(DEFAULT_INPUT_POPULATION_LABELS_NAME);
    write_population_labels_manifest(
        &population_labels_manifest_path,
        &metadata_by_sample,
        &expected_samples,
    )?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_pca_stage(
        &input_vcf_path,
        &stage_root,
        &PcaStageParams {
            toolchain: contract.tool_id.clone(),
            components: GOVERNED_PCA_COMPONENT_COUNT,
            sample_metadata_manifest: Some(population_labels_manifest_path.clone()),
            preprocessing: Default::default(),
        },
    )
    .with_context(|| format!("run governed VCF PCA smoke from {}", input_vcf_path.display()))?;

    let source_eigenvec_path = output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME);
    fs::copy(&stage_outputs.eigenvec_tsv, &source_eigenvec_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.eigenvec_tsv.display(),
            source_eigenvec_path.display()
        )
    })?;
    let source_eigenval_path = output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME);
    fs::copy(&stage_outputs.eigenval_tsv, &source_eigenval_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.eigenval_tsv.display(),
            source_eigenval_path.display()
        )
    })?;
    let source_pca_manifest_path = output_root.join(DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME);
    fs::copy(&stage_outputs.pca_manifest_json, &source_pca_manifest_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.pca_manifest_json.display(),
            source_pca_manifest_path.display()
        )
    })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let source_manifest = read_json(&source_pca_manifest_path)?;
    let execution_mode = source_manifest
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("PCA manifest missing `execution_mode`"))?
        .to_string();
    let tool_ok = source_manifest
        .pointer("/tool_attempts/pca/ok")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("PCA manifest missing `tool_attempts.pca.ok`"))?;
    let variant_count = source_manifest
        .get("variants_passing")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("PCA manifest missing `variants_passing`"))?;
    validate_pca_manifest_samples(&source_manifest, &expected_samples)?;

    let rows = parse_pca_rows(&source_eigenvec_path, &metadata_by_sample, &population_labels)?;
    let eigenvalues = parse_eigenvalues(&source_eigenval_path)?;
    if eigenvalues.len() < GOVERNED_PCA_COMPONENT_COUNT {
        bail!(
            "governed VCF PCA smoke expected at least {} eigenvalues, found {}",
            GOVERNED_PCA_COMPONENT_COUNT,
            eigenvalues.len()
        );
    }

    let observed_samples = rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    let observed_sample_set = observed_samples.iter().cloned().collect::<BTreeSet<_>>();
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let excluded_samples = expected_samples
        .iter()
        .filter(|sample_id| !observed_sample_set.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_samples = observed_samples
        .iter()
        .filter(|sample_id| !expected_sample_set.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();
    if !excluded_samples.is_empty()
        || !unexpected_samples.is_empty()
        || rows.len() != expected_samples.len()
    {
        bail!(
            "governed VCF PCA smoke expected samples {:?}, observed {:?}",
            expected_samples,
            observed_samples
        );
    }

    let pca_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(&pca_tsv_path, build_pca_tsv(&rows).as_bytes())?;
    let pca_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let published_artifacts_root = published_output_root.join("artifacts");
    let published_input_root = published_artifacts_root.join("input");
    let published_pca_tsv_path = published_output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    let published_pca_json_path = published_output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let published_input_vcf_path = published_input_root.join(DEFAULT_INPUT_VCF_NAME);
    let published_sample_metadata_path =
        published_input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    let published_population_metadata_path =
        published_input_root.join(DEFAULT_INPUT_POPULATION_METADATA_NAME);
    let published_population_labels_manifest_path =
        published_input_root.join(DEFAULT_INPUT_POPULATION_LABELS_NAME);
    let published_source_eigenvec_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME);
    let published_source_eigenval_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME);
    let published_source_pca_manifest_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME);
    let published_source_logs_path = published_output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    let published_stage_result_manifest_path =
        published_output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let sample_count = u64::try_from(rows.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let report = LocalVcfPcaSmokeReport {
        schema_version: LOCAL_VCF_PCA_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_PCA_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, &fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, &published_input_vcf_path),
        sample_metadata_path: path_relative_to_repo(repo_root, &published_sample_metadata_path),
        population_metadata_path: path_relative_to_repo(
            repo_root,
            &published_population_metadata_path,
        ),
        population_labels_manifest_path: path_relative_to_repo(
            repo_root,
            &published_population_labels_manifest_path,
        ),
        output_root: path_relative_to_repo(repo_root, &published_output_root),
        pca_tsv_path: path_relative_to_repo(repo_root, &published_pca_tsv_path),
        pca_json_path: path_relative_to_repo(repo_root, &published_pca_json_path),
        source_eigenvec_path: path_relative_to_repo(repo_root, &published_source_eigenvec_path),
        source_eigenval_path: path_relative_to_repo(repo_root, &published_source_eigenval_path),
        source_pca_manifest_path: path_relative_to_repo(
            repo_root,
            &published_source_pca_manifest_path,
        ),
        source_logs_path: path_relative_to_repo(repo_root, &published_source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(
            repo_root,
            &published_stage_result_manifest_path,
        ),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        tool_ok,
        variant_count,
        sample_count,
        excluded_samples,
        unexpected_samples,
        eigenvalues,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&pca_json_path, &report)?;

    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: report.command.clone() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at,
            finished_at,
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: vec![
            BenchStageResultOutputV1 {
                artifact_id: "pca_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_TSV_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &published_pca_tsv_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "pca_json".to_string(),
                declared_path: DEFAULT_OUTPUT_JSON_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &published_pca_json_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_eigenvec_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &published_source_eigenvec_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_eigenval_tsv".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &published_source_eigenval_path),
                role: "table_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_pca_manifest_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME.to_string(),
                realized_path: path_relative_to_repo(
                    repo_root,
                    &published_source_pca_manifest_path,
                ),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_logs_txt".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &published_source_logs_path),
                role: "log_output".to_string(),
                optional: false,
                exists: true,
            },
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    if published_output_root.exists() {
        fs::remove_dir_all(&published_output_root)
            .with_context(|| format!("remove {}", published_output_root.display()))?;
    }
    fs::rename(&output_root, &published_output_root).with_context(|| {
        format!("publish {} to {}", output_root.display(), published_output_root.display())
    })?;
    let _ = staging_dir.keep();

    Ok(report)
}

fn resolve_governed_vcf_pca_smoke_contract(tool_id: &str) -> Result<GovernedVcfPcaSmokeContract> {
    let matrix_row = collect_vcf_benchmark_binding_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_PCA_STAGE_ID && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!("VCF PCA smoke is missing a retained benchmark binding for `{tool_id}`")
        })?;
    if matrix_row.corpus_id != GOVERNED_VCF_PCA_CORPUS_ID {
        bail!(
            "VCF PCA smoke requires corpus `{GOVERNED_VCF_PCA_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_PCA_ASSET_PROFILE_ID {
        bail!(
            "VCF PCA smoke requires asset profile `{GOVERNED_VCF_PCA_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["pca_report".to_string()] {
        bail!(
            "VCF PCA smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPcaSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_PCA_INPUT_FIXTURE_ID.to_string(),
    })
}

fn build_metadata_by_sample(
    sample_metadata: &[SampleMetadataRow],
    expected_samples: &[String],
) -> Result<BTreeMap<String, SampleMetadataRow>> {
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let mut metadata_by_sample = BTreeMap::<String, SampleMetadataRow>::new();
    for row in sample_metadata {
        if !expected_sample_set.contains(&row.sample_id) {
            continue;
        }
        if row.role != "cohort" {
            bail!(
                "governed VCF PCA smoke expected cohort role for `{}`, found `{}`",
                row.sample_id,
                row.role
            );
        }
        metadata_by_sample.insert(row.sample_id.clone(), row.clone());
    }
    for sample_id in expected_samples {
        if !metadata_by_sample.contains_key(sample_id) {
            bail!("governed VCF PCA smoke metadata is missing expected sample `{sample_id}`");
        }
    }
    Ok(metadata_by_sample)
}

fn load_population_labels(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = raw.lines();
    let header = rows.next().ok_or_else(|| anyhow!("population metadata is empty"))?;
    if header.trim() != "population_id\tpopulation_label\tsuper_population\trole" {
        bail!("population metadata header drifted in {}", path.display());
    }
    let mut labels = BTreeMap::<String, String>::new();
    for (index, line) in rows.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            bail!(
                "population metadata row {} in {} must have 4 columns",
                index + 2,
                path.display()
            );
        }
        let population_id = columns[0].trim();
        let population_label = columns[1].trim();
        if population_id.is_empty() || population_label.is_empty() {
            bail!(
                "population metadata row {} in {} must keep non-empty ids and labels",
                index + 2,
                path.display()
            );
        }
        labels.insert(population_id.to_string(), population_label.to_string());
    }
    Ok(labels)
}

fn write_population_labels_manifest(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    expected_samples: &[String],
) -> Result<()> {
    let payload = serde_json::json!({
        "samples": expected_samples.iter().map(|sample_id| {
            let row = metadata_by_sample
                .get(sample_id)
                .unwrap_or_else(|| panic!("missing metadata for sample `{sample_id}`"));
            serde_json::json!({
                "sample": row.sample_id,
                "population": row.population_id,
            })
        }).collect::<Vec<_>>()
    });
    bijux_dna_infra::atomic_write_json(path, &payload)?;
    Ok(())
}

fn parse_pca_rows(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    population_labels: &BTreeMap<String, String>,
) -> Result<Vec<LocalVcfPcaSmokeRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| anyhow!("PCA table is empty: {}", path.display()))?;
    if header.trim() != "sample\tPC1\tPC2" {
        bail!("PCA table header drifted in {}: `{header}`", path.display());
    }
    let mut rows = Vec::<LocalVcfPcaSmokeRow>::new();
    let mut seen = BTreeSet::<String>::new();
    for (index, line) in lines.enumerate() {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 3 {
            bail!("PCA row {} in {} must provide sample, PC1, and PC2", index + 2, path.display());
        }
        let sample_id = columns[0].to_string();
        if !seen.insert(sample_id.clone()) {
            bail!("PCA table duplicated sample `{sample_id}` in {}", path.display());
        }
        let metadata = metadata_by_sample.get(&sample_id).ok_or_else(|| {
            anyhow!("PCA table sample `{sample_id}` is missing governed metadata")
        })?;
        let population_label = population_labels.get(&metadata.population_id).ok_or_else(|| {
            anyhow!("population metadata is missing label for `{}`", metadata.population_id)
        })?;
        let pc1 = columns[1]
            .parse::<f64>()
            .with_context(|| format!("parse PC1 for `{sample_id}` from {}", path.display()))?;
        let pc2 = columns[2]
            .parse::<f64>()
            .with_context(|| format!("parse PC2 for `{sample_id}` from {}", path.display()))?;
        rows.push(LocalVcfPcaSmokeRow {
            sample_id,
            population_id: metadata.population_id.clone(),
            population_label: population_label.clone(),
            sex: metadata.sex.clone(),
            pc1,
            pc2,
        });
    }
    if rows.is_empty() {
        bail!("PCA table contained no sample rows in {}", path.display());
    }
    Ok(rows)
}

fn parse_eigenvalues(path: &Path) -> Result<Vec<f64>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("PCA eigenvalue table is empty: {}", path.display()))?;
    if header.trim() != "component\teigenvalue" {
        bail!("PCA eigenvalue header drifted in {}: `{header}`", path.display());
    }
    let mut eigenvalues = Vec::<f64>::new();
    for (index, line) in lines.enumerate() {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() != 2 {
            bail!(
                "PCA eigenvalue row {} in {} must provide exactly 2 columns",
                index + 2,
                path.display()
            );
        }
        eigenvalues.push(columns[1].parse::<f64>().with_context(|| {
            format!("parse eigenvalue row {} from {}", index + 2, path.display())
        })?);
    }
    Ok(eigenvalues)
}

fn validate_pca_manifest_samples(
    manifest: &serde_json::Value,
    expected_samples: &[String],
) -> Result<()> {
    let manifest_sample_count = manifest
        .get("sample_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("PCA manifest missing `sample_count`"))?;
    let expected_sample_count =
        u64::try_from(expected_samples.len()).map_err(|_| anyhow!("sample count overflow"))?;
    if manifest_sample_count != expected_sample_count {
        bail!(
            "PCA manifest sample_count drifted: expected {}, found {}",
            expected_sample_count,
            manifest_sample_count
        );
    }
    let manifest_sample_ids = manifest
        .get("sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("PCA manifest missing `sample_ids`"))?
        .iter()
        .map(|row| {
            row.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("PCA manifest sample_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    if manifest_sample_ids != expected_samples {
        bail!(
            "PCA manifest sample_ids drifted: expected {:?}, found {:?}",
            expected_samples,
            manifest_sample_ids
        );
    }
    let manifest_sample_labels = manifest
        .get("sample_population_labels")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("PCA manifest missing `sample_population_labels`"))?;
    if manifest_sample_labels.len() != expected_samples.len() {
        bail!(
            "PCA manifest sample_population_labels length drifted: expected {}, found {}",
            expected_samples.len(),
            manifest_sample_labels.len()
        );
    }
    Ok(())
}

fn build_pca_tsv(rows: &[LocalVcfPcaSmokeRow]) -> String {
    let mut rendered = String::from("sample_id\tpopulation_id\tpopulation_label\tsex\tpc1\tpc2\n");
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row.sample_id, row.population_id, row.population_label, row.sex, row.pc1, row.pc2
        ));
    }
    rendered
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn timestamp_marker() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{parse_eigenvalues, parse_pca_rows, run_local_vcf_pca_smoke};
    use crate::commands::benchmark::local_corpus_fixture::vcf::SampleMetadataRow;
    use std::collections::BTreeMap;

    #[test]
    fn pca_rows_reject_duplicate_samples() {
        let temp = tempfile::tempdir().expect("tempdir");
        let table = temp.path().join("pca.tsv");
        std::fs::write(&table, "sample\tPC1\tPC2\nsample_a\t0.1\t0.2\nsample_a\t0.3\t0.4\n")
            .expect("write pca table");
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "sample_a".to_string(),
            SampleMetadataRow {
                sample_id: "sample_a".to_string(),
                population_id: "cohort_alpha".to_string(),
                sex: "female".to_string(),
                role: "cohort".to_string(),
                description: "sample a".to_string(),
            },
        );
        let mut populations = BTreeMap::new();
        populations.insert("cohort_alpha".to_string(), "Cohort Alpha".to_string());
        let err = parse_pca_rows(&table, &metadata, &populations).expect_err("duplicate samples");
        assert!(err.to_string().contains("duplicated sample"));
    }

    #[test]
    fn eigenvalues_parse_governed_tsv_rows() {
        let temp = tempfile::tempdir().expect("tempdir");
        let table = temp.path().join("eigenval.tsv");
        std::fs::write(&table, "component\teigenvalue\nPC1\t0.9\nPC2\t0.4\n")
            .expect("write eigenvalues");
        let eigenvalues = parse_eigenvalues(&table).expect("parse eigenvalues");
        assert_eq!(eigenvalues, vec![0.9, 0.4]);
    }

    #[test]
    fn governed_vcf_pca_smoke_reports_complete_cohort_rows() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report = run_local_vcf_pca_smoke(repo_root, "plink2").expect("run local pca smoke");
        assert_eq!(report.stage_id, "vcf.pca");
        assert_eq!(report.tool_id, "plink2");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.variant_count, 2);
        assert_eq!(report.sample_count, 4);
        assert!(report.excluded_samples.is_empty());
        assert!(report.unexpected_samples.is_empty());
        assert_eq!(
            report.rows.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
            vec!["sample_a", "sample_b", "sample_c", "sample_d"]
        );
        assert!(report.eigenvalues.len() >= 2, "expected at least two governed PCA eigenvalues");
    }

    #[test]
    fn governed_vcf_pca_smoke_supports_retained_eigensoft_binding() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report =
            run_local_vcf_pca_smoke(repo_root, "eigensoft").expect("run eigensoft pca smoke");
        assert_eq!(report.stage_id, "vcf.pca");
        assert_eq!(report.tool_id, "eigensoft");
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
        assert_eq!(report.variant_count, 2);
        assert_eq!(report.sample_count, 4);
        assert!(report.excluded_samples.is_empty());
        assert!(report.unexpected_samples.is_empty());
        assert_eq!(
            report.rows.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
            vec!["sample_a", "sample_b", "sample_c", "sample_d"]
        );
        assert!(report.eigenvalues.len() >= 2, "expected at least two governed PCA eigenvalues");
    }
}
