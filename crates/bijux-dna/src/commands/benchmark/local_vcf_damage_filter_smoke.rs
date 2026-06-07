use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{
    run_damage_filter_stage, DamageFilterStageParams, DamageUdgRegime,
};
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
use crate::commands::router::runtime::ProcessEnvGuard;

const DEFAULT_VCF_DAMAGE_FILTER_SMOKE_ROOT: &str = "target/local-smoke/vcf.damage_filter";
const LOCAL_VCF_DAMAGE_FILTER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_damage_filter_smoke.v1";
const LOCAL_VCF_DAMAGE_FILTER_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_damage_filter_smoke.metrics.v1";
const LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-damage-filter-smoke";
const GOVERNED_VCF_DAMAGE_FILTER_STAGE_ID: &str = "vcf.damage_filter";
const GOVERNED_VCF_DAMAGE_FILTER_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_DAMAGE_FILTER_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_DAMAGE_FILTER_ASSET_PROFILE_ID: &str = "vcf_single_sample";
const GOVERNED_VCF_DAMAGE_FILTER_INPUT_FIXTURE_ID: &str = "terminal_damage_single_sample";
const GOVERNED_VCF_DAMAGE_FILTER_SAMPLE_NAME: &str = "sample_a";
const DEFAULT_INPUT_VCF_NAME: &str = "damage_input.vcf";
const DEFAULT_OUTPUT_VCF_NAME: &str = "damage_filtered.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const DAMAGE_FILTER_SUMMARY_NAME: &str = "damage_filter_summary.json";
const DAMAGE_FILTER_COUNTS_NAME: &str = "damage_filter_counts.json";
const DAMAGE_FILTER_WARNINGS_NAME: &str = "warnings.json";
const DAMAGE_FILTER_MANIFEST_NAME: &str = "damage_genotype_manifest.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfDamageFilterSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
    sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfDamageFilterSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) input_variants: u64,
    pub(crate) removed_variants: u64,
    pub(crate) retained_variants: u64,
    pub(crate) low_quality_filtered_variants: u64,
    pub(crate) damage_ratio_filtered_variants: u64,
    pub(crate) terminal_damage_filtered_variants: u64,
    pub(crate) damage_context_rule: String,
    pub(crate) terminal_context_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfDamageFilterSmokeReport {
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
    pub(crate) summary_path: String,
    pub(crate) counts_path: String,
    pub(crate) warnings_path: String,
    pub(crate) damage_manifest_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) input_variants: u64,
    pub(crate) removed_variants: u64,
    pub(crate) retained_variants: u64,
    pub(crate) low_quality_filtered_variants: u64,
    pub(crate) damage_ratio_filtered_variants: u64,
    pub(crate) terminal_damage_filtered_variants: u64,
    pub(crate) damage_context_rule: String,
    pub(crate) terminal_context_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

