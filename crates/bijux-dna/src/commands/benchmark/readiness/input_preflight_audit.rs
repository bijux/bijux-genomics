use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::validate_typed_input_handoffs;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};
use serde::{Deserialize, Serialize};

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::all_domain_retained_tools::collect_all_domain_retained_tool_rows;
use super::stage_tool_assets::{collect_stage_tool_asset_rows, StageToolAssetRow};
use super::vcf_adapter_missing_input_audit::render_vcf_adapter_missing_input_audit;
use super::vcf_angsd_adapter::render_vcf_angsd_adapter;
use super::vcf_bcftools_adapter::render_vcf_bcftools_adapter;
use super::vcf_descent_family_adapter::render_vcf_descent_family_adapter;
use super::vcf_eigensoft_adapter::render_vcf_eigensoft_adapter;
use super::vcf_imputation_family_adapter::render_vcf_imputation_family_adapter;
use super::vcf_phasing_family_adapter::render_vcf_phasing_family_adapter;
use super::vcf_plink_family_adapter::render_vcf_plink_family_adapter;
use crate::commands::benchmark::local_stage_commands::local_stage_plans;
use crate::commands::cli::parse;
use crate::commands::cli::render;
use bijux_dna_stage_contract::StagePlanV1;

pub(crate) const DEFAULT_INPUT_PREFLIGHT_TESTS_PATH: &str =
    "benchmarks/readiness/tools/input-preflight-tests.json";
const INPUT_PREFLIGHT_TESTS_SCHEMA_VERSION: &str = "bijux.bench.readiness.input_preflight_audit.v1";
const REQUIRED_INPUT_CLASSES: [&str; 6] = ["fastq", "bam", "vcf", "reference", "database", "panel"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct InputPreflightTestRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) contract_surface: String,
    pub(crate) missing_input_role: String,
    pub(crate) missing_input_class: String,
    pub(crate) artifact_path: String,
    pub(crate) expected_error_fragment: String,
    pub(crate) observed_error: String,
    pub(crate) passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InputPreflightTestsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) passed_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) retained_tool_count: usize,
    pub(crate) covered_tool_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) missing_input_class_counts: BTreeMap<String, usize>,
    pub(crate) covered_tool_ids: Vec<String>,
    pub(crate) rows: Vec<InputPreflightTestRow>,
}

#[derive(Debug, Clone)]
struct ProbeArtifact {
    name: ArtifactId,
    path: PathBuf,
    role: ArtifactRole,
    missing_input_class: String,
}

