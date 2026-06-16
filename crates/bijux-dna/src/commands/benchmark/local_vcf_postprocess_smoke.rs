use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{run_postprocess_stage, PostprocessStageParams};
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;
use serde_json::Value;

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_call_bam_smoke_support::parse_output_sample_count;
use super::local_vcf_panel_workflow_smoke_support::governed_vcf_panel_species_context;
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_POSTPROCESS_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.postprocess";
const LOCAL_VCF_POSTPROCESS_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_postprocess_smoke.v1";
const LOCAL_VCF_POSTPROCESS_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_postprocess_smoke.metrics.v1";
const LOCAL_VCF_POSTPROCESS_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-postprocess-smoke";
const GOVERNED_VCF_POSTPROCESS_STAGE_ID: &str = "vcf.postprocess";
const GOVERNED_VCF_POSTPROCESS_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_POSTPROCESS_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID: &str = "vcf_single_sample";
const GOVERNED_VCF_POSTPROCESS_INPUT_FIXTURE_ID: &str = "multiallelic_normalization_single_sample";
const DEFAULT_INPUT_VCF_NAME: &str = "postprocess_input.vcf";
const DEFAULT_OUTPUT_VCF_NAME: &str = "postprocess.vcf.gz";
const DEFAULT_OUTPUT_VALIDATE_NAME: &str = "validate_outputs.json";
const DEFAULT_OUTPUT_MANIFEST_NAME: &str = "final_manifest.json";
const DEFAULT_OUTPUT_NORMALIZATION_NAME: &str = "normalization_contract.json";
const DEFAULT_OUTPUT_CHECKSUMS_NAME: &str = "artifact_checksums.json";
const DEFAULT_OUTPUT_LOGS_NAME: &str = "logs.txt";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfPostprocessSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PostprocessOutputSummary {
    record_count: u64,
    sample_count: u64,
    sample_ids: Vec<String>,
    mq_removed: bool,
    multiallelic_records_remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfPostprocessSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) readable_vcf: bool,
    pub(crate) tabix_present: bool,
    pub(crate) contigs_consistent_with_species_context: bool,
    pub(crate) left_align_applied: bool,
    pub(crate) multiallelic_records_split: u64,
    pub(crate) indels_normalized: u64,
    pub(crate) variant_ids_normalized: u64,
    pub(crate) invalid_records_removed: u64,
    pub(crate) filter_standardized_to_pass: u64,
    pub(crate) record_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) mq_removed: bool,
    pub(crate) multiallelic_records_remaining: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPostprocessSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) input_vcf_path: String,
    pub(crate) output_root: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) validate_outputs_path: String,
    pub(crate) final_manifest_path: String,
    pub(crate) normalization_contract_path: String,
    pub(crate) artifact_checksums_path: String,
    pub(crate) logs_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) readable_vcf: bool,
    pub(crate) tabix_present: bool,
    pub(crate) contigs_consistent_with_species_context: bool,
    pub(crate) left_align_applied: bool,
    pub(crate) multiallelic_records_split: u64,
    pub(crate) indels_normalized: u64,
    pub(crate) variant_ids_normalized: u64,
    pub(crate) invalid_records_removed: u64,
    pub(crate) filter_standardized_to_pass: u64,
    pub(crate) record_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) mq_removed: bool,
    pub(crate) multiallelic_records_remaining: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
}