pub(crate) fn run_vcf_damage_filter_smoke(
    args: &parse::BenchLocalRunVcfDamageFilterSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_damage_filter_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_damage_filter_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfDamageFilterSmokeReport> {
    let contract = resolve_governed_vcf_damage_filter_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_DAMAGE_FILTER_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_damage_filter_input_vcf(&input_vcf)?;
    let input_variants = parse_vcf_record_count(&input_vcf)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let params = DamageFilterStageParams {
        udg_regime: DamageUdgRegime::NonUdg,
        strict_regime: true,
        min_qual: 30.0,
        max_damage_ratio: 0.35,
    };
    let _env_guard = ProcessEnvGuard::capture(&[
        "BIJUX_VCF_DAMAGE_MASK_MODE",
        "BIJUX_VCF_DAMAGE_TERMINAL_THRESHOLD",
        "BIJUX_VCF_DAMAGE_PMD_MIN",
        "BIJUX_LIBRARY_TYPE",
    ]);
    std::env::set_var("BIJUX_VCF_DAMAGE_MASK_MODE", "remove");
    std::env::set_var("BIJUX_VCF_DAMAGE_TERMINAL_THRESHOLD", "0.50");
    std::env::set_var("BIJUX_VCF_DAMAGE_PMD_MIN", "3.0");
    std::env::remove_var("BIJUX_LIBRARY_TYPE");
    let stage_outputs =
        run_damage_filter_stage(&input_vcf, &stage_root, &params).with_context(|| {
            format!("run governed VCF damage_filter smoke from {}", input_vcf.display())
        })?;

    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.filtered_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.filtered_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.filtered_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.filtered_tbi.display(), output_tbi.display())
    })?;
    let summary_path = output_root.join(DAMAGE_FILTER_SUMMARY_NAME);
    fs::copy(&stage_outputs.damage_filter_summary_json, &summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.damage_filter_summary_json.display(),
            summary_path.display()
        )
    })?;
    let counts_path = output_root.join(DAMAGE_FILTER_COUNTS_NAME);
    fs::copy(&stage_outputs.damage_filter_counts_json, &counts_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.damage_filter_counts_json.display(),
            counts_path.display()
        )
    })?;
    let warnings_path = output_root.join(DAMAGE_FILTER_WARNINGS_NAME);
    fs::copy(&stage_outputs.warnings_json, &warnings_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.warnings_json.display(), warnings_path.display())
    })?;
    let damage_manifest_path = output_root.join(DAMAGE_FILTER_MANIFEST_NAME);
    fs::copy(&stage_outputs.damage_genotype_manifest_json, &damage_manifest_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.damage_genotype_manifest_json.display(),
                damage_manifest_path.display()
            )
        },
    )?;

    let validation =
        vcf_validate_input(&output_vcf, VcfFieldRequirement { require_gt: true, require_gl: true })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let retained_variants = parse_vcf_record_count(&output_vcf)?;
    if retained_variants >= input_variants {
        bail!(
            "governed damage_filter smoke must remove at least one variant, found input={input_variants} output={retained_variants}"
        );
    }
    let removed_variants = input_variants - retained_variants;
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed damage_filter smoke expects exactly one sample, found {sample_count}");
    }

    let summary_json = read_json(&summary_path)?;
    let counts_json = read_json(&counts_path)?;
    let terminal_context_count = summary_json
        .pointer("/prefilter/read_position_signal/ct_five_prime_high")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + summary_json
            .pointer("/prefilter/read_position_signal/ga_three_prime_high")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
    if terminal_context_count == 0 {
        bail!("governed damage_filter smoke requires at least one terminal damage context row");
    }
    let low_quality_filtered_variants = count_value(&counts_json, "/counts/low_qual");
    let damage_ratio_filtered_variants = count_value(&counts_json, "/counts/damage_ratio_exceeded");
    let terminal_damage_filtered_variants =
        count_value(&counts_json, "/counts/terminal_damage_filtered");
    let damage_context_rule = format_damage_context_rule(&summary_json)?;

    let metrics = LocalVcfDamageFilterSmokeMetrics {
        schema_version: LOCAL_VCF_DAMAGE_FILTER_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_variants,
        removed_variants,
        retained_variants,
        low_quality_filtered_variants,
        damage_ratio_filtered_variants,
        terminal_damage_filtered_variants,
        damage_context_rule: damage_context_rule.clone(),
        terminal_context_count,
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
        &format!("{LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "damage_filtered_vcf",
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
                "damage_filter_summary_json",
                DAMAGE_FILTER_SUMMARY_NAME.to_string(),
                summary_path.as_path(),
                "report_output",
            ),
            (
                "damage_filter_counts_json",
                DAMAGE_FILTER_COUNTS_NAME.to_string(),
                counts_path.as_path(),
                "report_output",
            ),
            (
                "warnings_json",
                DAMAGE_FILTER_WARNINGS_NAME.to_string(),
                warnings_path.as_path(),
                "report_output",
            ),
            (
                "damage_genotype_manifest_json",
                DAMAGE_FILTER_MANIFEST_NAME.to_string(),
                damage_manifest_path.as_path(),
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

    Ok(LocalVcfDamageFilterSmokeReport {
        schema_version: LOCAL_VCF_DAMAGE_FILTER_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
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
        summary_path: path_relative_to_repo(repo_root, &summary_path),
        counts_path: path_relative_to_repo(repo_root, &counts_path),
        warnings_path: path_relative_to_repo(repo_root, &warnings_path),
        damage_manifest_path: path_relative_to_repo(repo_root, &damage_manifest_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: metrics.exit_code,
        input_variants: metrics.input_variants,
        removed_variants: metrics.removed_variants,
        retained_variants: metrics.retained_variants,
        low_quality_filtered_variants: metrics.low_quality_filtered_variants,
        damage_ratio_filtered_variants: metrics.damage_ratio_filtered_variants,
        terminal_damage_filtered_variants: metrics.terminal_damage_filtered_variants,
        damage_context_rule: metrics.damage_context_rule,
        terminal_context_count: metrics.terminal_context_count,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn resolve_governed_vcf_damage_filter_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfDamageFilterSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_DAMAGE_FILTER_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_DAMAGE_FILTER_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_DAMAGE_FILTER_TOOL_ID {
        bail!(
            "VCF damage_filter smoke requires retained tool `{GOVERNED_VCF_DAMAGE_FILTER_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF damage_filter smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_DAMAGE_FILTER_CORPUS_ID {
        bail!(
            "VCF damage_filter smoke requires corpus `{GOVERNED_VCF_DAMAGE_FILTER_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_DAMAGE_FILTER_ASSET_PROFILE_ID {
        bail!(
            "VCF damage_filter smoke requires asset profile `{GOVERNED_VCF_DAMAGE_FILTER_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["damage_filtered_vcf".to_string()] {
        bail!(
            "VCF damage_filter smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfDamageFilterSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_DAMAGE_FILTER_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_DAMAGE_FILTER_SAMPLE_NAME.to_string(),
    })
}

fn write_governed_damage_filter_input_vcf(path: &Path) -> Result<()> {
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=bijux-damage-filter-smoke\n\
##contig=<ID=chr1,length=12>\n\
##contig=<ID=chr2,length=12>\n\
##INFO=<ID=CT_GA_DAMAGE_RATIO,Number=1,Type=Float,Description=\"C>T or G>A damage ratio\">\n\
##INFO=<ID=DEAM5P,Number=1,Type=Float,Description=\"5 prime terminal deamination signal\">\n\
##INFO=<ID=DEAM3P,Number=1,Type=Float,Description=\"3 prime terminal deamination signal\">\n\
##INFO=<ID=PMD_SCORE,Number=1,Type=Float,Description=\"postmortem damage score\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled genotype likelihoods\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{sample}\n\
chr1\t3\trs_keep\tA\tG\t60\tPASS\tCT_GA_DAMAGE_RATIO=0.02;DEAM5P=0.01;DEAM3P=0.01;PMD_SCORE=5\tGT:PL\t0/1:0,18,36\n\
chr1\t5\trs_damage\tC\tT\t72\tPASS\tCT_GA_DAMAGE_RATIO=0.80;DEAM5P=0.90;DEAM3P=0.10;PMD_SCORE=5\tGT:PL\t0/1:0,12,24\n\
chr2\t7\trs_transition_keep\tG\tA\t68\tPASS\tCT_GA_DAMAGE_RATIO=0.25;DEAM5P=0.05;DEAM3P=0.20;PMD_SCORE=5\tGT:PL\t0/1:0,14,28\n\
chr2\t9\trs_lowqual\tC\tT\t25\tPASS\tCT_GA_DAMAGE_RATIO=0.10;DEAM5P=0.10;DEAM3P=0.05;PMD_SCORE=5\tGT:PL\t0/1:0,20,40\n",
        sample = GOVERNED_VCF_DAMAGE_FILTER_SAMPLE_NAME,
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

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn count_value(value: &serde_json::Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(serde_json::Value::as_u64).unwrap_or(0)
}

fn format_damage_context_rule(summary_json: &serde_json::Value) -> Result<String> {
    let mode = summary_json
        .pointer("/masking_strategy/mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("damage_filter summary is missing masking mode"))?;
    let max_damage_ratio = summary_json
        .pointer("/thresholds/max_damage_ratio")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("damage_filter summary is missing max_damage_ratio"))?;
    let terminal_damage_threshold = summary_json
        .pointer("/thresholds/terminal_damage_threshold")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("damage_filter summary is missing terminal_damage_threshold"))?;
    let pmd_min = summary_json
        .pointer("/thresholds/pmd_min")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("damage_filter summary is missing pmd_min"))?;
    Ok(format!(
        "{mode}_ct_ga_with_ratio_gt_{max_damage_ratio:.2}_or_terminal_signal_ge_{terminal_damage_threshold:.2}_or_pmd_lt_{pmd_min:.1}"
    ))
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfDamageFilterSmokeContract,
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
    use std::path::PathBuf;

    use super::{
        format_damage_context_rule, parse_vcf_record_count,
        resolve_governed_vcf_damage_filter_smoke_contract, write_governed_damage_filter_input_vcf,
    };

    #[test]
    fn governed_damage_filter_contract_uses_single_sample_matrix_row() {
        let contract = resolve_governed_vcf_damage_filter_smoke_contract("bcftools")
            .expect("resolve governed damage filter contract");
        assert_eq!(contract.stage_id, "vcf.damage_filter");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "terminal_damage_single_sample");
        assert_eq!(contract.sample_name, "sample_a");
    }

    #[test]
    fn governed_damage_filter_fixture_contains_four_records() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_vcf = dir.path().join("damage_input.vcf");
        write_governed_damage_filter_input_vcf(&input_vcf).expect("write governed input");
        let record_count = parse_vcf_record_count(&input_vcf).expect("count records");
        assert_eq!(record_count, 4);
    }

    #[test]
    fn damage_context_rule_uses_stage_thresholds() {
        let summary = serde_json::json!({
            "masking_strategy": {"mode": "remove"},
            "thresholds": {
                "max_damage_ratio": 0.35,
                "terminal_damage_threshold": 0.50,
                "pmd_min": 3.0
            }
        });
        let rule = format_damage_context_rule(&summary).expect("format damage rule");
        assert_eq!(
            rule,
            "remove_ct_ga_with_ratio_gt_0.35_or_terminal_signal_ge_0.50_or_pmd_lt_3.0"
        );
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn parse_vcf_record_count_reads_governed_single_sample_fixture() {
        let repo_root = repo_root();
        let fixture_vcf = repo_root
            .join("benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf");
        let record_count = parse_vcf_record_count(&fixture_vcf).expect("count fixture records");
        assert_eq!(record_count, 2);
    }
}
