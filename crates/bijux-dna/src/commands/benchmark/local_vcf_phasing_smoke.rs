use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_stages_vcf::pipeline::{
    assert_bgzip_tabix_artifacts, run_phasing_stage, PhasingBackend, PhasingStageParams,
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

const DEFAULT_VCF_PHASING_SMOKE_ROOT: &str = "runs/bench/local-smoke/vcf.phasing";
const LOCAL_VCF_PHASING_SMOKE_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_phasing_smoke.v1";
const LOCAL_VCF_PHASING_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_phasing_smoke.metrics.v1";
const LOCAL_VCF_PHASING_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-phasing-smoke";
const GOVERNED_VCF_PHASING_STAGE_ID: &str = "vcf.phasing";
const GOVERNED_VCF_PHASING_TOOL_ID: &str = "shapeit5";
const GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID: &str = "cohort_unphased_two_sample";
const DEFAULT_INPUT_VCF_NAME: &str = "phasing_input.vcf";
const DEFAULT_OUTPUT_VCF_NAME: &str = "phased.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_OUTPUT_PANEL_ASSETS_NAME: &str = "panel_assets.json";
const DEFAULT_OUTPUT_QC_NAME: &str = "phasing_qc.json";
const DEFAULT_OUTPUT_MANIFEST_NAME: &str = "phasing_manifest.json";
const DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME: &str = "phase_block_stats.tsv";
const DEFAULT_OUTPUT_SWITCH_PROXY_NAME: &str = "switch_error_proxy.tsv";
const DEFAULT_OUTPUT_LOGS_NAME: &str = "logs.txt";
const EXPECTED_SAMPLE_IDS: [&str; 2] = ["cohort_alpha", "cohort_beta"];
const EXPECTED_INPUT_GENOTYPES: u64 = 8;
const EXPECTED_PHASED_GENOTYPES: u64 = 8;
const EXPECTED_UNPHASED_GENOTYPES: u64 = 0;
const EXPECTED_PHASE_SET_COUNT: u64 = 2;
const GOVERNED_SHAPEIT5_REGION: &str = "1:1-1000000";
const GOVERNED_SHAPEIT5_SEED: u64 = 37;
const GOVERNED_SHAPEIT5_THREADS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfPhasingSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) input_genotypes: u64,
    pub(crate) phased_genotypes: u64,
    pub(crate) unphased_genotypes: u64,
    pub(crate) phase_set_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPhasingSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) input_vcf_path: String,
    pub(crate) output_root: String,
    pub(crate) output_vcf_path: String,
    pub(crate) output_tbi_path: String,
    pub(crate) panel_assets_path: String,
    pub(crate) phasing_qc_path: String,
    pub(crate) phasing_manifest_path: String,
    pub(crate) phase_block_stats_path: String,
    pub(crate) switch_error_proxy_path: String,
    pub(crate) logs_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) input_genotypes: u64,
    pub(crate) phased_genotypes: u64,
    pub(crate) unphased_genotypes: u64,
    pub(crate) phase_set_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhaseSummary {
    genotype_count: u64,
    phased_genotypes: u64,
    unphased_genotypes: u64,
    phase_set_count: u64,
    sample_ids: Vec<String>,
}

