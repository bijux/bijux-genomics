use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::params::VcfCallParams;
use bijux_dna_stages_vcf::pipeline::run_call_diploid_stage;
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;

use super::local_stage_result_manifest::{path_relative_to_repo, validate_stage_result_manifest};
use super::local_vcf_call_bam_smoke_support::{
    build_stage_result_manifest, materialize_reference_fasta, parse_output_sample_count,
    resolve_governed_vcf_call_bam_smoke_contract, DEFAULT_STAGE_RESULT_NAME,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_CALL_DIPLOID_SMOKE_ROOT: &str = "target/local-smoke/vcf.call_diploid";
const LOCAL_VCF_CALL_DIPLOID_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_diploid_smoke.v1";
const LOCAL_VCF_CALL_DIPLOID_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_diploid_smoke.metrics.v1";
const LOCAL_VCF_CALL_DIPLOID_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-call-diploid-smoke";
const GOVERNED_VCF_CALL_DIPLOID_STAGE_ID: &str = "vcf.call_diploid";
const DEFAULT_OUTPUT_VCF_NAME: &str = "diploid.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfCallDiploidSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_id: String,
    pub(crate) input_bam: String,
    pub(crate) reference: String,
    pub(crate) ploidy: &'static str,
    pub(crate) called_genotypes: u64,
    pub(crate) heterozygous_count: u64,
    pub(crate) homozygous_ref_count: u64,
    pub(crate) homozygous_alt_count: u64,
    pub(crate) missing_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfCallDiploidSmokeReport {
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
    pub(crate) ploidy: &'static str,
    pub(crate) called_genotypes: u64,
    pub(crate) heterozygous_count: u64,
    pub(crate) homozygous_ref_count: u64,
    pub(crate) homozygous_alt_count: u64,
    pub(crate) missing_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) diploid_compatible: bool,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiploidGenotypeSummary {
    called_genotypes: u64,
    heterozygous_count: u64,
    homozygous_ref_count: u64,
    homozygous_alt_count: u64,
    missing_count: u64,
}

pub(crate) fn run_vcf_call_diploid_smoke(
    args: &parse::BenchLocalRunVcfCallDiploidSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_call_diploid_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_call_diploid_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfCallDiploidSmokeReport> {
    let contract = resolve_governed_vcf_call_bam_smoke_contract(
        repo_root,
        GOVERNED_VCF_CALL_DIPLOID_STAGE_ID,
        tool_id,
        "diploid_vcf",
    )?;
    let output_root = repo_root.join(DEFAULT_VCF_CALL_DIPLOID_SMOKE_ROOT).join(&contract.tool_id);
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
            "run governed VCF call_diploid smoke for `{}` from {}",
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
    let genotype_summary = summarize_diploid_genotypes(&output_vcf)
        .with_context(|| format!("summarize diploid genotypes in {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;

    let metrics = LocalVcfCallDiploidSmokeMetrics {
        schema_version: LOCAL_VCF_CALL_DIPLOID_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        ploidy: "diploid",
        called_genotypes: genotype_summary.called_genotypes,
        heterozygous_count: genotype_summary.heterozygous_count,
        homozygous_ref_count: genotype_summary.homozygous_ref_count,
        homozygous_alt_count: genotype_summary.homozygous_alt_count,
        missing_count: genotype_summary.missing_count,
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
        &format!("{LOCAL_VCF_CALL_DIPLOID_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "diploid_vcf",
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

    Ok(LocalVcfCallDiploidSmokeReport {
        schema_version: LOCAL_VCF_CALL_DIPLOID_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_CALL_DIPLOID_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
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
        ploidy: metrics.ploidy,
        called_genotypes: metrics.called_genotypes,
        heterozygous_count: metrics.heterozygous_count,
        homozygous_ref_count: metrics.homozygous_ref_count,
        homozygous_alt_count: metrics.homozygous_alt_count,
        missing_count: metrics.missing_count,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        diploid_compatible: true,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn summarize_diploid_genotypes(vcf_path: &Path) -> Result<DiploidGenotypeSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut summary = DiploidGenotypeSummary {
        called_genotypes: 0,
        heterozygous_count: 0,
        homozygous_ref_count: 0,
        homozygous_alt_count: 0,
        missing_count: 0,
    };
    let mut observed_gt = false;

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("diploid smoke output row is missing FORMAT/sample fields: {line}");
        }
        let gt_index = fields[8]
            .split(':')
            .position(|token| token == "GT")
            .ok_or_else(|| anyhow!("diploid smoke output row is missing GT in FORMAT: {line}"))?;
        observed_gt = true;

        for sample_field in &fields[9..] {
            let gt = sample_field
                .split(':')
                .nth(gt_index)
                .ok_or_else(|| anyhow!("diploid smoke sample field is missing GT value: {line}"))?;
            let (left, right, missing) = parse_diploid_genotype(gt)?;
            if missing {
                summary.missing_count += 1;
                continue;
            }
            summary.called_genotypes += 1;
            match (left, right) {
                (0, 0) => summary.homozygous_ref_count += 1,
                (left, right) if left == right => summary.homozygous_alt_count += 1,
                _ => summary.heterozygous_count += 1,
            }
        }
    }

    if !observed_gt {
        bail!("diploid smoke output did not contain any GT fields");
    }
    Ok(summary)
}

fn parse_diploid_genotype(genotype: &str) -> Result<(u32, u32, bool)> {
    let separator = if genotype.contains('|') {
        '|'
    } else if genotype.contains('/') {
        '/'
    } else {
        bail!("diploid smoke genotype is not diploid-compatible: `{genotype}`");
    };
    let parts = genotype.split(separator).collect::<Vec<_>>();
    if parts.len() != 2 {
        bail!("diploid smoke genotype must contain exactly two alleles: `{genotype}`");
    }
    if parts.iter().all(|allele| *allele == ".") {
        return Ok((0, 0, true));
    }
    let left = parts[0].parse::<u32>().with_context(|| {
        format!("diploid smoke genotype has non-numeric left allele: `{genotype}`")
    })?;
    let right = parts[1].parse::<u32>().with_context(|| {
        format!("diploid smoke genotype has non-numeric right allele: `{genotype}`")
    })?;
    Ok((left, right, false))
}

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{parse_diploid_genotype, summarize_diploid_genotypes};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn diploid_genotype_parser_accepts_governed_shapes() {
        assert_eq!(parse_diploid_genotype("0/0").expect("hom ref"), (0, 0, false));
        assert_eq!(parse_diploid_genotype("0|1").expect("het"), (0, 1, false));
        assert_eq!(parse_diploid_genotype("1/1").expect("hom alt"), (1, 1, false));
        assert_eq!(parse_diploid_genotype("./.").expect("missing"), (0, 0, true));
    }

    #[test]
    fn diploid_genotype_summary_reads_governed_fixture() {
        let repo_root = repo_root();
        let fixture_vcf = repo_root
            .join("tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf");
        let summary =
            summarize_diploid_genotypes(&fixture_vcf).expect("summarize fixture genotypes");

        assert_eq!(summary.called_genotypes, 2);
        assert_eq!(summary.heterozygous_count, 1);
        assert_eq!(summary.homozygous_ref_count, 0);
        assert_eq!(summary.homozygous_alt_count, 1);
        assert_eq!(summary.missing_count, 0);
    }
}
