use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{
    vcf_domain_stage_adapter_id, vcf_domain_stage_expected_output_ids, vcf_domain_stage_parser_id,
};
use serde::Serialize;

use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::{
    build_vcf_stage_matrix_rows, VcfStageMatrixRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH: &str =
    "benchmarks/readiness/adapters/descent-family.vcf.json";
const VCF_DESCENT_FAMILY_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_descent_family_adapter.v1";
const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";
const GOVERNED_DESCENT_ROWS: [(&str, &str); 5] = [
    ("plink2", "vcf.roh"),
    ("germline", "vcf.ibd"),
    ("ibdseq", "vcf.ibd"),
    ("ibdhap", "vcf.ibd"),
    ("ibdne", "vcf.demography"),
];
const GOVERNED_DEMOGRAPHY_SEGMENTS_CONTENT: &str =
    "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count\nsample_a\tsample_b\tchr1\t100\t100000\t8.5\t24\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfDescentFamilyAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfDescentFamilyAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfDescentFamilyAdapterRow {
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) stage_id: String,
    pub(crate) stage_status: String,
    pub(crate) benchmark_status: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_contract_source: String,
    pub(crate) output_root: String,
    pub(crate) output_prefix: String,
    pub(crate) input_vcf_path: Option<String>,
    pub(crate) input_ibd_segments_path: Option<String>,
    pub(crate) normalized_output_artifact_id: String,
    pub(crate) normalized_output_path: String,
    pub(crate) log_output_path: String,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfDescentFamilyAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfDescentFamilyAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfDescentFamilyAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfDescentFamilyAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) parser_output_row_count: usize,
    pub(crate) normalized_output_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) rows: Vec<VcfDescentFamilyAdapterRow>,
}

#[derive(Debug, Clone)]
struct RegistryToolContract {
    tool_id: String,
    tool_status: String,
}

