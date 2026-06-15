use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_stages_vcf::pipeline::{
    assert_bgzip_tabix_artifacts, run_prepare_reference_panel_stage, PrepareReferencePanelParams,
};
use bijux_dna_stages_vcf::vcf_io::{read_vcf_text, vcf_validate_input, VcfFieldRequirement};
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

const DEFAULT_VCF_PREPARE_PANEL_SMOKE_ROOT: &str =
    "runs/bench/local-smoke/vcf.prepare_reference_panel";
const LOCAL_VCF_PREPARE_PANEL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_prepare_reference_panel_smoke.v1";
const LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_prepare_reference_panel_smoke.metrics.v1";
const LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-prepare-reference-panel-smoke";
const GOVERNED_VCF_PREPARE_PANEL_STAGE_ID: &str = "vcf.prepare_reference_panel";
const GOVERNED_VCF_PREPARE_PANEL_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID: &str = "vcf_production_regression";
const GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID: &str = "vcf_reference_panel";
const GOVERNED_VCF_PREPARE_PANEL_PANEL_ID: &str = "hsapiens_grch38_mini";
const GOVERNED_VCF_PREPARE_PANEL_MAP_ID: &str = "hsapiens_grch38_chr_map";
const GOVERNED_VCF_PREPARE_PANEL_INPUT_FIXTURE_ID: &str = "reference_panel_sort_and_deduplicate";
const DEFAULT_INPUT_VCF_NAME: &str = "prepare_reference_panel_input.vcf";
const DEFAULT_RAW_PANEL_NAME: &str = "panel.vcf.gz";
const DEFAULT_OUTPUT_PANEL_NAME: &str = "panel.vcf.gz";
const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
const DEFAULT_PANEL_MANIFEST_NAME: &str = "panel_manifest.json";
const DEFAULT_OVERLAP_NAME: &str = "overlap.json";
const DEFAULT_PANEL_OVERLAP_NAME: &str = "panel_overlap.json";
const DEFAULT_PANEL_FILES_NAME: &str = "panel_files.json";
const DEFAULT_OVERLAP_TSV_NAME: &str = "overlap.tsv";
const DEFAULT_CHUNKS_NAME: &str = "chunks.json";
const EXPECTED_SAMPLE_ID: &str = "sample1";
const EXPECTED_INPUT_VARIANT_COUNT: u64 = 5;
const EXPECTED_OUTPUT_VARIANT_COUNT: u64 = 4;
const EXPECTED_DUPLICATE_SITES_REMOVED: u64 = 1;
const EXPECTED_NORMALIZATION_STATUS: &str = "sorted_indexed_deduplicated";

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedVcfPrepareReferencePanelSmokeContract {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    input_fixture_id: String,
    panel_id: String,
    map_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPrepareReferencePanelSmokeMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) input_variants: u64,
    pub(crate) output_variants: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) duplicate_sites_removed: u64,
    pub(crate) normalization_status: String,
    pub(crate) index_path: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) tool_id: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfPrepareReferencePanelSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) input_vcf_path: String,
    pub(crate) raw_panel_path: String,
    pub(crate) output_root: String,
    pub(crate) panel_vcf_path: String,
    pub(crate) panel_tbi_path: String,
    pub(crate) panel_manifest_path: String,
    pub(crate) overlap_path: String,
    pub(crate) panel_overlap_path: String,
    pub(crate) panel_files_path: String,
    pub(crate) overlap_tsv_path: String,
    pub(crate) chunks_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) input_variants: u64,
    pub(crate) output_variants: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) sample_consistent: bool,
    pub(crate) duplicate_sites_removed: u64,
    pub(crate) normalization_status: String,
    pub(crate) index_path: String,
    pub(crate) parseable: bool,
    pub(crate) validation_checks: BTreeMap<String, bool>,
    pub(crate) gt_present: bool,
    pub(crate) gl_present: bool,
}

