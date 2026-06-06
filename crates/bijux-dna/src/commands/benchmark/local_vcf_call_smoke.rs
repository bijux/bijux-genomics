use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::params::VcfCallParams;
use bijux_dna_stages_vcf::metrics::parse_vcf_call_summary;
use bijux_dna_stages_vcf::pipeline::run_call_diploid_stage;
use bijux_dna_stages_vcf::vcf_io::{vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;

use super::local_stage_result_manifest::path_relative_to_repo;
use super::local_stage_result_manifest::validate_stage_result_manifest;
use super::local_vcf_call_bam_smoke_support::{
    build_stage_result_manifest, materialize_reference_fasta, parse_output_sample_count,
    resolve_governed_vcf_call_bam_smoke_contract, GovernedVcfCallBamSmokeContract,
    DEFAULT_STAGE_RESULT_NAME,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_CALL_SMOKE_ROOT: &str = "target/local-smoke/vcf.call";
const LOCAL_VCF_CALL_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_call_smoke.v1";
const LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_smoke.metrics.v1";
const LOCAL_VCF_CALL_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-call-smoke";
const GOVERNED_VCF_CALL_STAGE_ID: &str = "vcf.call";
const GOVERNED_VCF_CALL_RESOLVED_STAGE_ID: &str = "vcf.call_diploid";
const DEFAULT_OUTPUT_VCF_NAME: &str = "calls.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfCallSmokeContract {
    stage_id: String,
    resolved_stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_bam: String,
    input_bam_index: String,
    reference: String,
    sample_id: String,
    sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfCallSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) resolved_stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_id: String,
    pub(crate) input_bam: String,
    pub(crate) reference: String,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfCallSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) resolved_stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_id: String,
    pub(crate) sample_name: String,
    pub(crate) input_bam: String,
    pub(crate) input_bam_index: String,
    pub(crate) reference: String,
    pub(crate) materialized_reference_path: String,
    pub(crate) output_root: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) variant_count: u64,
    pub(crate) snp_count: u64,
    pub(crate) indel_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

pub(crate) fn run_vcf_call_smoke(args: &parse::BenchLocalRunVcfCallSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_call_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_call_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfCallSmokeReport> {
    let contract = resolve_governed_vcf_call_smoke_contract(repo_root, tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_CALL_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    fs::create_dir_all(&artifacts_root)
        .with_context(|| format!("create {}", artifacts_root.display()))?;

    let input_bam = repo_root.join(&contract.input_bam);
    let reference = repo_root.join(&contract.reference);
    let materialized_reference = materialize_reference_fasta(&reference, &artifacts_root)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_call_diploid_stage(
        &input_bam,
        &artifacts_root,
        &VcfCallParams {
            caller: contract.tool_id.clone(),
            sample_name: contract.sample_name.clone(),
            reference_fasta: Some(materialized_reference.display().to_string()),
            ..VcfCallParams::default()
        },
    )
    .with_context(|| {
        format!(
            "run governed VCF call smoke for `{}` from {}",
            contract.tool_id,
            input_bam.display()
        )
    })?;

    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.called_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.called_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.called_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.called_tbi.display(), output_tbi.display())
    })?;

    let validation = vcf_validate_input(
        &output_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", output_vcf.display()))?;
    let call_summary = parse_vcf_call_summary(&output_vcf, &contract.sample_name)
        .with_context(|| format!("parse call summary from {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;

    let metrics = LocalVcfCallSmokeMetrics {
        schema_version: LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        resolved_stage_id: contract.resolved_stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        variant_count: call_summary.variants_called,
        snp_count: call_summary.snps,
        indel_count: call_summary.indels,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let base_contract = GovernedVcfCallBamSmokeContract {
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_bam: contract.input_bam.clone(),
        input_bam_index: contract.input_bam_index.clone(),
        reference: contract.reference.clone(),
        sample_id: contract.sample_id.clone(),
        sample_name: contract.sample_name.clone(),
    };
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        &base_contract,
        &format!("{LOCAL_VCF_CALL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            ("called_vcf", DEFAULT_OUTPUT_VCF_NAME.to_string(), output_vcf.as_path(), "vcf_output"),
            (
                "vcf_index",
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

    Ok(LocalVcfCallSmokeReport {
        schema_version: LOCAL_VCF_CALL_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_CALL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        resolved_stage_id: contract.resolved_stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        sample_id: contract.sample_id,
        sample_name: contract.sample_name,
        input_bam: contract.input_bam,
        input_bam_index: contract.input_bam_index,
        reference: contract.reference,
        materialized_reference_path: path_relative_to_repo(repo_root, &materialized_reference),
        output_root: path_relative_to_repo(repo_root, &output_root),
        output_vcf_path: path_relative_to_repo(repo_root, &output_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &output_tbi),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: metrics.exit_code,
        variant_count: metrics.variant_count,
        snp_count: metrics.snp_count,
        indel_count: metrics.indel_count,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn resolve_governed_vcf_call_smoke_contract(
    repo_root: &Path,
    requested_tool_id: &str,
) -> Result<GovernedVcfCallSmokeContract> {
    let base_contract = resolve_governed_vcf_call_bam_smoke_contract(
        repo_root,
        GOVERNED_VCF_CALL_STAGE_ID,
        requested_tool_id,
        "called_vcf",
    )?;
    Ok(GovernedVcfCallSmokeContract {
        stage_id: base_contract.stage_id,
        resolved_stage_id: GOVERNED_VCF_CALL_RESOLVED_STAGE_ID.to_string(),
        tool_id: base_contract.tool_id,
        corpus_id: base_contract.corpus_id,
        input_bam: base_contract.input_bam,
        input_bam_index: base_contract.input_bam_index,
        reference: base_contract.reference,
        sample_id: base_contract.sample_id,
        sample_name: base_contract.sample_name,
    })
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
        parse_output_sample_count, resolve_governed_vcf_call_smoke_contract,
        GOVERNED_VCF_CALL_STAGE_ID,
    };
    use crate::commands::benchmark::local_vcf_call_bam_smoke_support::GOVERNED_VCF_CALL_TOOL_ID;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_call_smoke_contract_uses_governed_matrix_and_bam_fixture() {
        let repo_root = repo_root();
        let contract =
            resolve_governed_vcf_call_smoke_contract(&repo_root, GOVERNED_VCF_CALL_TOOL_ID)
                .expect("resolve governed vcf call smoke contract");

        assert_eq!(contract.stage_id, GOVERNED_VCF_CALL_STAGE_ID);
        assert_eq!(contract.resolved_stage_id, "vcf.call_diploid");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.sample_id, "human_like_validation");
        assert_eq!(contract.sample_name, "core-v1-pass");
        assert_eq!(
            contract.input_bam,
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam"
        );
        assert_eq!(
            contract.reference,
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        );
    }

    #[test]
    fn parse_output_sample_count_reads_governed_fixture_vcf() {
        let repo_root = repo_root();
        let fixture_vcf =
            repo_root.join("tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf");
        let sample_count = parse_output_sample_count(&fixture_vcf).expect("parse sample count");
        assert_eq!(sample_count, 4);
    }
}
