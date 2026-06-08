use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::run_qc_stage;
use bijux_dna_stages_vcf::pipeline::QcStageParams;
use serde::{Deserialize, Serialize};

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_QC_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.qc";
const LOCAL_VCF_QC_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_qc_smoke.v1";
const LOCAL_VCF_QC_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_qc_smoke.metrics.v1";
const LOCAL_VCF_QC_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-qc-smoke";
const GOVERNED_VCF_QC_STAGE_ID: &str = "vcf.qc";
const GOVERNED_VCF_QC_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_QC_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_QC_ASSET_PROFILE_ID: &str = "vcf_cohort";
const GOVERNED_VCF_QC_INPUT_FIXTURE_ID: &str = "qc_cohort_missingness";
const GOVERNED_VCF_QC_SAMPLE_NAME: &str = "qc_cohort";
const DEFAULT_INPUT_VCF_NAME: &str = "qc_input.vcf";
const DEFAULT_OUTPUT_QC_NAME: &str = "qc.json";
const DEFAULT_OUTPUT_SUMMARY_NAME: &str = "qc_summary.json";
const DEFAULT_OUTPUT_QC_TABLES_NAME: &str = "qc_tables.tsv";
const DEFAULT_OUTPUT_IMPUTATION_QC_NAME: &str = "imputation_qc.tsv";
const DEFAULT_OUTPUT_WARNINGS_NAME: &str = "warnings.json";
const DEFAULT_OUTPUT_HISTOGRAMS_NAME: &str = "qc_histograms.json";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const EXPECTED_EXCLUDED_SAMPLE_ID: &str = "qc_sparse";
const EXPECTED_EXCLUDED_VARIANT_ID: &str = "chr1:30:G:A";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfQcSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
    sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalVcfQcSmokeSampleMissingnessRow {
    pub(crate) sample_id: String,
    pub(crate) total_genotype_count: u64,
    pub(crate) missing_genotype_count: u64,
    pub(crate) missingness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalVcfQcSmokeVariantMissingnessRow {
    pub(crate) variant_id: String,
    pub(crate) contig: String,
    pub(crate) position: u64,
    pub(crate) reference: String,
    pub(crate) alternate: String,
    pub(crate) total_sample_count: u64,
    pub(crate) missing_sample_count: u64,
    pub(crate) missingness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalVcfQcSmokeMafSummary {
    pub(crate) allele_frequency_mean: f64,
    pub(crate) maf_bin_counts: BTreeMap<String, u64>,
    pub(crate) observed_variant_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalVcfQcSmokeHeterozygosity {
    pub(crate) heterozygous_call_count: u64,
    pub(crate) homozygous_alt_call_count: u64,
    pub(crate) het_hom_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalVcfQcSmokeHweSummary {
    pub(crate) tested_variant_count: u64,
    pub(crate) pvalue_mean: Option<f64>,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LocalVcfQcSummary {
    sample_missingness: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    variant_missingness: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    maf_summary: LocalVcfQcSmokeMafSummary,
    heterozygosity: LocalVcfQcSmokeHeterozygosity,
    hwe_summary: LocalVcfQcSmokeHweSummary,
    excluded_samples: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    excluded_variants: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    sample_missingness_exclusion_threshold: f64,
    variant_missingness_exclusion_threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfQcSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_missingness: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub(crate) variant_missingness: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub(crate) maf_summary: LocalVcfQcSmokeMafSummary,
    pub(crate) heterozygosity: LocalVcfQcSmokeHeterozygosity,
    pub(crate) hwe_summary: LocalVcfQcSmokeHweSummary,
    pub(crate) excluded_samples: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub(crate) excluded_variants: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub(crate) sample_missingness_exclusion_threshold: f64,
    pub(crate) variant_missingness_exclusion_threshold: f64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfQcSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
    pub(crate) input_vcf_path: String,
    pub(crate) output_root: String,
    pub(crate) qc_json_path: String,
    pub(crate) qc_summary_path: String,
    pub(crate) qc_tables_path: String,
    pub(crate) imputation_qc_path: String,
    pub(crate) warnings_path: String,
    pub(crate) qc_histograms_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) sample_missingness: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub(crate) variant_missingness: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub(crate) maf_summary: LocalVcfQcSmokeMafSummary,
    pub(crate) heterozygosity: LocalVcfQcSmokeHeterozygosity,
    pub(crate) hwe_summary: LocalVcfQcSmokeHweSummary,
    pub(crate) excluded_samples: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub(crate) excluded_variants: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub(crate) sample_missingness_exclusion_threshold: f64,
    pub(crate) variant_missingness_exclusion_threshold: f64,
}

pub(crate) fn run_local_vcf_qc_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfQcSmokeReport> {
    let contract = resolve_governed_vcf_qc_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_QC_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_qc_input_vcf(&input_vcf)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_qc_stage(
        &input_vcf,
        &stage_root,
        &QcStageParams {
            sample_name: contract.sample_name.clone(),
            is_ancient_dna: false,
            allow_hwe_for_ancient: false,
            production_profile: false,
            pre_filter_vcf: None,
        },
    )
    .with_context(|| format!("run governed VCF QC smoke from {}", input_vcf.display()))?;

    let qc_summary_path = output_root.join(DEFAULT_OUTPUT_SUMMARY_NAME);
    fs::copy(&stage_outputs.qc_summary_json, &qc_summary_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.qc_summary_json.display(), qc_summary_path.display())
    })?;
    let qc_tables_path = output_root.join(DEFAULT_OUTPUT_QC_TABLES_NAME);
    fs::copy(&stage_outputs.qc_tables_tsv, &qc_tables_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.qc_tables_tsv.display(), qc_tables_path.display())
    })?;
    let imputation_qc_path = output_root.join(DEFAULT_OUTPUT_IMPUTATION_QC_NAME);
    fs::copy(&stage_outputs.imputation_qc_tsv, &imputation_qc_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.imputation_qc_tsv.display(),
            imputation_qc_path.display()
        )
    })?;
    let warnings_path = output_root.join(DEFAULT_OUTPUT_WARNINGS_NAME);
    fs::copy(&stage_outputs.warnings_json, &warnings_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.warnings_json.display(), warnings_path.display())
    })?;
    let qc_histograms_path = output_root.join(DEFAULT_OUTPUT_HISTOGRAMS_NAME);
    fs::copy(&stage_outputs.qc_histograms_json, &qc_histograms_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.qc_histograms_json.display(),
            qc_histograms_path.display()
        )
    })?;

    let summary = read_qc_summary(&qc_summary_path)?;
    validate_qc_summary(&summary)?;

    let metrics = LocalVcfQcSmokeMetrics {
        schema_version: LOCAL_VCF_QC_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_missingness: summary.sample_missingness.clone(),
        variant_missingness: summary.variant_missingness.clone(),
        maf_summary: summary.maf_summary.clone(),
        heterozygosity: summary.heterozygosity.clone(),
        hwe_summary: summary.hwe_summary.clone(),
        excluded_samples: summary.excluded_samples.clone(),
        excluded_variants: summary.excluded_variants.clone(),
        sample_missingness_exclusion_threshold: summary.sample_missingness_exclusion_threshold,
        variant_missingness_exclusion_threshold: summary.variant_missingness_exclusion_threshold,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let qc_json_path = output_root.join(DEFAULT_OUTPUT_QC_NAME);
    let report = LocalVcfQcSmokeReport {
        schema_version: LOCAL_VCF_QC_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_QC_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_name: contract.sample_name.clone(),
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf),
        output_root: path_relative_to_repo(repo_root, &output_root),
        qc_json_path: path_relative_to_repo(repo_root, &qc_json_path),
        qc_summary_path: path_relative_to_repo(repo_root, &qc_summary_path),
        qc_tables_path: path_relative_to_repo(repo_root, &qc_tables_path),
        imputation_qc_path: path_relative_to_repo(repo_root, &imputation_qc_path),
        warnings_path: path_relative_to_repo(repo_root, &warnings_path),
        qc_histograms_path: path_relative_to_repo(repo_root, &qc_histograms_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        sample_missingness: metrics.sample_missingness.clone(),
        variant_missingness: metrics.variant_missingness.clone(),
        maf_summary: metrics.maf_summary.clone(),
        heterozygosity: metrics.heterozygosity.clone(),
        hwe_summary: metrics.hwe_summary.clone(),
        excluded_samples: metrics.excluded_samples.clone(),
        excluded_variants: metrics.excluded_variants.clone(),
        sample_missingness_exclusion_threshold: metrics.sample_missingness_exclusion_threshold,
        variant_missingness_exclusion_threshold: metrics.variant_missingness_exclusion_threshold,
    };
    bijux_dna_infra::atomic_write_json(&qc_json_path, &report)?;
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        &contract,
        &format!("{LOCAL_VCF_QC_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "qc_report_json",
                DEFAULT_OUTPUT_QC_NAME.to_string(),
                qc_json_path.as_path(),
                "report_output",
            ),
            (
                "qc_summary_json",
                DEFAULT_OUTPUT_SUMMARY_NAME.to_string(),
                qc_summary_path.as_path(),
                "report_output",
            ),
            (
                "qc_tables_tsv",
                DEFAULT_OUTPUT_QC_TABLES_NAME.to_string(),
                qc_tables_path.as_path(),
                "report_output",
            ),
            (
                "imputation_qc_tsv",
                DEFAULT_OUTPUT_IMPUTATION_QC_NAME.to_string(),
                imputation_qc_path.as_path(),
                "report_output",
            ),
            (
                "warnings_json",
                DEFAULT_OUTPUT_WARNINGS_NAME.to_string(),
                warnings_path.as_path(),
                "report_output",
            ),
            (
                "qc_histograms_json",
                DEFAULT_OUTPUT_HISTOGRAMS_NAME.to_string(),
                qc_histograms_path.as_path(),
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
    Ok(report)
}

pub(crate) fn run_vcf_qc_smoke(args: &parse::BenchLocalRunVcfQcSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_qc_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.qc_json_path);
    }
    Ok(())
}

