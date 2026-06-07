use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{
    assert_bgzip_tabix_artifacts, run_impute_stage, ImputationAcceptMode, ImputeBackend,
    ImputeStageParams,
};
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;

use super::local_stage_result_manifest::{path_relative_to_repo, validate_stage_result_manifest};
use super::local_vcf_panel_workflow_smoke_support::{
    build_stage_result_manifest, governed_vcf_panel_species_context,
    materialize_governed_vcf_panel_assets, resolve_governed_vcf_panel_workflow_smoke_contract,
    DEFAULT_STAGE_RESULT_NAME,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_IMPUTE_SMOKE_ROOT: &str = "target/local-smoke/vcf.impute";
const LOCAL_VCF_IMPUTE_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_impute_smoke.v1";
const LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_impute_smoke.metrics.v1";
const LOCAL_VCF_IMPUTE_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-impute-smoke";
const GOVERNED_VCF_IMPUTE_STAGE_ID: &str = "vcf.impute";
const GOVERNED_VCF_IMPUTE_TOOL_ID: &str = "beagle";
const GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID: &str = "masked_truth_two_sample";
const DEFAULT_INPUT_VCF_NAME: &str = "impute_input.vcf";
const DEFAULT_TRUTH_VCF_NAME: &str = "impute_truth.vcf";
const DEFAULT_OUTPUT_VCF_NAME: &str = "imputed.vcf.gz";
const DEFAULT_OUTPUT_PANEL_ASSETS_NAME: &str = "panel_assets.json";
const DEFAULT_OUTPUT_QC_NAME: &str = "imputation_qc.json";
const DEFAULT_OUTPUT_QC_TSV_NAME: &str = "imputation_qc.tsv";
const DEFAULT_OUTPUT_MANIFEST_NAME: &str = "imputation_manifest.json";
const DEFAULT_OUTPUT_OVERLAP_NAME: &str = "overlap_stats.json";
const DEFAULT_OUTPUT_WARNINGS_NAME: &str = "warnings.json";
const DEFAULT_OUTPUT_ACCEPT_NAME: &str = "imputation_accept.json";
const DEFAULT_OUTPUT_PANEL_MISMATCH_NAME: &str = "panel_mismatch_diagnostics.json";
const DEFAULT_OUTPUT_MAF_BINS_NAME: &str = "maf_bins.tsv";
const DEFAULT_OUTPUT_LOGS_NAME: &str = "logs.txt";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const EXPECTED_SAMPLE_IDS: [&str; 2] = ["masked_sample", "donor_sample"];
const EXPECTED_VARIANT_COUNT: u64 = 2;
const EXPECTED_MASKED_SAMPLE_GT: &str = "0/1";
const EXPECTED_DONOR_SAMPLE_GT: &str = "0/1";
const EXPECTED_MISSING_BEFORE: u64 = 1;
const EXPECTED_MISSING_AFTER: u64 = 0;
const EXPECTED_IMPUTED_GENOTYPES: u64 = 1;
const EXPECTED_LOW_CONFIDENCE_COUNT: u64 = 1;
const EXPECTED_MASKED_TRUTH_SITE_COUNT: u64 = 1;
const EXPECTED_MASKED_TRUTH_MATCH_COUNT: u64 = 1;
const EXPECTED_UNRESOLVED_COUNT: u64 = 0;
const GOVERNED_BEAGLE_SEED: u64 = 42;
const GOVERNED_BEAGLE_THREADS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImputeOutputSummary {
    variant_count: u64,
    sample_ids: Vec<String>,
    masked_sample_gt: String,
    donor_sample_gt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfImputeSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) variant_count: u64,
    pub(crate) missing_before: u64,
    pub(crate) missing_after: u64,
    pub(crate) imputed_genotypes: u64,
    pub(crate) low_confidence_count: u64,
    pub(crate) masked_truth_site_count: u64,
    pub(crate) masked_truth_match_count: u64,
    pub(crate) unresolved_count: u64,
    pub(crate) not_imputable_reasons: BTreeMap<String, u64>,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfImputeSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) input_vcf_path: String,
    pub(crate) truth_vcf_path: String,
    pub(crate) output_root: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) panel_assets_path: String,
    pub(crate) imputation_qc_path: String,
    pub(crate) imputation_qc_tsv_path: String,
    pub(crate) imputation_manifest_path: String,
    pub(crate) overlap_stats_path: String,
    pub(crate) warnings_path: String,
    pub(crate) imputation_accept_path: String,
    pub(crate) panel_mismatch_path: String,
    pub(crate) maf_bins_path: String,
    pub(crate) logs_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) variant_count: u64,
    pub(crate) missing_before: u64,
    pub(crate) missing_after: u64,
    pub(crate) imputed_genotypes: u64,
    pub(crate) low_confidence_count: u64,
    pub(crate) masked_truth_site_count: u64,
    pub(crate) masked_truth_match_count: u64,
    pub(crate) unresolved_count: u64,
    pub(crate) not_imputable_reasons: BTreeMap<String, u64>,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) masked_sample_gt: String,
    pub(crate) donor_sample_gt: String,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

