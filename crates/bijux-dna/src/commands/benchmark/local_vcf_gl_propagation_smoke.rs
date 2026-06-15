use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{run_gl_propagation_stage, GlPropagationStageParams};
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

const DEFAULT_VCF_GL_PROPAGATION_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.gl_propagation";
const LOCAL_VCF_GL_PROPAGATION_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_gl_propagation_smoke.v1";
const LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_gl_propagation_smoke.metrics.v1";
const LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-gl-propagation-smoke";
const GOVERNED_VCF_GL_PROPAGATION_STAGE_ID: &str = "vcf.gl_propagation";
const GOVERNED_VCF_GL_PROPAGATION_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID: &str = "vcf_single_sample";
const GOVERNED_VCF_GL_PROPAGATION_INPUT_FIXTURE_ID: &str = "likelihood_single_sample";
const GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME: &str = "sample_a";
const DEFAULT_INPUT_VCF_NAME: &str = "gl_input.vcf";
const DEFAULT_OUTPUT_VCF_NAME: &str = "propagated.vcf.gz";
const DEFAULT_OUTPUT_BCF_NAME: &str = "propagated.bcf";
const DEFAULT_OUTPUT_REPORT_NAME: &str = "gl_propagation_report.json";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const GOVERNED_RETAINED_LIKELIHOOD_FIELDS: &[&str] = &["GL", "PL", "GP"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfGlPropagationSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
    sample_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LikelihoodFieldSummary {
    fields: BTreeSet<String>,
    site_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfGlPropagationSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) input_likelihood_fields: Vec<String>,
    pub(crate) output_likelihood_fields: Vec<String>,
    pub(crate) lost_fields: Vec<String>,
    pub(crate) site_count_before: u64,
    pub(crate) site_count_after: u64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfGlPropagationSmokeReport {
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
    pub(crate) output_bcf_path: String,
    pub(crate) output_bcf_csi_path: String,
    pub(crate) report_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) input_likelihood_fields: Vec<String>,
    pub(crate) output_likelihood_fields: Vec<String>,
    pub(crate) lost_fields: Vec<String>,
    pub(crate) site_count_before: u64,
    pub(crate) site_count_after: u64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

pub(crate) fn run_vcf_gl_propagation_smoke(
    args: &parse::BenchLocalRunVcfGlPropagationSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_gl_propagation_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_gl_propagation_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfGlPropagationSmokeReport> {
    let contract = resolve_governed_vcf_gl_propagation_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_GL_PROPAGATION_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_gl_propagation_input_vcf(&input_vcf)?;
    let input_summary = summarize_likelihood_fields(&input_vcf)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs =
        run_gl_propagation_stage(&input_vcf, &stage_root, &GlPropagationStageParams::recommended())
            .with_context(|| {
                format!("run governed VCF gl_propagation smoke from {}", input_vcf.display())
            })?;

    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.normalized_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.normalized_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.normalized_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.normalized_tbi.display(), output_tbi.display())
    })?;
    let bcf_path = output_root.join(DEFAULT_OUTPUT_BCF_NAME);
    let normalized_bcf = stage_outputs
        .normalized_bcf
        .as_ref()
        .ok_or_else(|| anyhow!("gl_propagation smoke expected normalized_bcf output"))?;
    fs::copy(normalized_bcf, &bcf_path)
        .with_context(|| format!("copy {} to {}", normalized_bcf.display(), bcf_path.display()))?;
    let bcf_index_path = PathBuf::from(format!("{}.csi", bcf_path.display()));
    let normalized_bcf_csi = stage_outputs
        .normalized_bcf_csi
        .as_ref()
        .ok_or_else(|| anyhow!("gl_propagation smoke expected normalized_bcf_csi output"))?;
    fs::copy(normalized_bcf_csi, &bcf_index_path).with_context(|| {
        format!("copy {} to {}", normalized_bcf_csi.display(), bcf_index_path.display())
    })?;
    let report_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    fs::copy(&stage_outputs.gl_propagation_report_json, &report_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.gl_propagation_report_json.display(),
            report_path.display()
        )
    })?;

    let validation =
        vcf_validate_input(&output_vcf, VcfFieldRequirement { require_gt: true, require_gl: true })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let output_summary = summarize_likelihood_fields(&output_vcf)?;
    let lost_fields =
        input_summary.fields.difference(&output_summary.fields).cloned().collect::<Vec<_>>();
    if !lost_fields.is_empty() {
        bail!("governed gl_propagation smoke lost likelihood fields: {}", lost_fields.join(","));
    }
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed gl_propagation smoke expects exactly one sample, found {sample_count}");
    }

    let metrics = LocalVcfGlPropagationSmokeMetrics {
        schema_version: LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_likelihood_fields: input_summary.fields.iter().cloned().collect(),
        output_likelihood_fields: output_summary.fields.iter().cloned().collect(),
        lost_fields,
        site_count_before: input_summary.site_count,
        site_count_after: output_summary.site_count,
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
        &format!("{LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "gl_propagated_vcf",
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
                "gl_propagated_bcf",
                DEFAULT_OUTPUT_BCF_NAME.to_string(),
                bcf_path.as_path(),
                "bcf_output",
            ),
            (
                "bcf_index",
                format!("{DEFAULT_OUTPUT_BCF_NAME}.csi"),
                bcf_index_path.as_path(),
                "bcf_index",
            ),
            (
                "gl_propagation_report",
                DEFAULT_OUTPUT_REPORT_NAME.to_string(),
                report_path.as_path(),
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

    Ok(LocalVcfGlPropagationSmokeReport {
        schema_version: LOCAL_VCF_GL_PROPAGATION_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: contract.input_fixture_id,
        sample_name: contract.sample_name,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf),
        output_root: path_relative_to_repo(repo_root, &output_root),
        output_vcf_path: path_relative_to_repo(repo_root, &output_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &output_tbi),
        output_bcf_path: path_relative_to_repo(repo_root, &bcf_path),
        output_bcf_csi_path: path_relative_to_repo(repo_root, &bcf_index_path),
        report_path: path_relative_to_repo(repo_root, &report_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: metrics.exit_code,
        input_likelihood_fields: metrics.input_likelihood_fields,
        output_likelihood_fields: metrics.output_likelihood_fields,
        lost_fields: metrics.lost_fields,
        site_count_before: metrics.site_count_before,
        site_count_after: metrics.site_count_after,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn resolve_governed_vcf_gl_propagation_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfGlPropagationSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_GL_PROPAGATION_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_GL_PROPAGATION_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_GL_PROPAGATION_TOOL_ID {
        bail!(
            "VCF gl_propagation smoke requires retained tool `{GOVERNED_VCF_GL_PROPAGATION_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF gl_propagation smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID {
        bail!(
            "VCF gl_propagation smoke requires corpus `{GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID {
        bail!(
            "VCF gl_propagation smoke requires asset profile `{GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["gl_propagated_vcf".to_string()] {
        bail!(
            "VCF gl_propagation smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfGlPropagationSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_GL_PROPAGATION_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME.to_string(),
    })
}

fn write_governed_gl_propagation_input_vcf(path: &Path) -> Result<()> {
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=bijux-gl-propagation-smoke\n\
##contig=<ID=chr1,length=12>\n\
##contig=<ID=chr2,length=12>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
##FORMAT=<ID=GL,Number=G,Type=Float,Description=\"Genotype Likelihood\">\n\
##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled Genotype Likelihood\">\n\
##FORMAT=<ID=GP,Number=G,Type=Float,Description=\"Genotype Posterior\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{sample}\n\
chr1\t3\trs1\tA\tG\t60\tPASS\t.\tGT:GL:PL:GP\t0/1:0.0,-1.0,-2.0:10,0,20:0.05,0.90,0.05\n\
chr1\t7\trs2\tC\tT\t55\tPASS\t.\tGT:GL:PL:GP\t1/1:-2.2,-1.0,0.0:22,10,0:0.01,0.09,0.90\n\
chr2\t9\trs3\tG\tA\t70\tPASS\t.\tGT:GL:PL:GP\t0/0:0.0,-1.6,-3.2:0,16,32:0.92,0.07,0.01\n",
        sample = GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME,
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}

fn summarize_likelihood_fields(vcf_path: &Path) -> Result<LikelihoodFieldSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut fields = BTreeSet::<String>::new();
    let mut site_count = 0_u64;
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 10 {
            bail!("gl_propagation smoke row is missing FORMAT/sample fields: {line}");
        }
        site_count += 1;
        for token in parts[8].split(':') {
            if GOVERNED_RETAINED_LIKELIHOOD_FIELDS.contains(&token) {
                fields.insert(token.to_string());
            }
        }
    }
    Ok(LikelihoodFieldSummary { fields, site_count })
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfGlPropagationSmokeContract,
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
        resolve_governed_vcf_gl_propagation_smoke_contract, summarize_likelihood_fields,
        write_governed_gl_propagation_input_vcf,
    };

    #[test]
    fn governed_gl_propagation_contract_uses_single_sample_matrix_row() {
        let contract = resolve_governed_vcf_gl_propagation_smoke_contract("bcftools")
            .expect("resolve governed gl propagation contract");
        assert_eq!(contract.stage_id, "vcf.gl_propagation");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "likelihood_single_sample");
        assert_eq!(contract.sample_name, "sample_a");
    }

    #[test]
    fn governed_gl_fixture_retains_all_likelihood_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_vcf = dir.path().join("gl_input.vcf");
        write_governed_gl_propagation_input_vcf(&input_vcf).expect("write governed input");
        let summary = summarize_likelihood_fields(&input_vcf).expect("summarize fields");
        assert_eq!(summary.site_count, 3);
        assert_eq!(
            summary.fields.into_iter().collect::<Vec<_>>(),
            vec!["GL".to_string(), "GP".to_string(), "PL".to_string()]
        );
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn summarize_likelihood_fields_reads_governed_single_sample_fixture() {
        let repo_root = repo_root();
        let fixture_vcf = repo_root.join(
            "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf",
        );
        let summary = summarize_likelihood_fields(&fixture_vcf).expect("summarize fixture fields");
        assert_eq!(summary.site_count, 2);
        assert!(summary.fields.is_empty());
    }
}