pub(crate) fn run_local_vcf_prepare_reference_panel_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPrepareReferencePanelSmokeReport> {
    let contract = resolve_governed_vcf_prepare_reference_panel_smoke_contract(tool_id)?;
    let output_root = repo_root.join(DEFAULT_VCF_PREPARE_PANEL_SMOKE_ROOT).join(&contract.tool_id);
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
    write_governed_prepare_reference_panel_input_vcf(&input_vcf_path)?;
    let raw_panel_path = materialize_governed_prepare_reference_panel_raw_fixture(&input_root)?;
    let species_context = governed_species_context();

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_prepare_reference_panel_stage(
        &input_vcf_path,
        &raw_panel_path,
        &stage_root,
        &species_context,
        &PrepareReferencePanelParams {
            species_id: species_context.species_id.clone(),
            build_id: species_context.build_id.clone(),
            panel_id: Some(contract.panel_id.clone()),
            map_id: Some(contract.map_id.clone()),
        },
    )
    .with_context(|| {
        format!("run governed VCF panel preparation smoke from {}", raw_panel_path.display())
    })?;

    let panel_vcf_path = output_root.join(DEFAULT_OUTPUT_PANEL_NAME);
    fs::copy(&stage_outputs.prepared_panel_vcf, &panel_vcf_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.prepared_panel_vcf.display(),
            panel_vcf_path.display()
        )
    })?;
    let panel_tbi_path = PathBuf::from(format!("{}.tbi", panel_vcf_path.display()));
    fs::copy(&stage_outputs.prepared_panel_tbi, &panel_tbi_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.prepared_panel_tbi.display(),
            panel_tbi_path.display()
        )
    })?;
    let panel_manifest_path = output_root.join(DEFAULT_PANEL_MANIFEST_NAME);
    fs::copy(&stage_outputs.panel_manifest_json, &panel_manifest_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.panel_manifest_json.display(),
            panel_manifest_path.display()
        )
    })?;
    let overlap_path = output_root.join(DEFAULT_OVERLAP_NAME);
    fs::copy(&stage_outputs.overlap_json, &overlap_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.overlap_json.display(), overlap_path.display())
    })?;
    let panel_overlap_path = output_root.join(DEFAULT_PANEL_OVERLAP_NAME);
    fs::copy(&stage_outputs.panel_overlap_json, &panel_overlap_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.panel_overlap_json.display(),
            panel_overlap_path.display()
        )
    })?;
    let panel_files_path = output_root.join(DEFAULT_PANEL_FILES_NAME);
    fs::copy(&stage_outputs.panel_files_json, &panel_files_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.panel_files_json.display(),
            panel_files_path.display()
        )
    })?;
    let overlap_tsv_path = output_root.join(DEFAULT_OVERLAP_TSV_NAME);
    fs::copy(&stage_outputs.overlap_tsv, &overlap_tsv_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.overlap_tsv.display(), overlap_tsv_path.display())
    })?;
    let chunks_path = output_root.join(DEFAULT_CHUNKS_NAME);
    fs::copy(&stage_outputs.chunks_json, &chunks_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.chunks_json.display(), chunks_path.display())
    })?;

    assert_bgzip_tabix_artifacts(&panel_vcf_path, &panel_tbi_path)?;
    let validation = vcf_validate_input(
        &panel_vcf_path,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", panel_vcf_path.display()))?;
    let sample_ids = parse_output_sample_ids(&panel_vcf_path)?;
    let sample_count =
        u64::try_from(sample_ids.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let input_variants = parse_record_count(&raw_panel_path)?;
    let output_variants = parse_record_count(&panel_vcf_path)?;
    let panel_manifest = read_json(&panel_manifest_path)?;
    let duplicate_sites_removed = panel_manifest
        .pointer("/normalization/duplicate_sites_removed")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            anyhow!("panel manifest is missing normalization.duplicate_sites_removed")
        })?;
    let normalization_status = panel_manifest
        .pointer("/normalization/status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.status"))?
        .to_string();
    let manifest_input_variants = panel_manifest
        .pointer("/normalization/input_variant_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.input_variant_count"))?;
    let manifest_output_variants = panel_manifest
        .pointer("/normalization/output_variant_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.output_variant_count"))?;
    let manifest_sample_count = panel_manifest
        .pointer("/normalization/sample_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.sample_count"))?;
    let manifest_sample_ids = panel_manifest
        .pointer("/normalization/sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.sample_ids"))?
        .iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    if input_variants != EXPECTED_INPUT_VARIANT_COUNT {
        bail!(
            "governed VCF prepare_reference_panel smoke expected {EXPECTED_INPUT_VARIANT_COUNT} raw variants, found {input_variants}"
        );
    }
    if output_variants != EXPECTED_OUTPUT_VARIANT_COUNT {
        bail!(
            "governed VCF prepare_reference_panel smoke expected {EXPECTED_OUTPUT_VARIANT_COUNT} output variants, found {output_variants}"
        );
    }
    if duplicate_sites_removed != EXPECTED_DUPLICATE_SITES_REMOVED {
        bail!(
            "governed VCF prepare_reference_panel smoke expected {EXPECTED_DUPLICATE_SITES_REMOVED} duplicate sites removed, found {duplicate_sites_removed}"
        );
    }
    if normalization_status != EXPECTED_NORMALIZATION_STATUS {
        bail!(
            "governed VCF prepare_reference_panel smoke expected normalization status `{EXPECTED_NORMALIZATION_STATUS}`, found `{normalization_status}`"
        );
    }
    if input_variants.saturating_sub(output_variants) != duplicate_sites_removed {
        bail!(
            "governed VCF prepare_reference_panel smoke duplicate count drifted: input={input_variants} output={output_variants} removed={duplicate_sites_removed}"
        );
    }
    if manifest_input_variants != input_variants || manifest_output_variants != output_variants {
        bail!(
            "panel manifest normalization counts drifted from observed counts: manifest=({manifest_input_variants},{manifest_output_variants}) observed=({input_variants},{output_variants})"
        );
    }
    if manifest_sample_count != sample_count {
        bail!(
            "panel manifest sample count drifted from observed output: manifest={manifest_sample_count} observed={sample_count}"
        );
    }
    let sample_consistent =
        sample_ids == vec![EXPECTED_SAMPLE_ID.to_string()] && manifest_sample_ids == sample_ids;
    if !sample_consistent {
        bail!(
            "governed VCF prepare_reference_panel smoke expected sample ids {:?}, found output {:?} and manifest {:?}",
            vec![EXPECTED_SAMPLE_ID.to_string()],
            sample_ids,
            manifest_sample_ids
        );
    }

    let metrics = LocalVcfPrepareReferencePanelSmokeMetrics {
        schema_version: LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_variants,
        output_variants,
        sample_count,
        sample_ids: sample_ids.clone(),
        duplicate_sites_removed,
        normalization_status: normalization_status.clone(),
        index_path: path_relative_to_repo(repo_root, &panel_tbi_path),
        panel_id: contract.panel_id.clone(),
        map_id: contract.map_id.clone(),
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
        &format!("{LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "prepared_panel_vcf",
                DEFAULT_OUTPUT_PANEL_NAME.to_string(),
                panel_vcf_path.as_path(),
                "vcf_output",
            ),
            (
                "prepared_panel_tbi",
                format!("{DEFAULT_OUTPUT_PANEL_NAME}.tbi"),
                panel_tbi_path.as_path(),
                "index_output",
            ),
            (
                "panel_manifest_json",
                DEFAULT_PANEL_MANIFEST_NAME.to_string(),
                panel_manifest_path.as_path(),
                "report_output",
            ),
            (
                "overlap_json",
                DEFAULT_OVERLAP_NAME.to_string(),
                overlap_path.as_path(),
                "report_output",
            ),
            (
                "panel_overlap_json",
                DEFAULT_PANEL_OVERLAP_NAME.to_string(),
                panel_overlap_path.as_path(),
                "report_output",
            ),
            (
                "panel_files_json",
                DEFAULT_PANEL_FILES_NAME.to_string(),
                panel_files_path.as_path(),
                "report_output",
            ),
            (
                "overlap_tsv",
                DEFAULT_OVERLAP_TSV_NAME.to_string(),
                overlap_tsv_path.as_path(),
                "table_output",
            ),
            (
                "chunks_json",
                DEFAULT_CHUNKS_NAME.to_string(),
                chunks_path.as_path(),
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

    Ok(LocalVcfPrepareReferencePanelSmokeReport {
        schema_version: LOCAL_VCF_PREPARE_PANEL_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id,
        tool_id: contract.tool_id,
        corpus_id: contract.corpus_id,
        input_fixture_id: contract.input_fixture_id,
        panel_id: contract.panel_id,
        map_id: contract.map_id,
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        raw_panel_path: path_relative_to_repo(repo_root, &raw_panel_path),
        output_root: path_relative_to_repo(repo_root, &output_root),
        panel_vcf_path: path_relative_to_repo(repo_root, &panel_vcf_path),
        panel_tbi_path: path_relative_to_repo(repo_root, &panel_tbi_path),
        panel_manifest_path: path_relative_to_repo(repo_root, &panel_manifest_path),
        overlap_path: path_relative_to_repo(repo_root, &overlap_path),
        panel_overlap_path: path_relative_to_repo(repo_root, &panel_overlap_path),
        panel_files_path: path_relative_to_repo(repo_root, &panel_files_path),
        overlap_tsv_path: path_relative_to_repo(repo_root, &overlap_tsv_path),
        chunks_path: path_relative_to_repo(repo_root, &chunks_path),
        metrics_path: path_relative_to_repo(repo_root, &metrics_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: 0,
        input_variants,
        output_variants,
        sample_count,
        sample_ids,
        sample_consistent,
        duplicate_sites_removed,
        normalization_status,
        index_path: path_relative_to_repo(repo_root, &panel_tbi_path),
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    })
}

pub(crate) fn run_vcf_prepare_reference_panel_smoke(
    args: &parse::BenchLocalRunVcfPrepareReferencePanelSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_prepare_reference_panel_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.panel_vcf_path);
    }
    Ok(())
}