pub(crate) fn run_vcf_impute_smoke(args: &parse::BenchLocalRunVcfImputeSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_impute_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_impute_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfImputeSmokeReport> {
    let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
        GOVERNED_VCF_IMPUTE_STAGE_ID,
        tool_id,
        "imputed_vcf",
    )?;
    let output_root = repo_root.join(DEFAULT_VCF_IMPUTE_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    fs::create_dir_all(&input_root).with_context(|| format!("create {}", input_root.display()))?;
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    let truth_vcf_path = input_root.join(DEFAULT_TRUTH_VCF_NAME);
    write_governed_impute_input_vcf(&input_vcf_path)?;
    write_governed_impute_truth_vcf(&truth_vcf_path)?;
    let panel_assets_report = materialize_governed_vcf_panel_assets(&input_root.join("reference"))
        .with_context(|| {
            format!("materialize governed VCF panel assets under {}", input_root.display())
        })?;
    let panel_assets_path = output_root.join(DEFAULT_OUTPUT_PANEL_ASSETS_NAME);
    bijux_dna_infra::atomic_write_json(&panel_assets_path, &panel_assets_report)?;

    let species_context = governed_vcf_panel_species_context();
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_impute_stage(
        &input_vcf_path,
        &stage_root,
        &species_context,
        &ImputeStageParams {
            species_id: species_context.species_id.clone(),
            build_id: species_context.build_id.clone(),
            backend: ImputeBackend::Beagle,
            panel_id: Some(contract.panel_id.clone()),
            map_id: None,
            threads: GOVERNED_BEAGLE_THREADS,
            seed: GOVERNED_BEAGLE_SEED,
            emit_ds: true,
            emit_gp: true,
            truth_vcf: Some(truth_vcf_path.clone()),
            imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            chunk_window_bp: None,
            chunk_overlap_bp: 0,
        },
    )
    .with_context(|| format!("run governed VCF impute smoke from {}", input_vcf_path.display()))?;

    let output_vcf_path = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.imputed_vcf, &output_vcf_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.imputed_vcf.display(), output_vcf_path.display())
    })?;
    let output_tbi_path = PathBuf::from(format!("{}.tbi", output_vcf_path.display()));
    fs::copy(&stage_outputs.imputed_tbi, &output_tbi_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.imputed_tbi.display(), output_tbi_path.display())
    })?;
    let imputation_qc_path = output_root.join(DEFAULT_OUTPUT_QC_NAME);
    fs::copy(&stage_outputs.imputation_qc_json, &imputation_qc_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.imputation_qc_json.display(),
            imputation_qc_path.display()
        )
    })?;
    let imputation_qc_tsv_path = output_root.join(DEFAULT_OUTPUT_QC_TSV_NAME);
    fs::copy(&stage_outputs.imputation_qc_tsv, &imputation_qc_tsv_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.imputation_qc_tsv.display(),
            imputation_qc_tsv_path.display()
        )
    })?;
    let imputation_manifest_path = output_root.join(DEFAULT_OUTPUT_MANIFEST_NAME);
    fs::copy(&stage_outputs.imputation_manifest_json, &imputation_manifest_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.imputation_manifest_json.display(),
                imputation_manifest_path.display()
            )
        },
    )?;
    let overlap_stats_path = output_root.join(DEFAULT_OUTPUT_OVERLAP_NAME);
    fs::copy(&stage_outputs.overlap_stats_json, &overlap_stats_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.overlap_stats_json.display(),
            overlap_stats_path.display()
        )
    })?;
    let warnings_path = output_root.join(DEFAULT_OUTPUT_WARNINGS_NAME);
    fs::copy(&stage_outputs.warnings_json, &warnings_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.warnings_json.display(), warnings_path.display())
    })?;
    let imputation_accept_path = output_root.join(DEFAULT_OUTPUT_ACCEPT_NAME);
    fs::copy(&stage_outputs.imputation_accept_json, &imputation_accept_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.imputation_accept_json.display(),
                imputation_accept_path.display()
            )
        },
    )?;
    let panel_mismatch_path = output_root.join(DEFAULT_OUTPUT_PANEL_MISMATCH_NAME);
    fs::copy(&stage_outputs.panel_mismatch_diagnostics_json, &panel_mismatch_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.panel_mismatch_diagnostics_json.display(),
                panel_mismatch_path.display()
            )
        },
    )?;
    let maf_bins_path = output_root.join(DEFAULT_OUTPUT_MAF_BINS_NAME);
    fs::copy(&stage_outputs.maf_bin_quality_tsv, &maf_bins_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.maf_bin_quality_tsv.display(),
            maf_bins_path.display()
        )
    })?;
    let logs_path = output_root.join(DEFAULT_OUTPUT_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), logs_path.display())
    })?;

    assert_bgzip_tabix_artifacts(&output_vcf_path, &output_tbi_path)?;
    let validation = vcf_validate_input(
        &output_vcf_path,
        VcfFieldRequirement { require_gt: true, require_gl: true },
    )
    .with_context(|| format!("validate {}", output_vcf_path.display()))?;
    let output_summary = summarize_imputed_output(&output_vcf_path)?;
    if output_summary.variant_count != EXPECTED_VARIANT_COUNT {
        bail!(
            "governed VCF impute smoke expected {} variants, found {}",
            EXPECTED_VARIANT_COUNT,
            output_summary.variant_count
        );
    }
    if output_summary.sample_ids
        != EXPECTED_SAMPLE_IDS.iter().map(|value| value.to_string()).collect::<Vec<_>>()
    {
        bail!(
            "governed VCF impute smoke expected sample ids {:?}, found {:?}",
            EXPECTED_SAMPLE_IDS,
            output_summary.sample_ids
        );
    }
    if output_summary.masked_sample_gt != EXPECTED_MASKED_SAMPLE_GT {
        bail!(
            "governed VCF impute smoke expected masked sample GT `{EXPECTED_MASKED_SAMPLE_GT}`, found `{}`",
            output_summary.masked_sample_gt
        );
    }
    if output_summary.donor_sample_gt != EXPECTED_DONOR_SAMPLE_GT {
        bail!(
            "governed VCF impute smoke expected donor sample GT `{EXPECTED_DONOR_SAMPLE_GT}`, found `{}`",
            output_summary.donor_sample_gt
        );
    }

    let imputation_qc = read_json(&imputation_qc_path)?;
    if imputation_qc.get("backend").and_then(serde_json::Value::as_str) != Some("beagle") {
        bail!("imputation QC report drifted away from beagle backend");
    }
    if imputation_qc.pointer("/concordance/truth_provided").and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        bail!("imputation QC report drifted away from masked truth validation");
    }
    let missing_before = read_u64(&imputation_qc, "/missing_genotypes_before")?;
    let missing_after = read_u64(&imputation_qc, "/missing_genotypes_after")?;
    let imputed_genotypes = read_u64(&imputation_qc, "/imputed_genotypes")?;
    let low_confidence_count = read_u64(&imputation_qc, "/low_confidence_count")?;
    let masked_truth_site_count = read_u64(&imputation_qc, "/concordance/masked_truth_site_count")?;
    let masked_truth_match_count = read_u64(&imputation_qc, "/concordance/imputed_match_count")?;
    let unresolved_count = read_u64(&imputation_qc, "/concordance/unresolved_count")?;
    let not_imputable_reasons = imputation_qc
        .get("not_imputable_reasons")
        .and_then(serde_json::Value::as_object)
        .map(|rows| {
            rows.iter()
                .filter_map(|(key, value)| value.as_u64().map(|count| (key.clone(), count)))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    if missing_before != EXPECTED_MISSING_BEFORE
        || missing_after != EXPECTED_MISSING_AFTER
        || imputed_genotypes != EXPECTED_IMPUTED_GENOTYPES
        || low_confidence_count != EXPECTED_LOW_CONFIDENCE_COUNT
        || masked_truth_site_count != EXPECTED_MASKED_TRUTH_SITE_COUNT
        || masked_truth_match_count != EXPECTED_MASKED_TRUTH_MATCH_COUNT
        || unresolved_count != EXPECTED_UNRESOLVED_COUNT
    {
        bail!(
            "governed VCF impute smoke drifted from masked-truth contract: before={} after={} imputed={} low_confidence={} masked_sites={} matches={} unresolved={}",
            missing_before,
            missing_after,
            imputed_genotypes,
            low_confidence_count,
            masked_truth_site_count,
            masked_truth_match_count,
            unresolved_count
        );
    }
    if !not_imputable_reasons.is_empty() {
        bail!(
            "governed VCF impute smoke expected zero unresolved reasons, found {:?}",
            not_imputable_reasons
        );
    }

    let sample_count = u64::try_from(output_summary.sample_ids.len())
        .map_err(|_| anyhow!("sample count overflow"))?;
    let metrics = LocalVcfImputeSmokeMetrics {
        schema_version: LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel_id.clone(),
        map_id: contract.map_id.clone(),
        variant_count: output_summary.variant_count,
        missing_before,
        missing_after,
        imputed_genotypes,
        low_confidence_count,
        masked_truth_site_count,
        masked_truth_match_count,
        unresolved_count,
        not_imputable_reasons: not_imputable_reasons.clone(),
        sample_count,
        sample_ids: output_summary.sample_ids.clone(),
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
        &format!("{LOCAL_VCF_IMPUTE_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "imputed_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf_path.as_path(),
                "vcf_output",
            ),
            (
                "imputed_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi_path.as_path(),
                "index_output",
            ),
            (
                "panel_assets_json",
                DEFAULT_OUTPUT_PANEL_ASSETS_NAME.to_string(),
                panel_assets_path.as_path(),
                "report_output",
            ),
            (
                "imputation_qc_json",
                DEFAULT_OUTPUT_QC_NAME.to_string(),
                imputation_qc_path.as_path(),
                "report_output",
            ),
            (
                "imputation_qc_tsv",
                DEFAULT_OUTPUT_QC_TSV_NAME.to_string(),
                imputation_qc_tsv_path.as_path(),
                "table_output",
            ),
            (
                "imputation_manifest_json",
                DEFAULT_OUTPUT_MANIFEST_NAME.to_string(),
                imputation_manifest_path.as_path(),
                "report_output",
            ),
            (
                "overlap_stats_json",
                DEFAULT_OUTPUT_OVERLAP_NAME.to_string(),
                overlap_stats_path.as_path(),
                "report_output",
            ),
            (
                "warnings_json",
                DEFAULT_OUTPUT_WARNINGS_NAME.to_string(),
                warnings_path.as_path(),
                "report_output",
            ),
            (
                "imputation_accept_json",
                DEFAULT_OUTPUT_ACCEPT_NAME.to_string(),
                imputation_accept_path.as_path(),
                "report_output",
            ),
            (
                "panel_mismatch_diagnostics_json",
                DEFAULT_OUTPUT_PANEL_MISMATCH_NAME.to_string(),
                panel_mismatch_path.as_path(),
                "report_output",
            ),
            (
                "maf_bins_tsv",
                DEFAULT_OUTPUT_MAF_BINS_NAME.to_string(),
                maf_bins_path.as_path(),
                "table_output",
            ),
            ("logs_txt", DEFAULT_OUTPUT_LOGS_NAME.to_string(), logs_path.as_path(), "log_output"),
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

    Ok(LocalVcfImputeSmokeReport {
        schema_version: LOCAL_VCF_IMPUTE_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_IMPUTE_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel_id,
        map_id: contract.map_id,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        truth_vcf_path: path_relative_to_repo(repo_root, &truth_vcf_path),
        output_root: path_relative_to_repo(repo_root, &output_root),
        output_vcf_path: path_relative_to_repo(repo_root, &output_vcf_path),
        output_tbi_path: path_relative_to_repo(repo_root, &output_tbi_path),
        panel_assets_path: path_relative_to_repo(repo_root, &panel_assets_path),
        imputation_qc_path: path_relative_to_repo(repo_root, &imputation_qc_path),
        imputation_qc_tsv_path: path_relative_to_repo(repo_root, &imputation_qc_tsv_path),
        imputation_manifest_path: path_relative_to_repo(repo_root, &imputation_manifest_path),
        overlap_stats_path: path_relative_to_repo(repo_root, &overlap_stats_path),
        warnings_path: path_relative_to_repo(repo_root, &warnings_path),
        imputation_accept_path: path_relative_to_repo(repo_root, &imputation_accept_path),
        panel_mismatch_path: path_relative_to_repo(repo_root, &panel_mismatch_path),
        maf_bins_path: path_relative_to_repo(repo_root, &maf_bins_path),
        logs_path: path_relative_to_repo(repo_root, &logs_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: 0,
        variant_count: output_summary.variant_count,
        missing_before,
        missing_after,
        imputed_genotypes,
        low_confidence_count,
        masked_truth_site_count,
        masked_truth_match_count,
        unresolved_count,
        not_imputable_reasons,
        sample_count,
        sample_ids: output_summary.sample_ids,
        masked_sample_gt: output_summary.masked_sample_gt,
        donor_sample_gt: output_summary.donor_sample_gt,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn write_governed_impute_input_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF impute smoke input path has no parent directory"))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##reference=GRCh38\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\tdonor_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t./.\t0/1\n\
1\t140\t.\tC\tT\t60\tPASS\t.\tGT\t0/0\t0/0\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

fn write_governed_impute_truth_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF impute smoke truth path has no parent directory"))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##reference=GRCh38\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\tdonor_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/1\n\
1\t140\t.\tC\tT\t60\tPASS\t.\tGT\t0/0\t0/0\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

fn summarize_imputed_output(vcf_path: &Path) -> Result<ImputeOutputSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("imputed VCF is missing the #CHROM header"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    let first_record = raw
        .lines()
        .find(|line| !line.starts_with('#') && !line.trim().is_empty())
        .ok_or_else(|| anyhow!("imputed VCF is missing records"))?;
    let fields = first_record.split('\t').collect::<Vec<_>>();
    let gt_idx = format_index(&fields, "GT").ok_or_else(|| anyhow!("imputed VCF missing GT"))?;
    let masked_sample_gt = fields
        .get(9)
        .and_then(|sample| sample.split(':').collect::<Vec<_>>().get(gt_idx).copied())
        .ok_or_else(|| anyhow!("imputed VCF missing masked sample GT"))?
        .to_string();
    let donor_sample_gt = fields
        .get(10)
        .and_then(|sample| sample.split(':').collect::<Vec<_>>().get(gt_idx).copied())
        .ok_or_else(|| anyhow!("imputed VCF missing donor sample GT"))?
        .to_string();
    let variant_count =
        raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()).count() as u64;
    Ok(ImputeOutputSummary { variant_count, sample_ids, masked_sample_gt, donor_sample_gt })
}

fn format_index(fields: &[&str], name: &str) -> Option<usize> {
    fields.get(8)?.split(':').position(|field| field == name)
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn read_u64(value: &serde_json::Value, pointer: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing `{pointer}` in imputation QC payload"))
}

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{
        run_local_vcf_impute_smoke, summarize_imputed_output, write_governed_impute_input_vcf,
        write_governed_impute_truth_vcf,
    };

    #[test]
    fn governed_impute_fixture_tracks_masked_input_and_truth() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_path = dir.path().join("input.vcf");
        let truth_path = dir.path().join("truth.vcf");
        write_governed_impute_input_vcf(&input_path).expect("write input");
        write_governed_impute_truth_vcf(&truth_path).expect("write truth");

        let input_summary = summarize_imputed_output(&input_path).expect("summarize input");
        let truth_summary = summarize_imputed_output(&truth_path).expect("summarize truth");
        assert_eq!(input_summary.variant_count, 2);
        assert_eq!(input_summary.masked_sample_gt, "./.");
        assert_eq!(truth_summary.masked_sample_gt, "0/1");
        assert_eq!(truth_summary.donor_sample_gt, "0/1");
    }

    #[test]
    fn governed_vcf_impute_smoke_reports_masked_truth_match() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        let report =
            run_local_vcf_impute_smoke(repo_root.path(), "beagle").expect("run local impute smoke");
        assert_eq!(report.stage_id, "vcf.impute");
        assert_eq!(report.tool_id, "beagle");
        assert_eq!(report.missing_before, 1);
        assert_eq!(report.missing_after, 0);
        assert_eq!(report.imputed_genotypes, 1);
        assert_eq!(report.low_confidence_count, 1);
        assert_eq!(report.masked_truth_site_count, 1);
        assert_eq!(report.masked_truth_match_count, 1);
        assert_eq!(report.unresolved_count, 0);
        assert_eq!(report.masked_sample_gt, "0/1");
        assert_eq!(report.donor_sample_gt, "0/1");
        assert!(report.parseable);
        assert!(report.gl_present);
    }
}
