use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::params::VcfCallParams;
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_stages_vcf::pipeline::run_call_pseudohaploid_stage;
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::local_stage_result_manifest::{path_relative_to_repo, validate_stage_result_manifest};
use super::local_vcf_call_bam_smoke_support::{
    build_stage_result_manifest, materialize_reference_fasta, parse_output_sample_count,
    resolve_governed_vcf_call_bam_smoke_contract, DEFAULT_STAGE_RESULT_NAME,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_CALL_PSEUDOHAPLOID_SMOKE_ROOT: &str = "target/local-smoke/vcf.call_pseudohaploid";
const LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_pseudohaploid_smoke.v1";
const LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_pseudohaploid_smoke.metrics.v1";
const LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-call-pseudohaploid-smoke";
const GOVERNED_VCF_CALL_PSEUDOHAPLOID_STAGE_ID: &str = "vcf.call_pseudohaploid";
const DEFAULT_OUTPUT_VCF_NAME: &str = "pseudohaploid.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const GOVERNED_PSEUDOHAPLOID_SAMPLING_POLICY: &str = "deterministic_first_allele_projection";
const GOVERNED_PSEUDOHAPLOID_SEED_USAGE: &str = "control_seed_for_replay_proof";
const GOVERNED_PSEUDOHAPLOID_RANDOM_SEED: u64 = 73;
const GOVERNED_PSEUDOHAPLOID_DETERMINISM_SCOPE: &str =
    "vcf_payload_without_bcftools_command_headers";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfCallPseudohaploidSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_id: String,
    pub(crate) input_bam: String,
    pub(crate) reference: String,
    pub(crate) target_sites: u64,
    pub(crate) covered_sites: u64,
    pub(crate) called_sites: u64,
    pub(crate) missing_sites: u64,
    pub(crate) sampling_policy: &'static str,
    pub(crate) seed_usage: &'static str,
    pub(crate) random_seed: u64,
    pub(crate) sample_count: u64,
    pub(crate) tool_id: String,
    pub(crate) raw_output_sha256: String,
    pub(crate) raw_replay_output_sha256: String,
    pub(crate) canonical_output_sha256: String,
    pub(crate) canonical_replay_output_sha256: String,
    pub(crate) determinism_scope: &'static str,
    pub(crate) raw_replay_match: bool,
    pub(crate) deterministic_replay_match: bool,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfCallPseudohaploidSmokeReport {
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
    pub(crate) target_sites: u64,
    pub(crate) covered_sites: u64,
    pub(crate) called_sites: u64,
    pub(crate) missing_sites: u64,
    pub(crate) sampling_policy: &'static str,
    pub(crate) seed_usage: &'static str,
    pub(crate) random_seed: u64,
    pub(crate) sample_count: u64,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) haploid_compatible: bool,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
    pub(crate) raw_output_sha256: String,
    pub(crate) raw_replay_output_sha256: String,
    pub(crate) canonical_output_sha256: String,
    pub(crate) canonical_replay_output_sha256: String,
    pub(crate) determinism_scope: &'static str,
    pub(crate) raw_replay_match: bool,
    pub(crate) deterministic_replay_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PseudohaploidSiteSummary {
    target_sites: u64,
    covered_sites: u64,
    called_sites: u64,
    missing_sites: u64,
}

pub(crate) fn run_vcf_call_pseudohaploid_smoke(
    args: &parse::BenchLocalRunVcfCallPseudohaploidSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_call_pseudohaploid_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_call_pseudohaploid_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfCallPseudohaploidSmokeReport> {
    let contract = resolve_governed_vcf_call_bam_smoke_contract(
        repo_root,
        GOVERNED_VCF_CALL_PSEUDOHAPLOID_STAGE_ID,
        tool_id,
        "pseudohaploid_vcf",
    )?;
    let output_root =
        repo_root.join(DEFAULT_VCF_CALL_PSEUDOHAPLOID_SMOKE_ROOT).join(&contract.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    let artifacts_root = output_root.join("artifacts");
    let replay_root = artifacts_root.join("replay");
    fs::create_dir_all(&artifacts_root)
        .with_context(|| format!("create {}", artifacts_root.display()))?;

    let input_bam = repo_root.join(&contract.input_bam);
    let reference = repo_root.join(&contract.reference);
    let materialized_reference = materialize_reference_fasta(&reference, &artifacts_root)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_call_pseudohaploid_stage(
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
            "run governed VCF call_pseudohaploid smoke for `{}` from {}",
            contract.tool_id,
            input_bam.display()
        )
    })?;

    let replay_outputs = run_call_pseudohaploid_stage(
        &input_bam,
        &replay_root,
        &VcfCallParams {
            caller: contract.tool_id.clone(),
            sample_name: contract.sample_name.clone(),
            reference_fasta: Some(materialized_reference.display().to_string()),
            ..VcfCallParams::default()
        },
    )
    .with_context(|| {
        format!(
            "replay governed VCF call_pseudohaploid smoke for `{}` from {}",
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
    let pseudohaploid_summary = summarize_pseudohaploid_sites(&output_vcf)
        .with_context(|| format!("summarize pseudohaploid sites in {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(&output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed pseudohaploid smoke expects exactly one sample, found {sample_count}");
    }
    let raw_output_sha256 =
        hash_file_sha256(&output_vcf).map_err(|err| anyhow!(err.to_string()))?;
    let raw_replay_output_sha256 =
        hash_file_sha256(&replay_outputs.called_vcf).map_err(|err| anyhow!(err.to_string()))?;
    let canonical_output_sha256 = canonical_vcf_replay_sha256(&output_vcf)?;
    let canonical_replay_output_sha256 = canonical_vcf_replay_sha256(&replay_outputs.called_vcf)?;
    let raw_replay_match = raw_output_sha256 == raw_replay_output_sha256;
    let deterministic_replay_match = canonical_output_sha256 == canonical_replay_output_sha256;
    if !deterministic_replay_match {
        bail!(
            "pseudohaploid replay drifted for `{}` after canonicalization: {} != {}",
            contract.tool_id,
            canonical_output_sha256,
            canonical_replay_output_sha256
        );
    }

    let metrics = LocalVcfCallPseudohaploidSmokeMetrics {
        schema_version: LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        target_sites: pseudohaploid_summary.target_sites,
        covered_sites: pseudohaploid_summary.covered_sites,
        called_sites: pseudohaploid_summary.called_sites,
        missing_sites: pseudohaploid_summary.missing_sites,
        sampling_policy: GOVERNED_PSEUDOHAPLOID_SAMPLING_POLICY,
        seed_usage: GOVERNED_PSEUDOHAPLOID_SEED_USAGE,
        random_seed: GOVERNED_PSEUDOHAPLOID_RANDOM_SEED,
        sample_count,
        tool_id: contract.tool_id.clone(),
        raw_output_sha256: raw_output_sha256.clone(),
        raw_replay_output_sha256: raw_replay_output_sha256.clone(),
        canonical_output_sha256: canonical_output_sha256.clone(),
        canonical_replay_output_sha256: canonical_replay_output_sha256.clone(),
        determinism_scope: GOVERNED_PSEUDOHAPLOID_DETERMINISM_SCOPE,
        raw_replay_match,
        deterministic_replay_match,
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
        &format!("{LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "pseudohaploid_vcf",
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

    Ok(LocalVcfCallPseudohaploidSmokeReport {
        schema_version: LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_SCHEMA_VERSION,
        command: format!(
            "{LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_COMMAND} --tool-id {}",
            contract.tool_id
        ),
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
        target_sites: metrics.target_sites,
        covered_sites: metrics.covered_sites,
        called_sites: metrics.called_sites,
        missing_sites: metrics.missing_sites,
        sampling_policy: metrics.sampling_policy,
        seed_usage: metrics.seed_usage,
        random_seed: metrics.random_seed,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        haploid_compatible: true,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
        raw_output_sha256: metrics.raw_output_sha256,
        raw_replay_output_sha256: metrics.raw_replay_output_sha256,
        canonical_output_sha256: metrics.canonical_output_sha256,
        canonical_replay_output_sha256: metrics.canonical_replay_output_sha256,
        determinism_scope: metrics.determinism_scope,
        raw_replay_match: metrics.raw_replay_match,
        deterministic_replay_match: metrics.deterministic_replay_match,
    })
}

fn summarize_pseudohaploid_sites(vcf_path: &Path) -> Result<PseudohaploidSiteSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut summary = PseudohaploidSiteSummary {
        target_sites: 0,
        covered_sites: 0,
        called_sites: 0,
        missing_sites: 0,
    };
    let mut observed_gt = false;

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("pseudohaploid smoke output row is missing FORMAT/sample fields: {line}");
        }
        let gt_index = fields[8].split(':').position(|token| token == "GT").ok_or_else(|| {
            anyhow!("pseudohaploid smoke output row is missing GT in FORMAT: {line}")
        })?;
        observed_gt = true;
        summary.target_sites += 1;

        let sample_field = fields[9];
        let gt = sample_field.split(':').nth(gt_index).ok_or_else(|| {
            anyhow!("pseudohaploid smoke sample field is missing GT value: {line}")
        })?;
        parse_pseudohaploid_gt(gt)?;
        summary.covered_sites += 1;
        if gt == "." {
            summary.missing_sites += 1;
        } else {
            summary.called_sites += 1;
        }
    }

    if !observed_gt {
        bail!("pseudohaploid smoke output did not contain any GT fields");
    }
    Ok(summary)
}

fn parse_pseudohaploid_gt(genotype: &str) -> Result<Option<u32>> {
    if genotype == "." {
        return Ok(None);
    }
    if genotype.contains('/') || genotype.contains('|') {
        bail!("pseudohaploid smoke genotype is not haploid-compatible: `{genotype}`");
    }
    let allele = genotype.parse::<u32>().with_context(|| {
        format!("pseudohaploid smoke genotype has non-numeric allele: `{genotype}`")
    })?;
    Ok(Some(allele))
}

fn canonical_vcf_replay_sha256(vcf_path: &Path) -> Result<String> {
    let raw = read_vcf_text(vcf_path)?;
    let canonical = raw
        .lines()
        .filter(|line| {
            !line.starts_with("##bcftoolsCommand=")
                && !line.starts_with("##bcftools_callCommand=")
                && !line.starts_with("##bcftools_viewCommand=")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(sha256_hex(canonical.as_bytes()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{parse_pseudohaploid_gt, summarize_pseudohaploid_sites};

    #[test]
    fn pseudohaploid_gt_parser_accepts_governed_shapes() {
        assert_eq!(parse_pseudohaploid_gt("0").expect("ref allele"), Some(0));
        assert_eq!(parse_pseudohaploid_gt("1").expect("alt allele"), Some(1));
        assert_eq!(parse_pseudohaploid_gt(".").expect("missing allele"), None);
    }

    #[test]
    fn pseudohaploid_site_summary_counts_missing_sites() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture_vcf = dir.path().join("pseudohaploid.vcf");
        std::fs::write(
            &fixture_vcf,
            "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\tDP=8\tGT\t1\nchr1\t2\t.\tC\tT\t60\tPASS\tDP=2\tGT\t.\nchr1\t3\t.\tG\tA\t60\tPASS\tDP=5\tGT\t0\n",
        )
        .expect("write pseudohaploid fixture");

        let summary =
            summarize_pseudohaploid_sites(&fixture_vcf).expect("summarize pseudohaploid sites");
        assert_eq!(summary.target_sites, 3);
        assert_eq!(summary.covered_sites, 3);
        assert_eq!(summary.called_sites, 2);
        assert_eq!(summary.missing_sites, 1);
    }

    #[test]
    fn pseudohaploid_site_summary_rejects_diploid_genotypes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture_vcf = dir.path().join("invalid_pseudohaploid.vcf");
        std::fs::write(
            &fixture_vcf,
            "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\tDP=8\tGT\t0/1\n",
        )
        .expect("write invalid pseudohaploid fixture");

        let err = summarize_pseudohaploid_sites(&fixture_vcf).expect_err("reject diploid genotype");
        assert!(err.to_string().contains("haploid-compatible"));
    }
}
