use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::params::VcfFilterParams;
use bijux_dna_stages_vcf::pipeline::run_filter_stage_real;
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_call_bam_smoke_support::parse_output_sample_count;
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_FILTER_SMOKE_ROOT: &str = "target/local-smoke/vcf.filter";
const LOCAL_VCF_FILTER_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_filter_smoke.v1";
const LOCAL_VCF_FILTER_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_filter_smoke.metrics.v1";
const LOCAL_VCF_FILTER_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-filter-smoke";
const GOVERNED_VCF_FILTER_STAGE_ID: &str = "vcf.filter";
const GOVERNED_VCF_FILTER_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_FILTER_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_FILTER_ASSET_PROFILE_ID: &str = "vcf_single_sample";
const GOVERNED_VCF_FILTER_INPUT_FIXTURE_ID: &str = "site_filter_single_sample";
const GOVERNED_VCF_FILTER_SAMPLE_NAME: &str = "sample_a";
const DEFAULT_INPUT_VCF_NAME: &str = "filter_input.vcf";
const DEFAULT_OUTPUT_VCF_NAME: &str = "filtered.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const FILTER_BREAKDOWN_JSON_NAME: &str = "filter_breakdown.json";
const FILTER_BREAKDOWN_TSV_NAME: &str = "filter_breakdown.tsv";
const FILTER_EXPLAIN_JSON_NAME: &str = "filter_explain.json";
const EXPECTED_FILTER_IDS: &[&str] = &["HIGH_MISSING", "LOWQUAL", "LOW_DP", "LOW_MQ"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfFilterSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
    sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilteredOutputSummary {
    pass_variants: u64,
    failed_variants: u64,
    filter_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfFilterSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) input_variants: u64,
    pub(crate) pass_variants: u64,
    pub(crate) failed_variants: u64,
    pub(crate) filter_ids: Vec<String>,
    pub(crate) depth_threshold: f64,
    pub(crate) quality_threshold: f64,
    pub(crate) missingness_threshold: f64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfFilterSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
    pub(crate) input_vcf_path: String,
    pub(crate) output_root: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) metrics_path: String,
    pub(crate) filter_breakdown_path: String,
    pub(crate) filter_breakdown_tsv_path: String,
    pub(crate) filter_explain_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) input_variants: u64,
    pub(crate) pass_variants: u64,
    pub(crate) failed_variants: u64,
    pub(crate) filter_ids: Vec<String>,
    pub(crate) depth_threshold: f64,
    pub(crate) quality_threshold: f64,
    pub(crate) missingness_threshold: f64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

