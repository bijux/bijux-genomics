use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::params::VcfCallParams;
use bijux_dna_stages_vcf::pipeline::run_call_gl_stage;
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;

use super::local_stage_result_manifest::{path_relative_to_repo, validate_stage_result_manifest};
use super::local_vcf_call_bam_smoke_support::{
    build_stage_result_manifest, materialize_reference_fasta, parse_output_sample_count,
    resolve_governed_vcf_call_bam_smoke_contract, DEFAULT_STAGE_RESULT_NAME,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_CALL_GL_SMOKE_ROOT: &str = "target/local-smoke/vcf.call_gl";
const LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_call_gl_smoke.v1";
const LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_gl_smoke.metrics.v1";
const LOCAL_VCF_CALL_GL_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-call-gl-smoke";
const GOVERNED_VCF_CALL_GL_STAGE_ID: &str = "vcf.call_gl";
const DEFAULT_OUTPUT_VCF_NAME: &str = "gl.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfCallGlSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_id: String,
    pub(crate) input_bam: String,
    pub(crate) reference: String,
    pub(crate) likelihood_field: String,
    pub(crate) sites_with_likelihoods: u64,
    pub(crate) samples_with_likelihoods: u64,
    pub(crate) missing_likelihoods: u64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfCallGlSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
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
    pub(crate) likelihood_field: String,
    pub(crate) sites_with_likelihoods: u64,
    pub(crate) samples_with_likelihoods: u64,
    pub(crate) missing_likelihoods: u64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LikelihoodSummary {
    likelihood_field: String,
    sites_with_likelihoods: u64,
    samples_with_likelihoods: u64,
    missing_likelihoods: u64,
}

pub(crate) fn run_vcf_call_gl_smoke(args: &parse::BenchLocalRunVcfCallGlSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_call_gl_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_call_gl_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfCallGlSmokeReport> {
    let contract = resolve_governed_vcf_call_bam_smoke_contract(
        repo_root,
        GOVERNED_VCF_CALL_GL_STAGE_ID,
        tool_id,
        "gl_sites_vcf",
    )?;
    let output_root = repo_root.join(DEFAULT_VCF_CALL_GL_SMOKE_ROOT).join(&contract.tool_id);
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
    let stage_outputs = run_call_gl_stage(
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
            "run governed VCF call_gl smoke for `{}` from {}",
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
        VcfFieldRequirement { require_gt: false, require_gl: true },
    )
    .with_context(|| format!("validate {}", output_vcf.display()))?;
    let likelihood_summary = summarize_likelihood_fields(&output_vcf)
        .with_context(|| format!("summarize likelihood fields in {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;

    let metrics = LocalVcfCallGlSmokeMetrics {
        schema_version: LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        likelihood_field: likelihood_summary.likelihood_field.clone(),
        sites_with_likelihoods: likelihood_summary.sites_with_likelihoods,
        samples_with_likelihoods: likelihood_summary.samples_with_likelihoods,
        missing_likelihoods: likelihood_summary.missing_likelihoods,
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
        &format!("{LOCAL_VCF_CALL_GL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "gl_sites_vcf",
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

    Ok(LocalVcfCallGlSmokeReport {
        schema_version: LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_CALL_GL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
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
        likelihood_field: metrics.likelihood_field,
        sites_with_likelihoods: metrics.sites_with_likelihoods,
        samples_with_likelihoods: metrics.samples_with_likelihoods,
        missing_likelihoods: metrics.missing_likelihoods,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn summarize_likelihood_fields(vcf_path: &Path) -> Result<LikelihoodSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut likelihood_field = None::<String>;
    let mut sites_with_likelihoods = 0_u64;
    let mut missing_likelihoods = 0_u64;
    let mut samples_with_likelihoods = BTreeSet::<usize>::new();

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("GL smoke output row is missing FORMAT/sample fields: {line}");
        }
        let format_tokens = fields[8].split(':').collect::<Vec<_>>();
        let field_name = ["GL", "GP", "PL"]
            .into_iter()
            .find(|candidate| format_tokens.iter().any(|token| token == candidate))
            .ok_or_else(|| anyhow!("GL smoke output row is missing GL/GP/PL in FORMAT: {line}"))?;
        let field_index = format_tokens
            .iter()
            .position(|token| *token == field_name)
            .ok_or_else(|| anyhow!("GL smoke output row lost {field_name} in FORMAT: {line}"))?;

        if let Some(previous) = &likelihood_field {
            if previous != field_name {
                bail!(
                    "GL smoke likelihood field drifted across rows: `{previous}` vs `{field_name}`"
                );
            }
        } else {
            likelihood_field = Some(field_name.to_string());
        }

        let mut row_has_likelihood = false;
        for (sample_index, sample_field) in fields[9..].iter().enumerate() {
            let sample_value = sample_field.split(':').nth(field_index).ok_or_else(|| {
                anyhow!("GL smoke sample field is missing {field_name} value: {line}")
            })?;
            if likelihood_value_is_missing(sample_value) {
                missing_likelihoods += 1;
                continue;
            }
            row_has_likelihood = true;
            samples_with_likelihoods.insert(sample_index);
        }
        if row_has_likelihood {
            sites_with_likelihoods += 1;
        }
    }

    Ok(LikelihoodSummary {
        likelihood_field: likelihood_field
            .ok_or_else(|| anyhow!("GL smoke output did not contain any GL/GP/PL fields"))?,
        sites_with_likelihoods,
        samples_with_likelihoods: u64::try_from(samples_with_likelihoods.len())
            .map_err(|_| anyhow!("GL smoke sample-with-likelihood count overflowed u64"))?,
        missing_likelihoods,
    })
}

fn likelihood_value_is_missing(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.split(',').all(|token| matches!(token.trim(), "." | ""))
}

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{likelihood_value_is_missing, summarize_likelihood_fields};

    #[test]
    fn likelihood_value_missing_helper_handles_gl_shapes() {
        assert!(likelihood_value_is_missing(".,.,."));
        assert!(likelihood_value_is_missing("."));
        assert!(!likelihood_value_is_missing("0,12,34"));
        assert!(!likelihood_value_is_missing("0.0,-1.0,-2.0"));
    }

    #[test]
    fn likelihood_summary_counts_present_and_missing_gl_values() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture_vcf = dir.path().join("gl_fixture.vcf");
        std::fs::write(
            &fixture_vcf,
            "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Likelihoods\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT:PL\t0/1:0,10,20\t0/0:.,.,.\nchr1\t2\t.\tC\tT\t60\tPASS\t.\tGT:PL\t1/1:5,6,0\t0/1:9,0,12\n",
        )
        .expect("write GL fixture");

        let summary = summarize_likelihood_fields(&fixture_vcf).expect("summarize GL values");
        assert_eq!(summary.likelihood_field, "PL");
        assert_eq!(summary.sites_with_likelihoods, 2);
        assert_eq!(summary.samples_with_likelihoods, 2);
        assert_eq!(summary.missing_likelihoods, 1);
    }

    #[test]
    fn likelihood_summary_rejects_hard_genotype_only_rows() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture_vcf = dir.path().join("gt_only.vcf");
        std::fs::write(
            &fixture_vcf,
            "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
        )
        .expect("write GT-only fixture");

        let err = summarize_likelihood_fields(&fixture_vcf).expect_err("reject GT-only rows");
        assert!(err.to_string().contains("GL/GP/PL"));
    }
}