pub(crate) fn run_vcf_phasing_smoke(args: &parse::BenchLocalRunVcfPhasingSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_phasing_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_vcf_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_phasing_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPhasingSmokeReport> {
    let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
        GOVERNED_VCF_PHASING_STAGE_ID,
        tool_id,
        "phased_vcf",
    )?;
    let output_root = repo_root.join(DEFAULT_VCF_PHASING_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_phasing_input_vcf(&input_vcf_path)?;
    let panel_assets_report = materialize_governed_vcf_panel_assets(&input_root.join("reference"))
        .with_context(|| {
            format!("materialize governed VCF panel assets under {}", input_root.display())
        })?;
    let panel_assets_path = output_root.join(DEFAULT_OUTPUT_PANEL_ASSETS_NAME);
    bijux_dna_infra::atomic_write_json(&panel_assets_path, &panel_assets_report)?;

    let species_context = governed_vcf_panel_species_context();
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_phasing_stage(
        &input_vcf_path,
        &stage_root,
        &species_context,
        &PhasingStageParams {
            species_id: species_context.species_id.clone(),
            build_id: species_context.build_id.clone(),
            backend: PhasingBackend::Shapeit5,
            map_id: Some(contract.map_id.clone()),
            threads: GOVERNED_SHAPEIT5_THREADS,
            seed: GOVERNED_SHAPEIT5_SEED,
            region: Some(GOVERNED_SHAPEIT5_REGION.to_string()),
            allow_gl_only_input: false,
        },
    )
    .with_context(|| format!("run governed VCF phasing smoke from {}", input_vcf_path.display()))?;

    let output_vcf_path = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.phased_vcf, &output_vcf_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.phased_vcf.display(), output_vcf_path.display())
    })?;
    let output_tbi_path = PathBuf::from(format!("{}.tbi", output_vcf_path.display()));
    fs::copy(&stage_outputs.phased_tbi, &output_tbi_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.phased_tbi.display(), output_tbi_path.display())
    })?;
    let phasing_qc_path = output_root.join(DEFAULT_OUTPUT_QC_NAME);
    fs::copy(&stage_outputs.phasing_qc_json, &phasing_qc_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.phasing_qc_json.display(), phasing_qc_path.display())
    })?;
    let phasing_manifest_path = output_root.join(DEFAULT_OUTPUT_MANIFEST_NAME);
    fs::copy(&stage_outputs.phasing_manifest_json, &phasing_manifest_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.phasing_manifest_json.display(),
            phasing_manifest_path.display()
        )
    })?;
    let phase_block_stats_path = output_root.join(DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME);
    fs::copy(&stage_outputs.phase_block_stats_tsv, &phase_block_stats_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.phase_block_stats_tsv.display(),
            phase_block_stats_path.display()
        )
    })?;
    let switch_error_proxy_path = output_root.join(DEFAULT_OUTPUT_SWITCH_PROXY_NAME);
    fs::copy(&stage_outputs.switch_error_proxy_tsv, &switch_error_proxy_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.switch_error_proxy_tsv.display(),
                switch_error_proxy_path.display()
            )
        },
    )?;
    let logs_path = output_root.join(DEFAULT_OUTPUT_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), logs_path.display())
    })?;

    assert_bgzip_tabix_artifacts(&output_vcf_path, &output_tbi_path)?;
    let validation = vcf_validate_input(
        &output_vcf_path,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", output_vcf_path.display()))?;
    let input_summary = summarize_phasing_genotypes(&input_vcf_path)?;
    let output_summary = summarize_phasing_genotypes(&output_vcf_path)?;

    if input_summary.genotype_count != EXPECTED_INPUT_GENOTYPES {
        bail!(
            "governed VCF phasing smoke expected {} input genotypes, found {}",
            EXPECTED_INPUT_GENOTYPES,
            input_summary.genotype_count
        );
    }
    if output_summary.phased_genotypes != EXPECTED_PHASED_GENOTYPES {
        bail!(
            "governed VCF phasing smoke expected {} phased genotypes, found {}",
            EXPECTED_PHASED_GENOTYPES,
            output_summary.phased_genotypes
        );
    }
    if output_summary.unphased_genotypes != EXPECTED_UNPHASED_GENOTYPES {
        bail!(
            "governed VCF phasing smoke expected {} unphased genotypes after phasing, found {}",
            EXPECTED_UNPHASED_GENOTYPES,
            output_summary.unphased_genotypes
        );
    }
    if output_summary.phase_set_count != EXPECTED_PHASE_SET_COUNT {
        bail!(
            "governed VCF phasing smoke expected {} phase sets, found {}",
            EXPECTED_PHASE_SET_COUNT,
            output_summary.phase_set_count
        );
    }
    if output_summary.sample_ids
        != EXPECTED_SAMPLE_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>()
    {
        bail!(
            "governed VCF phasing smoke expected sample ids {:?}, found {:?}",
            EXPECTED_SAMPLE_IDS,
            output_summary.sample_ids
        );
    }

    let phasing_qc = read_json(&phasing_qc_path)?;
    if phasing_qc.get("backend").and_then(serde_json::Value::as_str) != Some("shapeit5") {
        bail!("phasing QC report drifted away from shapeit5 backend");
    }
    let phasing_manifest = read_json(&phasing_manifest_path)?;
    if phasing_manifest.get("backend").and_then(serde_json::Value::as_str) != Some("shapeit5") {
        bail!("phasing manifest drifted away from shapeit5 backend");
    }
    if phasing_manifest.pointer("/map/map_id").and_then(serde_json::Value::as_str)
        != Some(contract.map_id.as_str())
    {
        bail!("phasing manifest map identity drifted from governed map contract");
    }

    let sample_count = u64::try_from(output_summary.sample_ids.len())
        .map_err(|_| anyhow!("sample count overflow"))?;
    let metrics = LocalVcfPhasingSmokeMetrics {
        schema_version: LOCAL_VCF_PHASING_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel_id.clone(),
        map_id: contract.map_id.clone(),
        input_genotypes: input_summary.genotype_count,
        phased_genotypes: output_summary.phased_genotypes,
        unphased_genotypes: output_summary.unphased_genotypes,
        phase_set_count: output_summary.phase_set_count,
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
        &format!("{LOCAL_VCF_PHASING_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "phased_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf_path.as_path(),
                "vcf_output",
            ),
            (
                "phased_tbi",
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
                "phasing_qc_json",
                DEFAULT_OUTPUT_QC_NAME.to_string(),
                phasing_qc_path.as_path(),
                "report_output",
            ),
            (
                "phasing_manifest_json",
                DEFAULT_OUTPUT_MANIFEST_NAME.to_string(),
                phasing_manifest_path.as_path(),
                "report_output",
            ),
            (
                "phase_block_stats_tsv",
                DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME.to_string(),
                phase_block_stats_path.as_path(),
                "table_output",
            ),
            (
                "switch_error_proxy_tsv",
                DEFAULT_OUTPUT_SWITCH_PROXY_NAME.to_string(),
                switch_error_proxy_path.as_path(),
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

    Ok(LocalVcfPhasingSmokeReport {
        schema_version: LOCAL_VCF_PHASING_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_PHASING_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel_id,
        map_id: contract.map_id,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        output_root: path_relative_to_repo(repo_root, &output_root),
        output_vcf_path: path_relative_to_repo(repo_root, &output_vcf_path),
        output_tbi_path: path_relative_to_repo(repo_root, &output_tbi_path),
        panel_assets_path: path_relative_to_repo(repo_root, &panel_assets_path),
        phasing_qc_path: path_relative_to_repo(repo_root, &phasing_qc_path),
        phasing_manifest_path: path_relative_to_repo(repo_root, &phasing_manifest_path),
        phase_block_stats_path: path_relative_to_repo(repo_root, &phase_block_stats_path),
        switch_error_proxy_path: path_relative_to_repo(repo_root, &switch_error_proxy_path),
        logs_path: path_relative_to_repo(repo_root, &logs_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: 0,
        input_genotypes: input_summary.genotype_count,
        phased_genotypes: output_summary.phased_genotypes,
        unphased_genotypes: output_summary.unphased_genotypes,
        phase_set_count: output_summary.phase_set_count,
        sample_count,
        sample_ids: output_summary.sample_ids,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

fn write_governed_phasing_input_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF phasing smoke input path has no parent directory"))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##reference=GRCh38\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tcohort_alpha\tcohort_beta\n\
1\t101\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t1/0\n\
1\t120\t.\tC\tT\t61\tPASS\t.\tGT\t0/1\t0/1\n\
1\t145\t.\tG\tA\t59\tPASS\t.\tGT\t1/0\t0/1\n\
1\t160\t.\tT\tC\t58\tPASS\t.\tGT\t0/1\t1/0\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

fn summarize_phasing_genotypes(vcf_path: &Path) -> Result<PhaseSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("phasing VCF is missing the #CHROM header"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    let mut in_phase_block = vec![false; sample_ids.len()];
    let mut genotype_count = 0u64;
    let mut phased_genotypes = 0u64;
    let mut unphased_genotypes = 0u64;
    let mut phase_set_count = 0u64;

    for line in raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        for (sample_index, sample_field) in fields.iter().skip(9).enumerate() {
            let gt = sample_field.split(':').next().unwrap_or_default();
            if gt.is_empty() || gt == "." || gt == "./." || gt == ".|." {
                in_phase_block[sample_index] = false;
                continue;
            }
            genotype_count += 1;
            if gt.contains('|') {
                phased_genotypes += 1;
                if !in_phase_block[sample_index] {
                    phase_set_count += 1;
                    in_phase_block[sample_index] = true;
                }
            } else if gt.contains('/') {
                unphased_genotypes += 1;
                in_phase_block[sample_index] = false;
            } else {
                in_phase_block[sample_index] = false;
            }
        }
    }

    Ok(PhaseSummary {
        genotype_count,
        phased_genotypes,
        unphased_genotypes,
        phase_set_count,
        sample_ids,
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{
        run_local_vcf_phasing_smoke, summarize_phasing_genotypes, write_governed_phasing_input_vcf,
    };

    #[test]
    fn governed_phasing_fixture_tracks_unphased_input_counts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_path = dir.path().join("input.vcf");
        write_governed_phasing_input_vcf(&input_path).expect("write fixture");
        let summary = summarize_phasing_genotypes(&input_path).expect("summarize input");
        assert_eq!(summary.genotype_count, 8);
        assert_eq!(summary.phased_genotypes, 0);
        assert_eq!(summary.unphased_genotypes, 8);
        assert_eq!(summary.phase_set_count, 0);
    }

    #[test]
    fn governed_vcf_phasing_smoke_reports_fully_phased_output() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        let report = run_local_vcf_phasing_smoke(repo_root.path(), "shapeit5")
            .expect("run local phasing smoke");
        assert_eq!(report.stage_id, "vcf.phasing");
        assert_eq!(report.tool_id, "shapeit5");
        assert_eq!(report.input_genotypes, 8);
        assert_eq!(report.phased_genotypes, 8);
        assert_eq!(report.unphased_genotypes, 0);
        assert_eq!(report.phase_set_count, 2);
        assert_eq!(report.sample_ids, vec!["cohort_alpha".to_string(), "cohort_beta".to_string()]);
        assert!(report.parseable);
    }
}