fn resolve_governed_vcf_prepare_reference_panel_smoke_contract(
    requested_tool_id: &str,
) -> Result<GovernedVcfPrepareReferencePanelSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == GOVERNED_VCF_PREPARE_PANEL_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_PREPARE_PANEL_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_PREPARE_PANEL_TOOL_ID {
        bail!(
            "VCF prepare_reference_panel smoke requires retained tool `{GOVERNED_VCF_PREPARE_PANEL_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF prepare_reference_panel smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID {
        bail!(
            "VCF prepare_reference_panel smoke requires corpus `{GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID {
        bail!(
            "VCF prepare_reference_panel smoke requires asset profile `{GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["prepared_panel".to_string(), "chunks_json".to_string()]
    {
        bail!(
            "VCF prepare_reference_panel smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPrepareReferencePanelSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_PREPARE_PANEL_INPUT_FIXTURE_ID.to_string(),
        panel_id: GOVERNED_VCF_PREPARE_PANEL_PANEL_ID.to_string(),
        map_id: GOVERNED_VCF_PREPARE_PANEL_MAP_ID.to_string(),
    })
}

fn governed_species_context() -> SpeciesContext {
    SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
            .to_string(),
        contigs: vec![
            ContigSpec { name: "1".to_string(), length_bp: 248_956_422 },
            ContigSpec { name: "2".to_string(), length_bp: 242_193_529 },
            ContigSpec { name: "chr1".to_string(), length_bp: 248_956_422 },
            ContigSpec { name: "chr2".to_string(), length_bp: 242_193_529 },
        ],
        sex_system: "xy".to_string(),
        par_policy: "grch38_par".to_string(),
        default_coverage_regime: None,
    }
}

fn write_governed_prepare_reference_panel_input_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF panel smoke input path has no parent directory"))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\n\
1\t101\t.\tA\tG\t60\tPASS\tDP=8\tGT\t0/1\n\
1\t105\t.\tC\tT\t42\tPASS\tDP=13\tGT\t0/1\n\
1\t111\t.\tG\tGA\t12\tLOWQUAL\tDP=5\tGT\t0/1\n\
1\t140\t.\tT\tC\t85\tPASS\tDP=31\tGT\t0/1\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

fn materialize_governed_prepare_reference_panel_raw_fixture(input_root: &Path) -> Result<PathBuf> {
    let panel_root = input_root
        .join("panel_store")
        .join(GOVERNED_VCF_PREPARE_PANEL_PANEL_ID)
        .join("local-reference-panel");
    let panel_raw = panel_root.join("raw");
    let panel_normalized = panel_root.join("normalized");
    let panel_derived = panel_root.join("derived");
    fs::create_dir_all(&panel_raw).with_context(|| format!("create {}", panel_raw.display()))?;
    fs::create_dir_all(&panel_normalized)
        .with_context(|| format!("create {}", panel_normalized.display()))?;
    fs::create_dir_all(&panel_derived)
        .with_context(|| format!("create {}", panel_derived.display()))?;
    let raw_panel_path = panel_raw.join(DEFAULT_RAW_PANEL_NAME);
    let payload = "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\n\
1\t140\t.\tT\tC\t85\tPASS\tDP=31\tGT\t0/1\n\
1\t101\t.\tA\tG\t60\tPASS\tDP=8\tGT\t0/1\n\
1\t105\t.\tC\tT\t42\tPASS\tDP=13\tGT\t0/1\n\
1\t111\t.\tG\tGA\t12\tLOWQUAL\tDP=5\tGT\t0/1\n\
1\t105\t.\tC\tT\t42\tPASS\tDP=13\tGT\t0/1\n";
    bijux_dna_infra::atomic_write_bytes(&raw_panel_path, payload.as_bytes())?;
    Ok(raw_panel_path)
}

fn parse_output_sample_ids(vcf_path: &Path) -> Result<Vec<String>> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("prepared panel output is missing the #CHROM header"))?;
    Ok(sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
}

fn parse_record_count(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    let count =
        raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()).count();
    u64::try_from(count).map_err(|_| anyhow!("record count overflow"))
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfPrepareReferencePanelSmokeContract,
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
        materialize_governed_prepare_reference_panel_raw_fixture, parse_record_count,
        resolve_governed_vcf_prepare_reference_panel_smoke_contract,
        run_local_vcf_prepare_reference_panel_smoke,
        write_governed_prepare_reference_panel_input_vcf,
    };

    #[test]
    fn governed_prepare_reference_panel_contract_uses_matrix_row() {
        let contract = resolve_governed_vcf_prepare_reference_panel_smoke_contract("bcftools")
            .expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.prepare_reference_panel");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.input_fixture_id, "reference_panel_sort_and_deduplicate");
        assert_eq!(contract.panel_id, "hsapiens_grch38_mini");
        assert_eq!(contract.map_id, "hsapiens_grch38_chr_map");
    }

    #[test]
    fn governed_prepare_reference_panel_fixture_tracks_duplicate_count() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input_path = dir.path().join("input.vcf");
        write_governed_prepare_reference_panel_input_vcf(&input_path).expect("write input fixture");
        assert_eq!(parse_record_count(&input_path).expect("count input"), 4);

        let raw_panel =
            materialize_governed_prepare_reference_panel_raw_fixture(dir.path()).expect("panel");
        assert_eq!(parse_record_count(&raw_panel).expect("count raw panel"), 5);
    }

    #[test]
    fn governed_prepare_reference_panel_smoke_reports_normalized_panel_metrics() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        let report = run_local_vcf_prepare_reference_panel_smoke(repo_root.path(), "bcftools")
            .expect("run local panel smoke");
        assert_eq!(report.stage_id, "vcf.prepare_reference_panel");
        assert_eq!(report.tool_id, "bcftools");
        assert_eq!(report.input_variants, 5);
        assert_eq!(report.output_variants, 4);
        assert_eq!(report.duplicate_sites_removed, 1);
        assert_eq!(report.normalization_status, "sorted_indexed_deduplicated");
        assert_eq!(report.sample_count, 1);
        assert_eq!(report.sample_ids, vec!["sample1".to_string()]);
        assert!(report.sample_consistent);
        assert!(report.parseable);
    }
}
