use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::metrics::VcfStatsMetricsV1;
use bijux_dna_domain_vcf::params::VcfStatsParams;
use bijux_dna_stages_vcf::pipeline::run_stats_stage_real;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use serde::Serialize;

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_STATS_SMOKE_ROOT: &str = "target/local-smoke/vcf.stats";
const LOCAL_VCF_STATS_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_stats_smoke.v1";
const LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_stats_smoke.metrics.v1";
const LOCAL_VCF_STATS_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-stats-smoke";
const GOVERNED_VCF_STATS_STAGE_ID: &str = "vcf.stats";
const GOVERNED_VCF_STATS_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_STATS_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_STATS_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_STATS_INPUT_FIXTURE_ID: &str = "stats_cohort_minimal";
const GOVERNED_VCF_STATS_SAMPLE_NAME: &str = "cohort_stats";
const DEFAULT_INPUT_VCF_NAME: &str = "stats_input.vcf";
const DEFAULT_OUTPUT_STATS_NAME: &str = "stats.json";
const DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME: &str = "bcftools_stats.txt";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfStatsSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
    sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfStatsTruthSummary {
    variant_count: u64,
    snp_count: u64,
    indel_count: u64,
    transition_count: u64,
    transversion_count: u64,
    sample_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfStatsSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
    pub(crate) transition_count: u64,
    pub(crate) transversion_count: u64,
    pub(crate) ti_tv: f64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfStatsSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
    pub(crate) input_vcf_path: String,
    pub(crate) output_root: String,
    pub(crate) stats_json_path: String,
    pub(crate) bcftools_stats_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
    pub(crate) transition_count: u64,
    pub(crate) transversion_count: u64,
    pub(crate) ti_tv: f64,
    pub(crate) sample_count: u64,
}

pub(crate) fn run_local_vcf_stats_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfStatsSmokeReport> {
    let contract = resolve_governed_vcf_stats_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_STATS_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_stats_input_vcf(&input_vcf)?;
    let truth = summarize_variant_truth(&input_vcf)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_stats_stage_real(
        &input_vcf,
        &stage_root,
        &VcfStatsParams { sample_name: contract.sample_name.clone(), ..VcfStatsParams::default() },
    )
    .with_context(|| format!("run governed VCF stats smoke from {}", input_vcf.display()))?;

    let stats_json_path = output_root.join(DEFAULT_OUTPUT_STATS_NAME);
    fs::copy(&stage_outputs.stats_json, &stats_json_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.stats_json.display(), stats_json_path.display())
    })?;
    let bcftools_stats_path = output_root.join(DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME);
    fs::copy(&stage_outputs.bcftools_stats_txt, &bcftools_stats_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.bcftools_stats_txt.display(),
            bcftools_stats_path.display()
        )
    })?;

    let stats_json = read_stats_json(&stats_json_path)?;
    validate_stats_against_truth(&stats_json, &truth, &contract.sample_name)?;

    let ti_tv = stats_json
        .ti_tv
        .ok_or_else(|| anyhow!("governed VCF stats smoke requires normalized ti_tv"))?;

    let metrics = LocalVcfStatsSmokeMetrics {
        schema_version: LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        variant_count: truth.variant_count,
        snp_count: truth.snp_count,
        indel_count: truth.indel_count,
        transition_count: truth.transition_count,
        transversion_count: truth.transversion_count,
        ti_tv,
        sample_count: truth.sample_count,
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
        &format!("{LOCAL_VCF_STATS_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "stats_json",
                DEFAULT_OUTPUT_STATS_NAME.to_string(),
                stats_json_path.as_path(),
                "report_output",
            ),
            (
                "bcftools_stats_txt",
                DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME.to_string(),
                bcftools_stats_path.as_path(),
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

    Ok(LocalVcfStatsSmokeReport {
        schema_version: LOCAL_VCF_STATS_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_STATS_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: contract.input_fixture_id,
        sample_name: contract.sample_name,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf),
        output_root: path_relative_to_repo(repo_root, &output_root),
        stats_json_path: path_relative_to_repo(repo_root, &stats_json_path),
        bcftools_stats_path: path_relative_to_repo(repo_root, &bcftools_stats_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: metrics.exit_code,
        variant_count: metrics.variant_count,
        snp_count: metrics.snp_count,
        indel_count: metrics.indel_count,
        transition_count: metrics.transition_count,
        transversion_count: metrics.transversion_count,
        ti_tv: metrics.ti_tv,
        sample_count: metrics.sample_count,
    })
}

pub(crate) fn run_vcf_stats_smoke(args: &parse::BenchLocalRunVcfStatsSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_stats_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.stats_json_path);
    }
    Ok(())
}