#[derive(Debug, Deserialize)]
struct LocalContaminationInputs {
    bam: PathBuf,
    bai: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalGenotypingInputs {
    bam: PathBuf,
    bai: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalHaplogroupsInputs {
    bam: PathBuf,
    bai: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalCaseInputs {
    bam: PathBuf,
    #[serde(default)]
    reference: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalCaseConfig {
    cases: Vec<LocalCaseInputs>,
}

pub(crate) fn run_render_input_preflight_audit(
    args: &parse::BenchReadinessRenderInputPreflightTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_input_preflight_audit(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_INPUT_PREFLIGHT_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_input_preflight_audit(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<InputPreflightTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_input_preflight_test_rows(repo_root)?;
    let retained_tool_ids = collect_all_domain_retained_tool_rows(repo_root)?
        .into_iter()
        .map(|row| row.tool_id)
        .collect::<BTreeSet<_>>();
    let row_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>();
    let covered_tool_ids =
        retained_tool_ids.intersection(&row_tool_ids).cloned().collect::<BTreeSet<_>>();

    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut missing_input_class_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *missing_input_class_counts.entry(row.missing_input_class.clone()).or_default() += 1;
    }

    ensure_tool_coverage(&retained_tool_ids, &covered_tool_ids)?;
    ensure_required_class_coverage(&missing_input_class_counts)?;
    ensure_no_ambiguous_input_classes(&missing_input_class_counts)?;
    if failed_row_count != 0 {
        let failed_rows = rows
            .iter()
            .filter(|row| !row.passed)
            .map(|row| {
                format!(
                    "{}:{}:{} expected `{}` observed `{}`",
                    row.stage_id,
                    row.tool_id,
                    row.missing_input_role,
                    row.expected_error_fragment,
                    row.observed_error
                )
            })
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "retained-tool input preflight tests must pass for every governed probe, failed rows: {}",
            failed_rows.join(", ")
        ));
    }

    let report = InputPreflightTestsReport {
        schema_version: INPUT_PREFLIGHT_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        retained_tool_count: retained_tool_ids.len(),
        covered_tool_count: covered_tool_ids.len(),
        domain_counts,
        missing_input_class_counts,
        covered_tool_ids: covered_tool_ids.into_iter().collect(),
        rows,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_input_preflight_test_rows(repo_root: &Path) -> Result<Vec<InputPreflightTestRow>> {
    let mut rows = collect_fastq_and_bam_input_preflight_rows(repo_root)?;
    rows.extend(collect_vcf_input_preflight_rows(repo_root)?);
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.missing_input_role.cmp(&right.missing_input_role))
    });
    Ok(rows)
}

fn collect_fastq_and_bam_input_preflight_rows(
    repo_root: &Path,
) -> Result<Vec<InputPreflightTestRow>> {
    let asset_rows = collect_stage_tool_asset_rows(repo_root)?;
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| matches!(row.domain.as_str(), "fastq" | "bam"))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for row in active_rows {
        let artifacts = collect_probe_artifacts_for_binding(repo_root, &row, &asset_rows)?;
        let base_step = build_probe_step(repo_root, &row, &artifacts)?;
        validate_stage_inputs(&base_step).with_context(|| {
            format!(
                "validate governed baseline inputs for `{}` / `{}` / `{}`",
                row.domain, row.stage_id, row.tool_id
            )
        })?;
        for artifact in &artifacts {
            rows.push(run_runtime_probe(repo_root, &row, &base_step, artifact));
        }
    }
    Ok(rows)
}

fn collect_probe_artifacts_for_binding(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
    asset_rows: &[StageToolAssetRow],
) -> Result<Vec<ProbeArtifact>> {
    let mut artifacts = if row.domain == "bam" && stage_uses_manual_bam_inputs(&row.stage_id) {
        collect_manual_bam_probe_artifacts(repo_root, &row.stage_id)?
    } else {
        collect_plan_probe_artifacts(repo_root, row)?
    };
    let synthetic_assets = asset_rows
        .iter()
        .filter(|asset| {
            asset.domain == row.domain
                && asset.stage_id == row.stage_id
                && asset.tool_id == row.tool_id
                && !asset.asset_role.ends_with("_output")
        })
        .map(stage_tool_asset_to_probe_artifact)
        .collect::<Vec<_>>();
    artifacts.extend(synthetic_assets);
    dedupe_probe_artifacts(&mut artifacts);
    if artifacts.is_empty() {
        return Err(anyhow!(
            "retained-tool input preflight tests found no governed inputs for `{}` / `{}` / `{}`",
            row.domain,
            row.stage_id,
            row.tool_id
        ));
    }
    Ok(artifacts)
}

fn collect_plan_probe_artifacts(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
) -> Result<Vec<ProbeArtifact>> {
    let plans = input_preflight_stage_plans(repo_root, &row.stage_id)?;
    let plan = if let Some(plan) = plans.iter().find(|plan| plan.tool_id.as_str() == row.tool_id) {
        plan
    } else if plans.len() == 1
        || all_plans_share_required_inputs(&plans)
        || all_plans_share_tool_id(&plans)
    {
        &plans[0]
    } else {
        return Err(anyhow!(
            "local benchmark stage `{}` did not yield a governed plan for retained tool `{}`",
            row.stage_id,
            row.tool_id
        ));
    };
    Ok(plan
        .io
        .inputs
        .clone()
        .into_iter()
        .filter(|artifact| !artifact.optional)
        .map(|artifact| ProbeArtifact {
            missing_input_class: infer_missing_input_class(artifact.name.as_str(), &artifact.path),
            name: artifact.name,
            path: artifact.path,
            role: artifact.role,
        })
        .collect())
}

fn input_preflight_stage_plans(repo_root: &Path, stage_id: &str) -> Result<Vec<StagePlanV1>> {
    match stage_id {
        "fastq.index_reference" => {
            bijux_dna_planner_fastq::stage_api::local_index_reference_output_contract_plans(
                repo_root,
            )
        }
        "fastq.profile_reads" => {
            Ok(bijux_dna_planner_fastq::stage_api::local_profile_reads_output_contract_plans(
                repo_root,
            )?
            .into_iter()
            .map(|case| case.plan)
            .collect())
        }
        "fastq.screen_taxonomy" => {
            bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_output_contract_plans(
                repo_root,
            )
        }
        "fastq.trim_reads" => Ok(
            bijux_dna_planner_fastq::stage_api::local_trim_reads_output_contract_plans(repo_root)?
                .into_iter()
                .map(|case| case.plan)
                .collect(),
        ),
        "fastq.trim_terminal_damage" => Ok(
            bijux_dna_planner_fastq::stage_api::local_trim_terminal_damage_output_contract_plans(
                repo_root,
            )?
            .into_iter()
            .map(|case| case.plan)
            .collect(),
        ),
        _ => local_stage_plans(repo_root, stage_id),
    }
}

fn all_plans_share_required_inputs(plans: &[StagePlanV1]) -> bool {
    let Some(first) = plans.first() else {
        return false;
    };
    let first_signature = required_input_signature(first);
    plans.iter().skip(1).all(|plan| required_input_signature(plan) == first_signature)
}

fn all_plans_share_tool_id(plans: &[StagePlanV1]) -> bool {
    let Some(first) = plans.first() else {
        return false;
    };
    plans.iter().skip(1).all(|plan| plan.tool_id.as_str() == first.tool_id.as_str())
}

fn required_input_signature(plan: &StagePlanV1) -> Vec<(String, String, String)> {
    plan.io
        .inputs
        .iter()
        .filter(|artifact| !artifact.optional)
        .map(|artifact| {
            (
                artifact.name.as_str().to_string(),
                artifact.role.as_str().to_string(),
                artifact.path.to_string_lossy().replace('\\', "/"),
            )
        })
        .collect()
}

fn collect_manual_bam_probe_artifacts(
    repo_root: &Path,
    stage_id: &str,
) -> Result<Vec<ProbeArtifact>> {
    match stage_id {
        "bam.bias_mitigation" => {
            let config = load_toml_config::<LocalCaseConfig>(
                repo_root,
                "configs/bench/local/bam-bias-mitigation.toml",
            )?;
            let case = config.cases.first().ok_or_else(|| {
                anyhow!("local bam.bias_mitigation config must keep at least one governed case")
            })?;
            let mut artifacts = vec![ProbeArtifact {
                name: ArtifactId::new("bam"),
                path: case.bam.clone(),
                role: ArtifactRole::Bam,
                missing_input_class: "bam".to_string(),
            }];
            if let Some(reference) = case.reference.as_ref() {
                artifacts.push(ProbeArtifact {
                    name: ArtifactId::new("reference_fasta"),
                    path: reference.clone(),
                    role: ArtifactRole::Reference,
                    missing_input_class: "reference".to_string(),
                });
            }
            Ok(artifacts)
        }
        "bam.contamination" => {
            let config = load_toml_config::<LocalContaminationInputs>(
                repo_root,
                "configs/bench/local/bam-contamination.toml",
            )?;
            Ok(vec![
                ProbeArtifact {
                    name: ArtifactId::new("bam"),
                    path: config.bam,
                    role: ArtifactRole::Bam,
                    missing_input_class: "bam".to_string(),
                },
                ProbeArtifact {
                    name: ArtifactId::new("bam_bai"),
                    path: config.bai,
                    role: ArtifactRole::Index,
                    missing_input_class: "bam".to_string(),
                },
            ])
        }
        "bam.genotyping" => {
            let config = load_toml_config::<LocalGenotypingInputs>(
                repo_root,
                "configs/bench/local/bam-genotyping.toml",
            )?;
            Ok(vec![
                ProbeArtifact {
                    name: ArtifactId::new("bam"),
                    path: config.bam,
                    role: ArtifactRole::Bam,
                    missing_input_class: "bam".to_string(),
                },
                ProbeArtifact {
                    name: ArtifactId::new("bam_bai"),
                    path: config.bai,
                    role: ArtifactRole::Index,
                    missing_input_class: "bam".to_string(),
                },
            ])
        }
        "bam.haplogroups" => {
            let config = load_toml_config::<LocalHaplogroupsInputs>(
                repo_root,
                "configs/bench/local/bam-haplogroups.toml",
            )?;
            Ok(vec![
                ProbeArtifact {
                    name: ArtifactId::new("bam"),
                    path: config.bam,
                    role: ArtifactRole::Bam,
                    missing_input_class: "bam".to_string(),
                },
                ProbeArtifact {
                    name: ArtifactId::new("bam_bai"),
                    path: config.bai,
                    role: ArtifactRole::Index,
                    missing_input_class: "bam".to_string(),
                },
            ])
        }
        "bam.kinship" => {
            let config = load_toml_config::<LocalCaseConfig>(
                repo_root,
                "configs/bench/local/bam-kinship.toml",
            )?;
            let case = config.cases.first().ok_or_else(|| {
                anyhow!("local bam.kinship config must keep at least one governed case")
            })?;
            Ok(vec![ProbeArtifact {
                name: ArtifactId::new("bam"),
                path: case.bam.clone(),
                role: ArtifactRole::Bam,
                missing_input_class: "bam".to_string(),
            }])
        }
        other => Err(anyhow!("unsupported manual BAM preflight stage `{other}`")),
    }
}

fn stage_tool_asset_to_probe_artifact(asset: &StageToolAssetRow) -> ProbeArtifact {
    let path = PathBuf::from(&asset.asset_path);
    ProbeArtifact {
        name: ArtifactId::new(asset.asset_role.clone()),
        path: path.clone(),
        role: ArtifactRole::Unknown,
        missing_input_class: infer_missing_input_class(&asset.asset_role, &path),
    }
}

fn dedupe_probe_artifacts(artifacts: &mut Vec<ProbeArtifact>) {
    let mut seen = BTreeSet::<(String, String)>::new();
    artifacts.retain(|artifact| {
        seen.insert((
            artifact.name.as_str().to_string(),
            artifact.path.to_string_lossy().replace('\\', "/"),
        ))
    });
}

fn build_probe_step(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
    artifacts: &[ProbeArtifact],
) -> Result<ExecutionStep> {
    let inputs = artifacts
        .iter()
        .map(|artifact| {
            let validation_path = prepare_input_path_for_validation(repo_root, row, artifact)?;
            let validation_role = validation_role_for_probe_artifact(repo_root, artifact)?;
            Ok(ArtifactSpec::required(artifact.name.clone(), validation_path, validation_role))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ExecutionStep {
        step_id: StepId::new(format!(
            "bench.readiness.{}.{}",
            row.stage_id.replace('.', "_"),
            row.tool_id
        )),
        stage_id: StageId::new(row.stage_id.clone()),
        command: CommandSpecV1 { template: vec!["true".to_string()] },
        image: ContainerImageRefV1 { image: "bijuxdna/preflight:latest".to_string(), digest: None },
        resources: ToolConstraints::default(),
        io: StageIO { inputs, outputs: Vec::new() },
        out_dir: repo_root.join("artifacts/bench-readiness/input-preflight-tests"),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    })
}

fn run_runtime_probe(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
    base_step: &ExecutionStep,
    artifact: &ProbeArtifact,
) -> InputPreflightTestRow {
    let mut mutated_step = base_step.clone();
    if let Some(input) = mutated_step.io.inputs.iter_mut().find(|input| input.name == artifact.name)
    {
        input.path = missing_probe_path(repo_root, row, artifact);
    }
    let expected_error_fragment = format!("missing required input {}", artifact.name.as_str());
    let observed_error = match validate_stage_inputs(&mutated_step) {
        Ok(()) => format!(
            "{} unexpectedly accepted missing input role `{}`",
            row.stage_id,
            artifact.name.as_str()
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);

    InputPreflightTestRow {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        contract_surface: "runtime_validation".to_string(),
        missing_input_role: artifact.name.as_str().to_string(),
        missing_input_class: artifact.missing_input_class.clone(),
        artifact_path: normalize_report_path(&artifact.path),
        expected_error_fragment,
        observed_error,
        passed,
        reason: format!(
            "governed {} benchmark-ready binding `{}` / `{}` fails before external execution through runtime input validation",
            row.domain, row.stage_id, row.tool_id
        ),
    }
}

fn collect_vcf_input_preflight_rows(repo_root: &Path) -> Result<Vec<InputPreflightTestRow>> {
    let temp_root = repo_root.join("artifacts/bench-readiness/input-preflight-tests/vcf");
    fs::create_dir_all(&temp_root).with_context(|| format!("create {}", temp_root.display()))?;

    let mut rows = Vec::new();

    let bcftools = render_vcf_bcftools_adapter(repo_root, temp_root.join("bcftools.adapter.json"))?;
    rows.extend(bcftools.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    let angsd = render_vcf_angsd_adapter(repo_root, temp_root.join("angsd.adapter.json"))?;
    rows.extend(angsd.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    let eigensoft =
        render_vcf_eigensoft_adapter(repo_root, temp_root.join("eigensoft.adapter.json"))?;
    rows.extend(eigensoft.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    let plink =
        render_vcf_plink_family_adapter(repo_root, "plink", temp_root.join("plink.adapter.json"))?;
    rows.extend(plink.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    let plink2 = render_vcf_plink_family_adapter(
        repo_root,
        "plink2",
        temp_root.join("plink2.adapter.json"),
    )?;
    rows.extend(plink2.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    for (tool_id, filename) in [
        ("shapeit5", "shapeit5.adapter.json"),
        ("eagle", "eagle.adapter.json"),
        ("beagle", "beagle.adapter.json"),
    ] {
        let report =
            render_vcf_phasing_family_adapter(repo_root, tool_id, temp_root.join(filename))?;
        rows.extend(report.rows.into_iter().map(|row| {
            map_vcf_adapter_row(
                "adapter_contract",
                row.stage_id,
                row.tool_id,
                row.required_inputs
                    .iter()
                    .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                    .map(|input| (input.artifact_id.clone(), input.path.clone()))
                    .unwrap_or_else(|| {
                        (row.missing_input_probe_artifact_id.clone(), String::new())
                    }),
                row.missing_input_expected_error_fragment,
                row.missing_input_observed_error,
                row.missing_input_test_passed,
                row.reason,
            )
        }));
    }

    let imputation = render_vcf_imputation_family_adapter(
        repo_root,
        temp_root.join("imputation-family.adapter.json"),
    )?;
    rows.extend(imputation.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    let descent = render_vcf_descent_family_adapter(
        repo_root,
        temp_root.join("descent-family.adapter.json"),
    )?;
    rows.extend(descent.rows.into_iter().map(|row| {
        map_vcf_adapter_row(
            "adapter_contract",
            row.stage_id,
            row.tool_id,
            row.required_inputs
                .iter()
                .find(|input| input.artifact_id == row.missing_input_probe_artifact_id)
                .map(|input| (input.artifact_id.clone(), input.path.clone()))
                .unwrap_or_else(|| (row.missing_input_probe_artifact_id.clone(), String::new())),
            row.missing_input_expected_error_fragment,
            row.missing_input_observed_error,
            row.missing_input_test_passed,
            row.reason,
        )
    }));

    let support_report = render_vcf_adapter_missing_input_audit(
        repo_root,
        temp_root.join("adapter-missing-input-support.json"),
    )?;
    rows.extend(support_report.rows.into_iter().map(|row| InputPreflightTestRow {
        domain: "vcf".to_string(),
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        contract_surface: row.contract_surface,
        missing_input_role: row.missing_input_role.clone(),
        missing_input_class: infer_missing_input_class(
            &row.missing_input_role,
            Path::new(&row.artifact_path),
        ),
        artifact_path: row.artifact_path,
        expected_error_fragment: row.expected_error_fragment,
        observed_error: row.observed_error,
        passed: row.passed,
        reason: row.reason,
    }));

    Ok(rows)
}

fn map_vcf_adapter_row(
    contract_surface: &str,
    stage_id: String,
    tool_id: String,
    missing_artifact: (String, String),
    expected_error_fragment: String,
    observed_error: String,
    passed: bool,
    reason: String,
) -> InputPreflightTestRow {
    let artifact_path = missing_artifact.1;
    InputPreflightTestRow {
        domain: "vcf".to_string(),
        stage_id,
        tool_id,
        contract_surface: contract_surface.to_string(),
        missing_input_role: missing_artifact.0.clone(),
        missing_input_class: infer_missing_input_class(
            &missing_artifact.0,
            Path::new(&artifact_path),
        ),
        artifact_path,
        expected_error_fragment,
        observed_error,
        passed,
        reason,
    }
}

fn ensure_tool_coverage(
    retained_tool_ids: &BTreeSet<String>,
    covered_tool_ids: &BTreeSet<String>,
) -> Result<()> {
    let missing = retained_tool_ids.difference(covered_tool_ids).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(anyhow!(
            "retained-tool input preflight tests are missing governed coverage for: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

fn ensure_required_class_coverage(counts: &BTreeMap<String, usize>) -> Result<()> {
    let missing = REQUIRED_INPUT_CLASSES
        .into_iter()
        .filter(|class_id| !counts.contains_key(*class_id))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(anyhow!(
            "retained-tool input preflight tests must cover these governed input classes: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

fn ensure_no_ambiguous_input_classes(counts: &BTreeMap<String, usize>) -> Result<()> {
    if let Some(other_count) = counts.get("other") {
        return Err(anyhow!(
            "retained-tool input preflight tests must not emit ambiguous `other` input classes, found {other_count}"
        ));
    }
    Ok(())
}

fn infer_missing_input_class(role: &str, path: &Path) -> String {
    let role = role.to_ascii_lowercase();
    let path_text = path.to_string_lossy().to_ascii_lowercase();
    if role.contains("metadata") {
        return "metadata".to_string();
    }
    if role.contains("panel") {
        return "panel".to_string();
    }
    if role.contains("regions")
        || role.contains("sites_bed")
        || path_text.ends_with(".bed")
        || path_text.contains("/regions/")
    {
        return "panel".to_string();
    }
    if role.contains("database")
        || path_text.contains("/database/")
        || path_text.contains("/taxonomy/")
        || path_text.contains("rrna")
    {
        return "database".to_string();
    }
    if role.contains("reads")
        || path_text.ends_with(".fastq")
        || path_text.ends_with(".fq")
        || path_text.ends_with(".fastq.gz")
        || path_text.ends_with(".fq.gz")
    {
        return "fastq".to_string();
    }
    if role.contains("bam")
        || role.contains("cram")
        || path_text.ends_with(".bam")
        || path_text.ends_with(".sam")
        || path_text.ends_with(".cram")
        || path_text.ends_with(".bai")
        || path_text.ends_with(".crai")
    {
        return "bam".to_string();
    }
    if role.contains("vcf")
        || role.contains("bcf")
        || path_text.ends_with(".vcf")
        || path_text.ends_with(".vcf.gz")
        || path_text.ends_with(".bcf")
        || path_text.ends_with(".tbi")
        || path_text.ends_with(".csi")
    {
        return "vcf".to_string();
    }
    if role.contains("reference")
        || path_text.contains("/reference/")
        || path_text.ends_with(".fasta")
        || path_text.ends_with(".fa")
        || path_text.ends_with(".fna")
        || path_text.ends_with(".fai")
        || path_text.ends_with(".dict")
    {
        return "reference".to_string();
    }
    if role.contains("index") {
        return "index".to_string();
    }
    if role.contains("segment") {
        return "segments".to_string();
    }
    if role.contains("table") || path_text.contains("/tables/") {
        return "table".to_string();
    }
    if role.contains("report") || path_text.ends_with(".json") {
        return "report".to_string();
    }
    "other".to_string()
}

fn missing_probe_path(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
    artifact: &ProbeArtifact,
) -> PathBuf {
    let extension = preserved_probe_suffix(&artifact.path);
    repo_root
        .join("artifacts/bench-readiness/input-preflight-tests/missing")
        .join(&row.domain)
        .join(row.stage_id.replace('.', "/"))
        .join(&row.tool_id)
        .join(format!("{}{}", artifact.name.as_str(), extension))
}

fn preserved_probe_suffix(path: &Path) -> String {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    for suffix in [
        ".fastq.gz",
        ".fq.gz",
        ".vcf.gz",
        ".m3vcf.gz",
        ".bam.bai",
        ".bam.csi",
        ".cram.crai",
        ".bcf.csi",
    ] {
        if lower.ends_with(suffix) {
            return suffix.to_string();
        }
    }
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|value| format!(".{value}"))
        .unwrap_or_default()
}

fn prepare_input_path_for_validation(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
    artifact: &ProbeArtifact,
) -> Result<PathBuf> {
    let original = if artifact.path.is_absolute() {
        artifact.path.clone()
    } else {
        repo_root.join(&artifact.path)
    };
    if !original.exists() {
        if let Some(anchor_path) =
            materialize_prefix_input_anchor(repo_root, row, artifact, &original)?
        {
            return Ok(anchor_path);
        }
        return Ok(artifact.path.clone());
    }
    if !original.is_file() {
        return Ok(original);
    }

    if let Some(sidecar_path) = missing_required_sidecar(&original) {
        let support_root = repo_root
            .join("artifacts/bench-readiness/input-preflight-tests/support")
            .join(&row.domain)
            .join(row.stage_id.replace('.', "/"))
            .join(&row.tool_id)
            .join(artifact.name.as_str());
        fs::create_dir_all(&support_root)
            .with_context(|| format!("create {}", support_root.display()))?;
        let support_path = support_root.join(
            original
                .file_name()
                .ok_or_else(|| anyhow!("resolve support filename for {}", original.display()))?,
        );
        fs::copy(&original, &support_path).with_context(|| {
            format!(
                "copy governed probe support file {} -> {}",
                original.display(),
                support_path.display()
            )
        })?;
        let support_sidecar = support_root.join(sidecar_path.file_name().ok_or_else(|| {
            anyhow!("resolve support sidecar filename for {}", sidecar_path.display())
        })?);
        if !support_sidecar.exists() {
            fs::write(&support_sidecar, [])
                .with_context(|| format!("write {}", support_sidecar.display()))?;
        }
        return Ok(support_path);
    }

    Ok(artifact.path.clone())
}

fn validation_role_for_probe_artifact(
    repo_root: &Path,
    artifact: &ProbeArtifact,
) -> Result<ArtifactRole> {
    let original = if artifact.path.is_absolute() {
        artifact.path.clone()
    } else {
        repo_root.join(&artifact.path)
    };
    if artifact.role == ArtifactRole::Index {
        let lower = original.to_string_lossy().to_ascii_lowercase();
        let is_runtime_checked_index = lower.ends_with(".bai")
            || lower.ends_with(".crai")
            || lower.ends_with(".tbi")
            || lower.ends_with(".csi")
            || lower.ends_with(".bam.bai")
            || lower.ends_with(".bam.csi")
            || lower.ends_with(".bcf.csi");
        if !is_runtime_checked_index || supports_prefix_backed_input(&original)? {
            return Ok(ArtifactRole::Unknown);
        }
    }
    Ok(artifact.role)
}

fn materialize_prefix_input_anchor(
    repo_root: &Path,
    row: &AllDomainActiveStageToolMatrixRow,
    artifact: &ProbeArtifact,
    original: &Path,
) -> Result<Option<PathBuf>> {
    if !supports_prefix_backed_input(original)? {
        return Ok(None);
    }
    let prefix = original
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| anyhow!("resolve prefix-backed input name for {}", original.display()))?;

    let support_root = repo_root
        .join("artifacts/bench-readiness/input-preflight-tests/support")
        .join(&row.domain)
        .join(row.stage_id.replace('.', "/"))
        .join(&row.tool_id)
        .join(artifact.name.as_str());
    fs::create_dir_all(&support_root)
        .with_context(|| format!("create {}", support_root.display()))?;
    let support_path = support_root.join(prefix);
    if !support_path.exists() {
        fs::write(&support_path, [])
            .with_context(|| format!("write {}", support_path.display()))?;
    }
    Ok(Some(support_path))
}

fn supports_prefix_backed_input(path: &Path) -> Result<bool> {
    let parent = match path.parent() {
        Some(parent) if parent.is_dir() => parent,
        _ => return Ok(false),
    };
    let prefix = match path.file_name().and_then(std::ffi::OsStr::to_str) {
        Some(prefix) if !prefix.is_empty() => prefix,
        _ => return Ok(false),
    };
    Ok(fs::read_dir(parent)
        .with_context(|| format!("read {}", parent.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| candidate.is_file())
        .filter_map(|candidate| {
            candidate.file_name().and_then(std::ffi::OsStr::to_str).map(|name| name.to_string())
        })
        .any(|name| {
            name.starts_with(&format!("{prefix}."))
                || name.starts_with(&format!("{prefix}_"))
                || name.starts_with(&format!("{prefix}-"))
        }))
}

fn missing_required_sidecar(path: &Path) -> Option<PathBuf> {
    let path_text = path.to_string_lossy().to_ascii_lowercase();
    if path_text.ends_with(".vcf.gz") || path_text.ends_with(".vcf") {
        let tbi = PathBuf::from(format!("{}.tbi", path.display()));
        let csi = PathBuf::from(format!("{}.csi", path.display()));
        if !tbi.exists() && !csi.exists() {
            return Some(tbi);
        }
    }
    if path_text.ends_with(".bcf") {
        let csi = path.with_extension("bcf.csi");
        let fallback = path.with_extension("csi");
        if !csi.exists() && !fallback.exists() {
            return Some(csi);
        }
    }
    if path_text.ends_with(".bam") {
        let bai = path.with_extension("bam.bai");
        let fallback_bai = path.with_extension("bai");
        let csi = path.with_extension("bam.csi");
        let fallback_csi = path.with_extension("csi");
        if !bai.exists() && !fallback_bai.exists() && !csi.exists() && !fallback_csi.exists() {
            return Some(bai);
        }
    }
    if path_text.ends_with(".cram") {
        let crai = path.with_extension("cram.crai");
        let fallback = path.with_extension("crai");
        if !crai.exists() && !fallback.exists() {
            return Some(crai);
        }
    }
    None
}

fn normalize_report_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn load_toml_config<T: for<'de> Deserialize<'de>>(
    repo_root: &Path,
    relative_path: &str,
) -> Result<T> {
    let path = repo_root.join(relative_path);
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&payload).with_context(|| format!("parse {}", path.display()))
}

fn stage_uses_manual_bam_inputs(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "bam.bias_mitigation"
            | "bam.contamination"
            | "bam.genotyping"
            | "bam.haplogroups"
            | "bam.kinship"
    )
}

fn validate_stage_inputs(step: &ExecutionStep) -> Result<()> {
    validate_typed_input_handoffs(step)?;
    for artifact in &step.io.inputs {
        let path = &artifact.path;
        if !path.exists() {
            if !artifact.optional {
                return Err(anyhow!(
                    "input contract violation: missing required input {} ({})",
                    artifact.name,
                    path.display()
                ));
            }
            continue;
        }
        validate_bgzip_tabix(path)?;
        validate_bam_index(path)?;
        validate_fastq_format(path)?;
    }
    Ok(())
}

fn validate_bgzip_tabix(input: &Path) -> Result<()> {
    if file_name_ends_with(input, ".vcf.gz") || has_extension(input, "vcf") {
        let tbi = PathBuf::from(format!("{}.tbi", input.display()));
        let csi = PathBuf::from(format!("{}.csi", input.display()));
        if !tbi.exists() && !csi.exists() {
            return Err(anyhow!(
                "input contract violation: missing VCF index (.tbi/.csi) for {}",
                input.display()
            ));
        }
    }
    if has_extension(input, "bcf") {
        let candidates = [input.with_extension("bcf.csi"), input.with_extension("csi")];
        if !candidates.iter().any(|path| path.exists()) {
            return Err(anyhow!(
                "input contract violation: missing BCF index (.csi) for {}",
                input.display()
            ));
        }
    }
    Ok(())
}

fn validate_bam_index(input: &Path) -> Result<()> {
    if has_extension(input, "bam") {
        let candidates = [
            input.with_extension("bam.bai"),
            input.with_extension("bai"),
            input.with_extension("bam.csi"),
            input.with_extension("csi"),
        ];
        if !candidates.iter().any(|path| path.exists()) {
            return Err(anyhow!(
                "input contract violation: missing BAM index (.bai/.csi) for {}",
                input.display()
            ));
        }
    }
    if has_extension(input, "cram") {
        let candidates = [input.with_extension("cram.crai"), input.with_extension("crai")];
        if !candidates.iter().any(|path| path.exists()) {
            return Err(anyhow!(
                "input contract violation: missing CRAM index (.crai) for {}",
                input.display()
            ));
        }
    }
    Ok(())
}

fn validate_fastq_format(input: &Path) -> Result<()> {
    let is_gzip_fastq =
        file_name_ends_with(input, ".fastq.gz") || file_name_ends_with(input, ".fq.gz");
    if !(has_extension(input, "fastq") || has_extension(input, "fq") || is_gzip_fastq) {
        return Ok(());
    }
    let file = std::fs::File::open(input).with_context(|| format!("open {}", input.display()))?;
    let mut reader: Box<dyn BufRead> = if is_gzip_fastq {
        Box::new(std::io::BufReader::new(flate2::read::MultiGzDecoder::new(file)))
    } else {
        Box::new(std::io::BufReader::new(file))
    };
    let mut first = String::new();
    let _ = reader.read_line(&mut first).with_context(|| format!("read {}", input.display()))?;
    if !first.starts_with('@') {
        return Err(anyhow!(
            "input contract violation: FASTQ header must start with '@' ({})",
            input.display()
        ));
    }
    Ok(())
}

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|value| value.eq_ignore_ascii_case(ext))
}

fn file_name_ends_with(path: &Path, suffix: &str) -> bool {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(suffix))
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