pub(crate) fn run_local_vcf_filter_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfFilterSmokeReport> {
    let contract = resolve_governed_vcf_filter_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_FILTER_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_filter_input_vcf(&input_vcf)?;
    let input_variants = parse_vcf_record_count(&input_vcf)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_filter_stage_real(
        &input_vcf,
        &stage_root,
        &VcfFilterParams {
            sample_name: contract.sample_name.clone(),
            min_qual: 30.0,
            require_pass: false,
            normalize: true,
            require_bgzip_tabix: true,
            production_profile: false,
            ..VcfFilterParams::default()
        },
    )
    .with_context(|| format!("run governed VCF filter smoke from {}", input_vcf.display()))?;

    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.filtered_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.filtered_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.filtered_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.filtered_tbi.display(), output_tbi.display())
    })?;
    let filter_breakdown_path = output_root.join(FILTER_BREAKDOWN_JSON_NAME);
    fs::copy(&stage_outputs.filter_breakdown_json, &filter_breakdown_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.filter_breakdown_json.display(),
            filter_breakdown_path.display()
        )
    })?;
    let filter_breakdown_tsv_path = output_root.join(FILTER_BREAKDOWN_TSV_NAME);
    fs::copy(&stage_outputs.filter_breakdown_tsv, &filter_breakdown_tsv_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.filter_breakdown_tsv.display(),
                filter_breakdown_tsv_path.display()
            )
        },
    )?;
    let filter_explain_path = output_root.join(FILTER_EXPLAIN_JSON_NAME);
    fs::copy(&stage_outputs.filter_explain_json, &filter_explain_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.filter_explain_json.display(),
            filter_explain_path.display()
        )
    })?;

    let validation = vcf_validate_input(
        &output_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed VCF filter smoke expects exactly one sample, found {sample_count}");
    }
    let output_summary = summarize_filtered_output(&output_vcf)?;
    if output_summary.pass_variants + output_summary.failed_variants != input_variants {
        bail!(
            "governed VCF filter smoke output count drifted: input={} pass={} failed={}",
            input_variants,
            output_summary.pass_variants,
            output_summary.failed_variants
        );
    }
    let expected_filter_ids =
        EXPECTED_FILTER_IDS.iter().map(|id| (*id).to_string()).collect::<Vec<_>>();
    if output_summary.filter_ids != expected_filter_ids {
        bail!(
            "governed VCF filter smoke expected filter ids {:?}, found {:?}",
            expected_filter_ids,
            output_summary.filter_ids
        );
    }

    let filter_breakdown = read_json(&filter_breakdown_path)?;
    let pass_count = filter_breakdown
        .pointer("/counts/PASS")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("filter breakdown is missing PASS count"))?;
    if pass_count != output_summary.pass_variants {
        bail!(
            "VCF filter smoke PASS count drifted between VCF and breakdown: {} vs {}",
            output_summary.pass_variants,
            pass_count
        );
    }
    for expected in EXPECTED_FILTER_IDS {
        let count = filter_breakdown
            .pointer(&format!("/counts/{expected}"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if count != 1 {
            bail!("governed VCF filter smoke expected one `{expected}` row, found {count}");
        }
    }

    let filter_explain = read_json(&filter_explain_path)?;
    let output_subset = filter_explain
        .pointer("/filter_scope/output_subset")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("filter explain is missing output subset"))?;
    if output_subset != "retain_tagged_records" {
        bail!(
            "governed VCF filter smoke requires tagged-record retention, found `{output_subset}`"
        );
    }
    let depth_threshold = extract_threshold(&filter_explain, "/thresholds/min_depth", "min_depth")?;
    let quality_threshold = extract_threshold(&filter_explain, "/thresholds/min_qual", "min_qual")?;
    let missingness_threshold = extract_threshold(
        &filter_explain,
        "/thresholds/sample_missingness_max",
        "sample_missingness_max",
    )?;

    let metrics = LocalVcfFilterSmokeMetrics {
        schema_version: LOCAL_VCF_FILTER_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_variants,
        pass_variants: output_summary.pass_variants,
        failed_variants: output_summary.failed_variants,
        filter_ids: output_summary.filter_ids.clone(),
        depth_threshold,
        quality_threshold,
        missingness_threshold,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        &contract,
        &format!("{LOCAL_VCF_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "filtered_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "vcf_index",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi.as_path(),
                "vcf_index",
            ),
            (
                "filter_breakdown_json",
                FILTER_BREAKDOWN_JSON_NAME.to_string(),
                filter_breakdown_path.as_path(),
                "report_output",
            ),
            (
                "filter_breakdown_tsv",
                FILTER_BREAKDOWN_TSV_NAME.to_string(),
                filter_breakdown_tsv_path.as_path(),
                "report_output",
            ),
            (
                "filter_explain_json",
                FILTER_EXPLAIN_JSON_NAME.to_string(),
                filter_explain_path.as_path(),
                "report_output",
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

    Ok(LocalVcfFilterSmokeReport {
        schema_version: LOCAL_VCF_FILTER_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: contract.input_fixture_id,
        sample_name: contract.sample_name,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf),
        output_root: path_relative_to_repo(repo_root, &output_root),
        output_vcf_path: path_relative_to_repo(repo_root, &output_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &output_tbi),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        filter_breakdown_path: path_relative_to_repo(repo_root, &filter_breakdown_path),
        filter_breakdown_tsv_path: path_relative_to_repo(repo_root, &filter_breakdown_tsv_path),
        filter_explain_path: path_relative_to_repo(repo_root, &filter_explain_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: metrics.exit_code,
        input_variants: metrics.input_variants,
        pass_variants: metrics.pass_variants,
        failed_variants: metrics.failed_variants,
        filter_ids: metrics.filter_ids,
        depth_threshold: metrics.depth_threshold,
        quality_threshold: metrics.quality_threshold,
        missingness_threshold: metrics.missingness_threshold,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

pub(crate) fn run_vcf_filter_smoke(args: &parse::BenchLocalRunVcfFilterSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_filter_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

fn resolve_governed_vcf_filter_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfFilterSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_FILTER_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_FILTER_STAGE_ID}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_FILTER_TOOL_ID {
        bail!(
            "VCF filter smoke requires retained tool `{GOVERNED_VCF_FILTER_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF filter smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_FILTER_CORPUS_ID {
        bail!(
            "VCF filter smoke requires corpus `{GOVERNED_VCF_FILTER_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_FILTER_ASSET_PROFILE_ID {
        bail!(
            "VCF filter smoke requires asset profile `{GOVERNED_VCF_FILTER_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["filtered_vcf".to_string()] {
        bail!(
            "VCF filter smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfFilterSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_FILTER_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_FILTER_SAMPLE_NAME.to_string(),
    })
}

fn write_governed_filter_input_vcf(path: &Path) -> Result<()> {
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=bijux-filter-smoke\n\
##contig=<ID=chr1,length=24>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##INFO=<ID=MQ,Number=1,Type=Float,Description=\"Mapping quality\">\n\
##INFO=<ID=FS,Number=1,Type=Float,Description=\"Strand bias\">\n\
##INFO=<ID=AF,Number=A,Type=Float,Description=\"Alternate allele frequency\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{sample}\n\
chr1\t3\trs_pass\tA\tG\t60\tPASS\tDP=12;MQ=55;FS=10;AF=0.40\tGT\t0/1\n\
chr1\t5\trs_lowqual\tC\tT\t20\tPASS\tDP=14;MQ=50;FS=9;AF=0.45\tGT\t0/1\n\
chr1\t7\trs_lowdp\tG\tA\t62\tPASS\tDP=4;MQ=52;FS=8;AF=0.35\tGT\t0/1\n\
chr1\t9\trs_lowmq\tT\tC\t64\tPASS\tDP=13;MQ=20;FS=7;AF=0.30\tGT\t0/1\n\
chr1\t11\trs_missing\tA\tC\t68\tPASS\tDP=18;MQ=60;FS=6;AF=0.25\tGT\t./.\n",
        sample = GOVERNED_VCF_FILTER_SAMPLE_NAME,
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}

fn parse_vcf_record_count(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    let count =
        raw.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).count();
    u64::try_from(count).map_err(|_| anyhow!("VCF record count overflowed u64"))
}

fn summarize_filtered_output(vcf_path: &Path) -> Result<FilteredOutputSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut pass_variants = 0_u64;
    let mut failed_variants = 0_u64;
    let mut filter_ids = BTreeSet::<String>::new();
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 8 {
            bail!("filter smoke row is missing FILTER field: {line}");
        }
        let filter_field = parts[6];
        if filter_field == "PASS" || filter_field == "." {
            pass_variants += 1;
            continue;
        }
        failed_variants += 1;
        for tag in filter_field.split(';') {
            if !tag.is_empty() && tag != "PASS" && tag != "." {
                filter_ids.insert(tag.to_string());
            }
        }
    }
    Ok(FilteredOutputSummary {
        pass_variants,
        failed_variants,
        filter_ids: filter_ids.into_iter().collect(),
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn extract_threshold(report: &serde_json::Value, pointer: &str, name: &str) -> Result<f64> {
    report
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("filter explain is missing `{name}` threshold"))
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfFilterSmokeContract,
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

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use bijux_dna_domain_vcf::params::VcfFilterParams;
    use bijux_dna_stages_vcf::pipeline::run_filter_stage_real;

    use super::{
        parse_vcf_record_count, resolve_governed_vcf_filter_smoke_contract,
        summarize_filtered_output, write_governed_filter_input_vcf,
    };

    #[test]
    fn governed_filter_contract_uses_single_sample_matrix_row() {
        let contract =
            resolve_governed_vcf_filter_smoke_contract("bcftools").expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.filter");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "site_filter_single_sample");
        assert_eq!(contract.sample_name, "sample_a");
    }

    #[test]
    fn governed_filter_fixture_contains_five_records() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_vcf = dir.path().join("filter_input.vcf");
        write_governed_filter_input_vcf(&input_vcf).expect("write governed input");
        let record_count = parse_vcf_record_count(&input_vcf).expect("count records");
        assert_eq!(record_count, 5);
    }

    #[test]
    fn governed_filter_fixture_exercises_expected_filter_tags() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_vcf = dir.path().join("filter_input.vcf");
        write_governed_filter_input_vcf(&input_vcf).expect("write governed input");
        let out = run_filter_stage_real(
            &input_vcf,
            dir.path(),
            &VcfFilterParams {
                sample_name: "sample_a".to_string(),
                min_qual: 30.0,
                require_pass: false,
                normalize: true,
                require_bgzip_tabix: true,
                production_profile: false,
                ..VcfFilterParams::default()
            },
        )
        .expect("run filter stage");
        let summary = summarize_filtered_output(&out.filtered_vcf).expect("summarize output");
        assert_eq!(summary.pass_variants, 1);
        assert_eq!(summary.failed_variants, 4);
        assert_eq!(
            summary.filter_ids,
            vec![
                "HIGH_MISSING".to_string(),
                "LOWQUAL".to_string(),
                "LOW_DP".to_string(),
                "LOW_MQ".to_string(),
            ]
        );
    }
}