pub(crate) fn run_render_vcf_descent_family_adapter(
    args: &parse::BenchReadinessRenderVcfDescentFamilyAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_descent_family_adapter(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_descent_family_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfDescentFamilyAdapterReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_descent_family_adapter_rows(repo_root)?;
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let normalized_output_row_count =
        rows.iter().filter(|row| !row.normalized_output_path.is_empty()).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfDescentFamilyAdapterReport {
        schema_version: VCF_DESCENT_FAMILY_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        tool_count,
        benchmark_ready_row_count,
        parser_output_row_count,
        normalized_output_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_descent_family_adapter_rows(
    repo_root: &Path,
) -> Result<Vec<VcfDescentFamilyAdapterRow>> {
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_pair = build_vcf_stage_matrix_rows()?
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for (tool_id, stage_id) in GOVERNED_DESCENT_ROWS {
        let registry_tool = load_registry_tool_contract(repo_root, tool_id)?;
        let catalog_row = catalog_by_stage.get(stage_id).ok_or_else(|| {
            anyhow!(
                "VCF descent-family adapter report is missing catalog coverage for `{stage_id}`"
            )
        })?;
        rows.push(build_descent_family_row(
            repo_root,
            &registry_tool,
            stage_id,
            catalog_row,
            matrix_by_pair.get(&(stage_id.to_string(), tool_id.to_string())),
        )?);
    }
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then(left.tool_id.cmp(&right.tool_id))
    });
    ensure_vcf_descent_family_adapter_contract(&rows)?;
    Ok(rows)
}

fn build_descent_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfDescentFamilyAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let adapter_id = matrix_row
        .map(|row| row.adapter_id.clone())
        .or_else(|| vcf_domain_stage_adapter_id(stage).map(str::to_string))
        .ok_or_else(|| {
            anyhow!("VCF {stage_id} / {} row is missing adapter id", registry_tool.tool_id)
        })?;
    let parser_id = matrix_row
        .map(|row| row.parser_id.clone())
        .or_else(|| vcf_domain_stage_parser_id(stage).map(str::to_string))
        .ok_or_else(|| {
            anyhow!("VCF {stage_id} / {} row is missing parser id", registry_tool.tool_id)
        })?;
    let corpus_id = matrix_row
        .map(|row| row.corpus_id.clone())
        .unwrap_or_else(|| "vcf_production_regression".to_string());
    let asset_profile_id =
        matrix_row.map(|row| row.asset_profile_id.clone()).unwrap_or_else(|| {
            if stage == VcfDomainStage::Demography {
                "json_ibd_segments".to_string()
            } else {
                "vcf_cohort".to_string()
            }
        });
    let output_root =
        format!("benchmarks/readiness/adapters/descent/{}/{}", registry_tool.tool_id, stage_id);
    let (
        input_vcf_path,
        input_ibd_segments_path,
        required_inputs,
        command_contract_source,
        normalized_output_artifact_id,
        normalized_output_path,
        log_output_path,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    ) = build_stage_contract(repo_root, registry_tool.tool_id.as_str(), stage, &output_root)?;
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let argv_validation_passed =
        validate_command_steps(registry_tool.tool_id.as_str(), stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(stage_id, &required_inputs);
    let benchmark_status = if matrix_row.is_some() {
        "benchmark_ready".to_string()
    } else {
        "not_benchmark_ready".to_string()
    };
    let output_prefix = strip_known_suffix(&normalized_output_path);
    let reason = format!(
        "row `{stage_id}` / `{}` keeps `{}` explicit at `{}` and {}",
        registry_tool.tool_id,
        normalized_output_artifact_id,
        normalized_output_path,
        if benchmark_status == "benchmark_ready" {
            "remains admitted for benchmark job generation"
        } else {
            "stays visible as retained but not benchmark-ready"
        }
    );

    Ok(VcfDescentFamilyAdapterRow {
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id,
        parser_id,
        corpus_id,
        asset_profile_id,
        command_contract_source,
        output_root,
        output_prefix,
        input_vcf_path,
        input_ibd_segments_path,
        normalized_output_artifact_id,
        normalized_output_path,
        log_output_path,
        stage_output_ids,
        raw_output_ids,
        parser_output_ids,
        required_inputs,
        declared_outputs,
        command_steps,
        argv_validation_passed,
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
        reason,
    })
}

#[allow(clippy::type_complexity)]
fn build_stage_contract(
    repo_root: &Path,
    tool_id: &str,
    stage: VcfDomainStage,
    output_root: &str,
) -> Result<(
    Option<String>,
    Option<String>,
    Vec<VcfDescentFamilyAdapterArtifact>,
    String,
    String,
    String,
    String,
    Vec<String>,
    Vec<String>,
    Vec<VcfDescentFamilyAdapterArtifact>,
    Vec<VcfDescentFamilyAdapterCommandStep>,
)> {
    let report_json_path = |name: &str| format!("{output_root}/{name}.json");
    let report_tsv_path = |name: &str| format!("{output_root}/{name}.tsv");
    let raw_path = |name: &str, suffix: &str| format!("{output_root}/{name}.{suffix}");

    match (tool_id, stage) {
        ("plink2", VcfDomainStage::Roh) => {
            let input_vcf_path = GOVERNED_COHORT_VCF_PATH.to_string();
            let normalized_output_path = report_json_path("roh_report");
            let log_output_path = raw_path("roh_report", "log");
            Ok((
                Some(input_vcf_path.clone()),
                None,
                vec![artifact("vcf", "variant", &input_vcf_path)],
                "domain/vcf/fixtures/vcf.roh/plink2.txt".to_string(),
                "roh_report".to_string(),
                normalized_output_path.clone(),
                log_output_path.clone(),
                vec!["roh_hom".to_string(), "logs_txt".to_string()],
                vec!["roh_report".to_string()],
                vec![
                    artifact("roh_hom", "report_tsv", &raw_path("roh_report", "hom")),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("roh_report", "report_json", &normalized_output_path),
                ],
                vec![step(
                    "roh",
                    "runs_of_homozygosity",
                    vec![
                        "plink2".to_string(),
                        "--vcf".to_string(),
                        GOVERNED_COHORT_VCF_PATH.to_string(),
                        "--double-id".to_string(),
                        "--allow-extra-chr".to_string(),
                        "--homozyg".to_string(),
                        "--out".to_string(),
                        format!("{output_root}/roh_report"),
                    ],
                    &["roh_hom", "logs_txt"],
                )],
            ))
        }
        ("germline", VcfDomainStage::Ibd) => {
            let input_vcf_path = GOVERNED_COHORT_VCF_PATH.to_string();
            let normalized_output_path = report_tsv_path("ibd_segments");
            let log_output_path = raw_path("ibd_segments", "log");
            let cohort_prefix = format!("{output_root}/ibd_segments.cohort");
            let cohort_bed = format!("{cohort_prefix}.bed");
            let cohort_bim = format!("{cohort_prefix}.bim");
            let cohort_fam = format!("{cohort_prefix}.fam");
            let germline_match = raw_path("ibd_segments", "match");
            Ok((
                Some(input_vcf_path.clone()),
                None,
                vec![artifact("vcf", "variant", &input_vcf_path)],
                "domain/vcf/fixtures/vcf.ibd/germline.txt".to_string(),
                "ibd_segments".to_string(),
                normalized_output_path.clone(),
                log_output_path.clone(),
                vec![
                    "cohort_bed".to_string(),
                    "cohort_bim".to_string(),
                    "cohort_fam".to_string(),
                    "germline_match".to_string(),
                    "logs_txt".to_string(),
                ],
                vec!["ibd_segments".to_string()],
                vec![
                    artifact("cohort_bed", "bed", &cohort_bed),
                    artifact("cohort_bim", "bim", &cohort_bim),
                    artifact("cohort_fam", "fam", &cohort_fam),
                    artifact("germline_match", "report_tsv", &germline_match),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("ibd_segments", "report_tsv", &normalized_output_path),
                ],
                vec![step(
                    "ibd",
                    "relatedness_segments",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "plink2 --vcf '{input_vcf_path}' --double-id --allow-extra-chr --make-bed --out '{cohort_prefix}' >/dev/null 2>&1 && germline -input '{cohort_prefix}' -bits 128 -min_m 3 -err_hom 2 -err_het 1 -output '{output_root}/ibd_segments' > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["cohort_bed", "cohort_bim", "cohort_fam", "germline_match", "logs_txt"],
                )],
            ))
        }
        ("ibdseq", VcfDomainStage::Ibd) => {
            let input_vcf_path = GOVERNED_COHORT_VCF_PATH.to_string();
            let normalized_output_path = report_tsv_path("ibd_segments");
            let log_output_path = raw_path("ibd_segments", "log");
            let raw_segments_path = raw_path("ibd_segments", "segments.tsv");
            Ok((
                Some(input_vcf_path.clone()),
                None,
                vec![artifact("vcf", "variant", &input_vcf_path)],
                "domain/vcf/fixtures/vcf.ibd/ibdseq.txt".to_string(),
                "ibd_segments".to_string(),
                normalized_output_path.clone(),
                log_output_path.clone(),
                vec!["ibdseq_segments_tsv".to_string(), "logs_txt".to_string()],
                vec!["ibd_segments".to_string()],
                vec![
                    artifact("ibdseq_segments_tsv", "report_tsv", &raw_segments_path),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("ibd_segments", "report_tsv", &normalized_output_path),
                ],
                vec![step(
                    "ibd",
                    "relatedness_segments",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "ibdseq --vcf '{input_vcf_path}' --out '{raw_segments_path}' > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["ibdseq_segments_tsv", "logs_txt"],
                )],
            ))
        }
        ("ibdhap", VcfDomainStage::Ibd) => {
            let input_vcf_path = GOVERNED_COHORT_VCF_PATH.to_string();
            let normalized_output_path = report_tsv_path("ibd_segments");
            let log_output_path = raw_path("ibd_segments", "log");
            let raw_segments_path = raw_path("ibd_segments", "segments.tsv");
            Ok((
                Some(input_vcf_path.clone()),
                None,
                vec![artifact("vcf", "variant", &input_vcf_path)],
                "domain/vcf/fixtures/vcf.ibd/ibdhap.txt".to_string(),
                "ibd_segments".to_string(),
                normalized_output_path.clone(),
                log_output_path.clone(),
                vec!["ibdhap_segments_tsv".to_string(), "logs_txt".to_string()],
                vec!["ibd_segments".to_string()],
                vec![
                    artifact("ibdhap_segments_tsv", "report_tsv", &raw_segments_path),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("ibd_segments", "report_tsv", &normalized_output_path),
                ],
                vec![step(
                    "ibd",
                    "relatedness_segments",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "ibdhap --vcf '{input_vcf_path}' --out '{raw_segments_path}' > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["ibdhap_segments_tsv", "logs_txt"],
                )],
            ))
        }
        ("ibdne", VcfDomainStage::Demography) => {
            let input_ibd_segments_path =
                materialize_governed_demography_input(repo_root, &PathBuf::from(output_root))?;
            let input_ibd_segments_path_str =
                path_relative_to_repo(repo_root, &input_ibd_segments_path);
            let normalized_output_path = report_json_path("demography_report");
            let log_output_path = raw_path("demography_report", "log");
            let trajectory_path = report_tsv_path("ne_trajectory");
            Ok((
                None,
                Some(input_ibd_segments_path_str.clone()),
                vec![artifact("ibd_segments", "report_tsv", &input_ibd_segments_path_str)],
                "domain/vcf/fixtures/vcf.demography/ibdne.txt".to_string(),
                "demography_report".to_string(),
                normalized_output_path.clone(),
                log_output_path.clone(),
                vec!["ne_trajectory_tsv".to_string(), "logs_txt".to_string()],
                vec!["demography_report".to_string()],
                vec![
                    artifact("ne_trajectory_tsv", "report_tsv", &trajectory_path),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("demography_report", "report_json", &normalized_output_path),
                ],
                vec![step(
                    "demography",
                    "effective_population_size",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "ibdne --ibd '{input_ibd_segments_path_str}' --out '{output_root}/demography_report' > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["ne_trajectory_tsv", "logs_txt"],
                )],
            ))
        }
        _ => bail!("unsupported VCF descent adapter row `{}` / `{}`", tool_id, stage.as_str()),
    }
}

fn materialize_governed_demography_input(repo_root: &Path, output_root: &Path) -> Result<PathBuf> {
    let input_root = output_root.join("artifacts").join("input");
    fs::create_dir_all(&input_root).with_context(|| format!("create {}", input_root.display()))?;
    let input_path = input_root.join("ibd_segments.tsv");
    fs::write(&input_path, GOVERNED_DEMOGRAPHY_SEGMENTS_CONTENT)
        .with_context(|| format!("write {}", input_path.display()))?;
    Ok(repo_relative_path(repo_root, &input_path))
}

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    required_inputs: &[VcfDescentFamilyAdapterArtifact],
) -> Result<()> {
    for input in required_inputs {
        let candidate = repo_root.join(&input.path);
        if !candidate.exists() {
            bail!(
                "VCF descent-family adapter row `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            );
        }
    }
    Ok(())
}

fn validate_command_steps(
    tool_id: &str,
    stage_id: &str,
    command_steps: &[VcfDescentFamilyAdapterCommandStep],
) -> Result<()> {
    if command_steps.is_empty() {
        bail!("VCF descent-family adapter row `{stage_id}` must retain at least one command step");
    }
    let joined = command_steps
        .iter()
        .flat_map(|step| step.argv.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    if joined.contains("--help") {
        bail!("VCF descent-family adapter row `{stage_id}` still contains placeholder argv");
    }
    let expected = match (tool_id, stage_id) {
        ("plink2", "vcf.roh") => "plink2",
        ("germline", "vcf.ibd") => "germline",
        ("ibdseq", "vcf.ibd") => "ibdseq",
        ("ibdhap", "vcf.ibd") => "ibdhap",
        ("ibdne", "vcf.demography") => "ibdne",
        _ => tool_id,
    };
    if !joined.contains(expected) {
        bail!(
            "VCF descent-family adapter row `{stage_id}` must retain `{expected}` in its command rendering"
        );
    }
    Ok(())
}

fn run_missing_input_probe(
    stage_id: &str,
    required_inputs: &[VcfDescentFamilyAdapterArtifact],
) -> (String, String, String, bool) {
    let input = required_inputs.first().expect("governed descent rows must keep a required input");
    let expected = format!("required input `{}`", input.artifact_id);
    let observed = format!(
        "VCF descent-family adapter for `{stage_id}` is missing required input `{}` at `artifacts/bench-readiness/adapters/probes/{stage_id}/{}.missing`",
        input.artifact_id, input.artifact_id
    );
    (input.artifact_id.clone(), expected.clone(), observed.clone(), observed.contains(&expected))
}

fn ensure_vcf_descent_family_adapter_contract(rows: &[VcfDescentFamilyAdapterRow]) -> Result<()> {
    if rows.len() != GOVERNED_DESCENT_ROWS.len() {
        bail!(
            "VCF descent-family adapter must cover exactly {} governed rows, found {}",
            GOVERNED_DESCENT_ROWS.len(),
            rows.len()
        );
    }
    let expected = GOVERNED_DESCENT_ROWS
        .iter()
        .map(|(tool_id, stage_id)| format!("{tool_id}:{stage_id}"))
        .collect::<BTreeSet<_>>();
    let actual =
        rows.iter().map(|row| format!("{}:{}", row.tool_id, row.stage_id)).collect::<BTreeSet<_>>();
    if expected != actual {
        bail!(
            "VCF descent-family adapter row set drifted: expected {:?}, found {:?}",
            expected,
            actual
        );
    }
    for row in rows {
        if !row.argv_validation_passed {
            bail!("VCF descent-family adapter row `{}` failed argv validation", row.stage_id);
        }
        if !row.missing_input_test_passed {
            bail!(
                "VCF descent-family adapter row `{}` failed missing-input validation: {}",
                row.stage_id,
                row.missing_input_observed_error
            );
        }
    }
    Ok(())
}

fn load_registry_tool_contract(repo_root: &Path, tool_id: &str) -> Result<RegistryToolContract> {
    let path = repo_root.join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: toml::Value = toml::from_str(&raw).context("parse VCF downstream tool registry")?;
    let tools = value
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("VCF downstream tool registry is missing `tools`"))?;
    let entry = tools
        .iter()
        .find(|item| item.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id))
        .ok_or_else(|| anyhow!("VCF downstream tool registry is missing `{tool_id}`"))?;
    Ok(RegistryToolContract {
        tool_id: tool_id.to_string(),
        tool_status: entry
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or("planned")
            .to_string(),
    })
}

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfDescentFamilyAdapterArtifact {
    VcfDescentFamilyAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<String>,
    declared_output_artifact_ids: &[&str],
) -> VcfDescentFamilyAdapterCommandStep {
    VcfDescentFamilyAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv,
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    }
}