pub(crate) fn run_vcf_postprocess_smoke(
    args: &parse::BenchLocalRunVcfPostprocessSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_postprocess_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_postprocess_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPostprocessSmokeReport> {
    let contract = resolve_governed_vcf_postprocess_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_POSTPROCESS_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    fs::create_dir_all(&input_root).with_context(|| format!("create {}", input_root.display()))?;
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf = input_root.join(DEFAULT_INPUT_VCF_NAME);
    write_governed_postprocess_input_vcf(&input_vcf)?;

    let species_context = governed_vcf_panel_species_context();
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_postprocess_stage(
        &input_vcf,
        &stage_root,
        &species_context,
        &PostprocessStageParams {
            species_id: species_context.species_id.clone(),
            build_id: species_context.build_id.clone(),
            per_chr_inputs: vec![],
            retain_info_fields: vec![],
            remove_info_fields: vec!["MQ".to_string()],
            compression_level: 6,
            compression_threads: 2,
            emit_bcf: false,
            normalize_indels: true,
            split_multiallelic: true,
            run_level_checksums_path: None,
        },
    )
    .with_context(|| format!("run governed VCF postprocess smoke from {}", input_vcf.display()))?;

    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.merged_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.merged_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.merged_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.merged_tbi.display(), output_tbi.display())
    })?;
    let validate_outputs_path = output_root.join(DEFAULT_OUTPUT_VALIDATE_NAME);
    fs::copy(&stage_outputs.validate_outputs_json, &validate_outputs_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.validate_outputs_json.display(),
            validate_outputs_path.display()
        )
    })?;
    let final_manifest_path = output_root.join(DEFAULT_OUTPUT_MANIFEST_NAME);
    fs::copy(&stage_outputs.final_manifest_json, &final_manifest_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.final_manifest_json.display(),
            final_manifest_path.display()
        )
    })?;
    let normalization_contract_path = output_root.join(DEFAULT_OUTPUT_NORMALIZATION_NAME);
    fs::copy(&stage_outputs.normalization_contract_json, &normalization_contract_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.normalization_contract_json.display(),
                normalization_contract_path.display()
            )
        })?;
    let artifact_checksums_path = output_root.join(DEFAULT_OUTPUT_CHECKSUMS_NAME);
    fs::copy(&stage_outputs.artifact_checksums_json, &artifact_checksums_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.artifact_checksums_json.display(),
                artifact_checksums_path.display()
            )
        },
    )?;
    let logs_path = output_root.join(DEFAULT_OUTPUT_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), logs_path.display())
    })?;

    let validation = vcf_validate_input(
        &output_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", output_vcf.display()))?;
    let validate_outputs = read_json_value(&validate_outputs_path)?;
    let final_manifest = read_json_value(&final_manifest_path)?;
    let output_summary = summarize_postprocess_output(&output_vcf)?;

    let metrics = LocalVcfPostprocessSmokeMetrics {
        schema_version: LOCAL_VCF_POSTPROCESS_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        readable_vcf: json_bool(&validate_outputs, "/readable_vcf")?,
        tabix_present: json_bool(&validate_outputs, "/tabix_present")?,
        contigs_consistent_with_species_context: json_bool(
            &validate_outputs,
            "/contigs_consistent_with_species_context",
        )?,
        left_align_applied: json_bool(&final_manifest, "/normalization/left_align_applied")?,
        multiallelic_records_split: json_u64(
            &final_manifest,
            "/normalization/multiallelic_records_split",
        )?,
        indels_normalized: json_u64(&final_manifest, "/normalization/indels_normalized")?,
        variant_ids_normalized: json_u64(&final_manifest, "/normalization/variant_ids_normalized")?,
        invalid_records_removed: json_u64(
            &final_manifest,
            "/normalization/invalid_records_removed",
        )?,
        filter_standardized_to_pass: json_u64(
            &final_manifest,
            "/normalization/filter_standardized_to_pass",
        )?,
        record_count: output_summary.record_count,
        sample_count: output_summary.sample_count,
        sample_ids: output_summary.sample_ids.clone(),
        mq_removed: output_summary.mq_removed,
        multiallelic_records_remaining: output_summary.multiallelic_records_remaining,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    if !metrics.readable_vcf
        || !metrics.tabix_present
        || !metrics.contigs_consistent_with_species_context
    {
        bail!("governed VCF postprocess smoke produced invalid validation state");
    }
    if metrics.multiallelic_records_split == 0 {
        bail!("governed VCF postprocess smoke must prove multiallelic splitting");
    }
    if metrics.indels_normalized == 0 {
        bail!("governed VCF postprocess smoke must prove indel normalization");
    }
    if metrics.variant_ids_normalized == 0 {
        bail!("governed VCF postprocess smoke must prove variant-id normalization");
    }
    if metrics.filter_standardized_to_pass == 0 {
        bail!("governed VCF postprocess smoke must prove filter standardization");
    }
    if !metrics.mq_removed {
        bail!("governed VCF postprocess smoke must remove MQ from INFO");
    }
    if metrics.multiallelic_records_remaining != 0 {
        bail!("governed VCF postprocess smoke must emit only biallelic output rows");
    }

    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        &contract,
        &format!("{LOCAL_VCF_POSTPROCESS_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "postprocess_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "postprocess_vcf_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi.as_path(),
                "vcf_index",
            ),
            (
                "metrics_json",
                DEFAULT_OUTPUT_METRICS_NAME.to_string(),
                metrics_path.as_path(),
                "report_output",
            ),
        ],
        &started_at,
        &finished_at,
        elapsed_seconds,
    );
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(LocalVcfPostprocessSmokeReport {
        schema_version: LOCAL_VCF_POSTPROCESS_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_POSTPROCESS_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: contract.input_fixture_id,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf),
        output_root: path_relative_to_repo(repo_root, &output_root),
        output_vcf_path: path_relative_to_repo(repo_root, &output_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &output_tbi),
        validate_outputs_path: path_relative_to_repo(repo_root, &validate_outputs_path),
        final_manifest_path: path_relative_to_repo(repo_root, &final_manifest_path),
        normalization_contract_path: path_relative_to_repo(repo_root, &normalization_contract_path),
        artifact_checksums_path: path_relative_to_repo(repo_root, &artifact_checksums_path),
        logs_path: path_relative_to_repo(repo_root, &logs_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: metrics.exit_code,
        readable_vcf: metrics.readable_vcf,
        tabix_present: metrics.tabix_present,
        contigs_consistent_with_species_context: metrics.contigs_consistent_with_species_context,
        left_align_applied: metrics.left_align_applied,
        multiallelic_records_split: metrics.multiallelic_records_split,
        indels_normalized: metrics.indels_normalized,
        variant_ids_normalized: metrics.variant_ids_normalized,
        invalid_records_removed: metrics.invalid_records_removed,
        filter_standardized_to_pass: metrics.filter_standardized_to_pass,
        record_count: metrics.record_count,
        sample_count: metrics.sample_count,
        sample_ids: metrics.sample_ids,
        mq_removed: metrics.mq_removed,
        multiallelic_records_remaining: metrics.multiallelic_records_remaining,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
    })
}