pub(crate) fn resolve_governed_vcf_qc_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfQcSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_QC_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_QC_STAGE_ID}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_QC_TOOL_ID {
        bail!(
            "VCF QC smoke requires retained tool `{GOVERNED_VCF_QC_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_QC_CORPUS_ID {
        bail!(
            "VCF QC smoke requires corpus `{GOVERNED_VCF_QC_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_QC_ASSET_PROFILE_ID {
        bail!(
            "VCF QC smoke requires asset profile `{GOVERNED_VCF_QC_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF QC smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    Ok(GovernedVcfQcSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_QC_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_QC_SAMPLE_NAME.to_string(),
    })
}

fn write_governed_qc_input_vcf(output_path: &Path) -> Result<()> {
    let parent =
        output_path.parent().ok_or_else(|| anyhow!("QC input VCF path has no parent directory"))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##INFO=<ID=INFO,Number=1,Type=Float,Description=\"Imputation info\">\n\
##INFO=<ID=R2,Number=1,Type=Float,Description=\"Imputation R2\">\n\
##INFO=<ID=AF,Number=1,Type=Float,Description=\"Allele frequency\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tqc_ref\tqc_sparse\tqc_balanced\n\
chr1\t10\t.\tA\tG\t60\tPASS\tDP=20;INFO=0.95;R2=0.90;AF=0.10\tGT\t0/1\t./.\t0/0\n\
chr1\t20\t.\tC\tT\t62\tPASS\tDP=18;INFO=0.90;R2=0.88;AF=0.25\tGT\t1/1\t./.\t0/1\n\
chr1\t30\t.\tG\tA\t59\tLOWQUAL\tDP=16;INFO=0.82;R2=0.80;AF=0.05\tGT\t0/0\t./.\t./.\n\
chr1\t40\t.\tT\tG\t65\tPASS\tDP=22;INFO=0.93;R2=0.91;AF=0.40\tGT\t0/1\t1/1\t0/1\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

fn read_qc_summary(path: &Path) -> Result<LocalVcfQcSummary> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse QC summary {}", path.display()))
}

fn validate_qc_summary(summary: &LocalVcfQcSummary) -> Result<()> {
    if summary.sample_missingness_exclusion_threshold != 0.5 {
        bail!(
            "governed VCF QC smoke expected sample exclusion threshold 0.5, found {}",
            summary.sample_missingness_exclusion_threshold
        );
    }
    if summary.variant_missingness_exclusion_threshold != 0.5 {
        bail!(
            "governed VCF QC smoke expected variant exclusion threshold 0.5, found {}",
            summary.variant_missingness_exclusion_threshold
        );
    }
    if summary.maf_summary.observed_variant_count != 4 {
        bail!(
            "governed VCF QC smoke expected 4 observed variants, found {}",
            summary.maf_summary.observed_variant_count
        );
    }
    if summary.heterozygosity.heterozygous_call_count != 4 {
        bail!(
            "governed VCF QC smoke expected 4 heterozygous calls, found {}",
            summary.heterozygosity.heterozygous_call_count
        );
    }
    if summary.heterozygosity.homozygous_alt_call_count != 2 {
        bail!(
            "governed VCF QC smoke expected 2 homozygous-alt calls, found {}",
            summary.heterozygosity.homozygous_alt_call_count
        );
    }
    if summary.heterozygosity.het_hom_ratio != Some(2.0) {
        bail!(
            "governed VCF QC smoke expected het/hom ratio 2.0, found {:?}",
            summary.heterozygosity.het_hom_ratio
        );
    }
    if summary.hwe_summary.tested_variant_count != 3 {
        bail!(
            "governed VCF QC smoke expected 3 HWE-tested variants, found {}",
            summary.hwe_summary.tested_variant_count
        );
    }
    if summary.hwe_summary.pvalue_mean != Some(0.825656) {
        bail!(
            "governed VCF QC smoke expected HWE p-value mean 0.825656, found {:?}",
            summary.hwe_summary.pvalue_mean
        );
    }
    if summary.hwe_summary.status != "computed_modern" {
        bail!(
            "governed VCF QC smoke expected HWE status `computed_modern`, found `{}`",
            summary.hwe_summary.status
        );
    }

    let qc_sparse = summary
        .sample_missingness
        .iter()
        .find(|row| row.sample_id == EXPECTED_EXCLUDED_SAMPLE_ID)
        .ok_or_else(|| {
            anyhow!("governed VCF QC smoke is missing sample `{EXPECTED_EXCLUDED_SAMPLE_ID}`")
        })?;
    if qc_sparse.missingness != 0.75 {
        bail!(
            "governed VCF QC smoke expected `{EXPECTED_EXCLUDED_SAMPLE_ID}` missingness 0.75, found {}",
            qc_sparse.missingness
        );
    }
    if summary.excluded_samples.len() != 1
        || summary.excluded_samples[0].sample_id != EXPECTED_EXCLUDED_SAMPLE_ID
    {
        bail!(
            "governed VCF QC smoke expected exactly one excluded sample `{EXPECTED_EXCLUDED_SAMPLE_ID}`, found {:?}",
            summary
                .excluded_samples
                .iter()
                .map(|row| row.sample_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    let excluded_variant = summary
        .variant_missingness
        .iter()
        .find(|row| row.variant_id == EXPECTED_EXCLUDED_VARIANT_ID)
        .ok_or_else(|| {
            anyhow!("governed VCF QC smoke is missing variant `{EXPECTED_EXCLUDED_VARIANT_ID}`")
        })?;
    if excluded_variant.missingness != (2.0 / 3.0) {
        bail!(
            "governed VCF QC smoke expected `{EXPECTED_EXCLUDED_VARIANT_ID}` missingness {}, found {}",
            2.0 / 3.0,
            excluded_variant.missingness
        );
    }
    if summary.excluded_variants.len() != 1
        || summary.excluded_variants[0].variant_id != EXPECTED_EXCLUDED_VARIANT_ID
    {
        bail!(
            "governed VCF QC smoke expected exactly one excluded variant `{EXPECTED_EXCLUDED_VARIANT_ID}`, found {:?}",
            summary
                .excluded_variants
                .iter()
                .map(|row| row.variant_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    Ok(())
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfQcSmokeContract,
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
        resolve_governed_vcf_qc_smoke_contract, run_local_vcf_qc_smoke, write_governed_qc_input_vcf,
    };

    #[test]
    fn governed_qc_contract_uses_cohort_matrix_row() {
        let contract = resolve_governed_vcf_qc_smoke_contract("plink2").expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.qc");
        assert_eq!(contract.tool_id, "plink2");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "qc_cohort_missingness");
        assert_eq!(contract.sample_name, "qc_cohort");
    }

    #[test]
    fn governed_qc_fixture_surfaces_named_exclusions() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        let report = run_local_vcf_qc_smoke(repo_root.path(), "plink2").expect("run local smoke");
        assert_eq!(report.stage_id, "vcf.qc");
        assert_eq!(report.tool_id, "plink2");
        assert_eq!(report.sample_missingness_exclusion_threshold, 0.5);
        assert_eq!(report.variant_missingness_exclusion_threshold, 0.5);
        assert_eq!(report.excluded_samples.len(), 1);
        assert_eq!(report.excluded_samples[0].sample_id, "qc_sparse");
        assert_eq!(report.excluded_variants.len(), 1);
        assert_eq!(report.excluded_variants[0].variant_id, "chr1:30:G:A");
        assert_eq!(report.maf_summary.observed_variant_count, 4);
        assert_eq!(report.heterozygosity.het_hom_ratio, Some(2.0));
        assert_eq!(report.hwe_summary.tested_variant_count, 3);
        assert_eq!(report.hwe_summary.pvalue_mean, Some(0.825656));
        assert_eq!(report.hwe_summary.status, "computed_modern");
    }

    #[test]
    fn governed_qc_fixture_writer_creates_named_samples() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_vcf = dir.path().join("qc_input.vcf");
        write_governed_qc_input_vcf(&input_vcf).expect("write governed input");
        let raw = std::fs::read_to_string(&input_vcf).expect("read input");
        assert!(raw.contains("\tqc_ref\tqc_sparse\tqc_balanced\n"));
        assert!(raw.contains("chr1\t30\t.\tG\tA\t59\tLOWQUAL"));
    }
}