fn resolve_governed_vcf_stats_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfStatsSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_STATS_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_STATS_STAGE_ID}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_STATS_TOOL_ID {
        bail!(
            "VCF stats smoke requires retained tool `{GOVERNED_VCF_STATS_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF stats smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_STATS_CORPUS_ID {
        bail!(
            "VCF stats smoke requires corpus `{GOVERNED_VCF_STATS_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_STATS_ASSET_PROFILE_ID {
        bail!(
            "VCF stats smoke requires asset profile `{GOVERNED_VCF_STATS_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["stats_json".to_string()] {
        bail!(
            "VCF stats smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfStatsSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_STATS_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_STATS_SAMPLE_NAME.to_string(),
    })
}

fn write_governed_stats_input_vcf(path: &Path) -> Result<()> {
    let payload = "\
##fileformat=VCFv4.2\n\
##reference=bijux-stats-smoke\n\
##contig=<ID=chr1,length=24>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample_a\tsample_b\n\
chr1\t3\trs_transition_1\tA\tG\t60\tPASS\tDP=12\tGT\t0/1\t0/0\n\
chr1\t5\trs_transition_2\tC\tT\t62\tPASS\tDP=14\tGT\t1/1\t0/1\n\
chr1\t7\trs_transversion\tA\tT\t64\tPASS\tDP=16\tGT\t0/1\t1/1\n\
chr1\t9\trs_indel\tAT\tA\t58\tPASS\tDP=18\tGT\t0/1\t0/0\n";
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}

fn summarize_variant_truth(vcf_path: &Path) -> Result<VcfStatsTruthSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut variant_count = 0_u64;
    let mut snp_count = 0_u64;
    let mut indel_count = 0_u64;
    let mut transition_count = 0_u64;
    let mut transversion_count = 0_u64;
    let mut sample_count = 0_u64;

    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_count = line.split('\t').skip(9).count() as u64;
            continue;
        }
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 5 {
            bail!("stats smoke row is malformed: {line}");
        }
        variant_count += 1;
        let ref_allele = fields[3];
        let alt_allele = fields[4];
        if ref_allele.len() == 1 && alt_allele.len() == 1 {
            snp_count += 1;
            if is_transition(ref_allele, alt_allele) {
                transition_count += 1;
            } else {
                transversion_count += 1;
            }
        } else {
            indel_count += 1;
        }
    }

    Ok(VcfStatsTruthSummary {
        variant_count,
        snp_count,
        indel_count,
        transition_count,
        transversion_count,
        sample_count,
    })
}

fn is_transition(ref_allele: &str, alt_allele: &str) -> bool {
    matches!(
        (ref_allele.as_bytes().first().copied(), alt_allele.as_bytes().first().copied()),
        (Some(b'A'), Some(b'G'))
            | (Some(b'G'), Some(b'A'))
            | (Some(b'C'), Some(b'T'))
            | (Some(b'T'), Some(b'C'))
    )
}

fn read_stats_json(path: &Path) -> Result<VcfStatsMetricsV1> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn validate_stats_against_truth(
    stats: &VcfStatsMetricsV1,
    truth: &VcfStatsTruthSummary,
    sample_name: &str,
) -> Result<()> {
    if stats.schema_version != "bijux.vcf.stats.v1" {
        bail!("VCF stats smoke requires normalized stats schema, found `{}`", stats.schema_version);
    }
    if stats.sample_name != sample_name {
        bail!(
            "VCF stats smoke sample name drifted: expected `{sample_name}`, found `{}`",
            stats.sample_name
        );
    }
    if stats.variants_total != truth.variant_count {
        bail!(
            "VCF stats smoke variant count drifted: expected {}, found {}",
            truth.variant_count,
            stats.variants_total
        );
    }
    if stats.snps != truth.snp_count {
        bail!(
            "VCF stats smoke SNP count drifted: expected {}, found {}",
            truth.snp_count,
            stats.snps
        );
    }
    if stats.indels != truth.indel_count {
        bail!(
            "VCF stats smoke indel count drifted: expected {}, found {}",
            truth.indel_count,
            stats.indels
        );
    }
    if stats.sample_count != truth.sample_count {
        bail!(
            "VCF stats smoke sample count drifted: expected {}, found {}",
            truth.sample_count,
            stats.sample_count
        );
    }
    let expected_ti_tv = truth.transition_count as f64 / truth.transversion_count as f64;
    let actual_ti_tv = stats
        .ti_tv
        .ok_or_else(|| anyhow!("VCF stats smoke requires ti_tv in normalized stats output"))?;
    if (actual_ti_tv - expected_ti_tv).abs() > 1e-9 {
        bail!("VCF stats smoke ti/tv drifted: expected {expected_ti_tv}, found {actual_ti_tv}");
    }
    Ok(())
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfStatsSmokeContract,
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
    use super::{
        resolve_governed_vcf_stats_smoke_contract, summarize_variant_truth,
        write_governed_stats_input_vcf,
    };

    #[test]
    fn governed_stats_contract_uses_cohort_matrix_row() {
        let contract =
            resolve_governed_vcf_stats_smoke_contract("bcftools").expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.stats");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "stats_cohort_minimal");
        assert_eq!(contract.sample_name, "cohort_stats");
    }

    #[test]
    fn governed_stats_fixture_matches_expected_truth() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_vcf = dir.path().join("stats_input.vcf");
        write_governed_stats_input_vcf(&input_vcf).expect("write governed input");
        let truth = summarize_variant_truth(&input_vcf).expect("summarize truth");
        assert_eq!(truth.variant_count, 4);
        assert_eq!(truth.snp_count, 3);
        assert_eq!(truth.indel_count, 1);
        assert_eq!(truth.transition_count, 2);
        assert_eq!(truth.transversion_count, 1);
        assert_eq!(truth.sample_count, 2);
    }
}