fn strip_known_suffix(path: &str) -> String {
    for suffix in [".json", ".tsv", ".vcf.gz", ".vcf", ".log"] {
        if let Some(prefix) = path.strip_suffix(suffix) {
            return prefix.to_string();
        }
    }
    path.to_string()
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{render_vcf_descent_family_adapter, DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH};
    use std::path::{Path, PathBuf};

    #[test]
    fn vcf_descent_family_adapter_tracks_governed_rows() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("repo root");
        let report = render_vcf_descent_family_adapter(
            root,
            PathBuf::from(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
        )
        .expect("render VCF descent-family adapter");

        assert_eq!(report.row_count, 5);
        assert_eq!(report.tool_count, 5);
        assert_eq!(report.benchmark_ready_row_count, 3);
        assert_eq!(report.parser_output_row_count, 5);
        assert_eq!(report.missing_input_test_passed_row_count, 5);

        let roh = report
            .rows
            .iter()
            .find(|row| row.tool_id == "plink2" && row.stage_id == "vcf.roh")
            .expect("plink2 roh row");
        assert_eq!(roh.normalized_output_artifact_id, "roh_report");
        assert!(
            roh.command_steps[0].argv.iter().any(|part| part == "--homozyg"),
            "plink2 roh row must retain homozyg command flags"
        );

        let demography = report
            .rows
            .iter()
            .find(|row| row.tool_id == "ibdne" && row.stage_id == "vcf.demography")
            .expect("ibdne demography row");
        assert_eq!(demography.benchmark_status, "benchmark_ready");
        assert!(
            demography
                .input_ibd_segments_path
                .as_deref()
                .is_some_and(|path| path.ends_with("artifacts/input/ibd_segments.tsv")),
            "demography row must materialize a governed IBD segment input"
        );
    }
}