fn resolve_governed_vcf_postprocess_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfPostprocessSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_POSTPROCESS_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_POSTPROCESS_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_POSTPROCESS_TOOL_ID {
        bail!(
            "VCF postprocess smoke requires retained tool `{GOVERNED_VCF_POSTPROCESS_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF postprocess smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_POSTPROCESS_CORPUS_ID {
        bail!(
            "VCF postprocess smoke requires corpus `{GOVERNED_VCF_POSTPROCESS_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID {
        bail!(
            "VCF postprocess smoke requires asset profile `{GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["postprocess_vcf".to_string()] {
        bail!(
            "VCF postprocess smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPostprocessSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_POSTPROCESS_INPUT_FIXTURE_ID.to_string(),
    })
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfPostprocessSmokeContract,
    command: &str,
    output_entries: &[(&str, String, &Path, &str)],
    started_at: &str,
    finished_at: &str,
    elapsed_seconds: f64,
) -> BenchStageResultManifestV1 {
    BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: command.to_string() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: started_at.to_string(),
            finished_at: finished_at.to_string(),
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_entries
            .iter()
            .map(|(artifact_id, declared_path, realized_path, role)| BenchStageResultOutputV1 {
                artifact_id: (*artifact_id).to_string(),
                declared_path: declared_path.clone(),
                realized_path: path_relative_to_repo(repo_root, realized_path),
                role: (*role).to_string(),
                optional: false,
                exists: true,
            })
            .collect(),
    }
}

fn write_governed_postprocess_input_vcf(path: &Path) -> Result<()> {
    let payload = concat!(
        "##fileformat=VCFv4.2\n",
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts2\ts1\n",
        "chr1\t100\t.\ta\tg,t\t60\t.\tMQ=50\tGT\t0/1\t1/2\n",
        "chr1\t101\t.\tAA\tA\t60\t.\tMQ=45\tGT\t0/1\t0/0\n",
        "chr1\t102\trskeep\tG\tC\t60\t.\tMQ=40\tGT\t0/0\t0/1\n"
    );
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}

fn summarize_postprocess_output(vcf_path: &Path) -> Result<PostprocessOutputSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("postprocess output is missing the #CHROM header"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(ToString::to_string).collect::<Vec<_>>();
    let record_lines = raw.lines().filter(|line| !line.starts_with('#')).collect::<Vec<_>>();
    let mq_removed = record_lines.iter().all(|line| {
        line.split('\t').nth(7).is_some_and(|info| {
            !info.split(';').any(|token| {
                token == "MQ=50" || token == "MQ=45" || token == "MQ=40" || token.starts_with("MQ=")
            })
        })
    });
    let multiallelic_records_remaining = record_lines
        .iter()
        .filter(|line| line.split('\t').nth(4).is_some_and(|alt| alt.contains(',')))
        .count();
    Ok(PostprocessOutputSummary {
        record_count: u64::try_from(record_lines.len())
            .map_err(|_| anyhow!("postprocess output record count overflowed u64"))?,
        sample_count: parse_output_sample_count(vcf_path)?,
        sample_ids,
        mq_removed,
        multiallelic_records_remaining: u64::try_from(multiallelic_records_remaining)
            .map_err(|_| anyhow!("multiallelic record count overflowed u64"))?,
    })
}

fn read_json_value(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn json_bool(value: &Value, pointer: &str) -> Result<bool> {
    value
        .pointer(pointer)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("missing boolean `{pointer}`"))
}

fn json_u64(value: &Value, pointer: &str) -> Result<u64> {
    value.pointer(pointer).and_then(Value::as_u64).ok_or_else(|| anyhow!("missing u64 `{pointer}`"))
}

fn timestamp_marker() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before unix epoch");
    format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{run_local_vcf_postprocess_smoke, write_governed_postprocess_input_vcf};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn governed_postprocess_input_fixture_keeps_multiallelic_and_indel_cases() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = dir.path().join("postprocess_input.vcf");
        write_governed_postprocess_input_vcf(&input).expect("write input fixture");
        let raw = std::fs::read_to_string(&input).expect("read input fixture");
        assert!(raw.contains("g,t"));
        assert!(raw.contains("\tAA\tA\t"));
        assert!(raw.contains("\ts2\ts1"));
    }

    #[test]
    fn local_vcf_postprocess_smoke_emits_governed_contract_outputs() {
        let root = repo_root();
        let report =
            run_local_vcf_postprocess_smoke(&root, "bcftools").expect("run postprocess smoke");
        assert_eq!(report.stage_id, "vcf.postprocess");
        assert_eq!(report.tool_id, "bcftools");
        assert!(root.join(&report.output_vcf_path).exists());
        assert!(root.join(&report.output_tbi_path).exists());
        assert!(root.join(&report.final_manifest_path).exists());
        assert!(root.join(&report.validate_outputs_path).exists());
        assert_eq!(report.sample_ids, vec!["s1".to_string(), "s2".to_string()]);
        assert_eq!(report.multiallelic_records_remaining, 0);
        assert!(report.mq_removed);
        assert!(report.parseable);
    }
}
