use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::local_vcf_admixture_smoke;
use super::local_vcf_call_diploid_smoke;
use super::local_vcf_call_gl_smoke;
use super::local_vcf_call_pseudohaploid_smoke;
use super::local_vcf_call_smoke;
use super::local_vcf_damage_filter_smoke;
use super::local_vcf_demography_smoke;
use super::local_vcf_filter_smoke;
use super::local_vcf_gl_propagation_smoke;
use super::local_vcf_ibd_smoke;
use super::local_vcf_imputation_metrics_smoke;
use super::local_vcf_impute_smoke;
use super::local_vcf_pca_smoke;
use super::local_vcf_phasing_smoke;
use super::local_vcf_population_structure_smoke;
use super::local_vcf_postprocess_smoke;
use super::local_vcf_prepare_reference_panel_smoke;
use super::local_vcf_qc_smoke;
use super::local_vcf_roh_smoke;
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use super::local_vcf_stats_smoke;
use super::readiness::tool_smoke_support::{path_relative_to_repo, repo_relative_path};
use super::readiness::vcf_local_container_smoke::{
    collect_vcf_local_container_smoke_rows, VcfLocalContainerSmokeRow,
};
use super::vcf_stage_families::{VcfStageFamily, VCF_STAGE_FAMILIES};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_MICRO_SMOKE_SUMMARY_PATH: &str =
    "runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json";
const VCF_MICRO_SMOKE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_micro_smoke_subset.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VcfMicroSmokeExecutionStatus {
    LocalSmoke,
    ContainerNeeded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfMicroSmokeFamilyRow {
    pub(crate) family_id: String,
    pub(crate) surface_label: String,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) representative_stage_id: String,
    pub(crate) representative_tool_id: String,
    pub(crate) registered_binary: String,
    pub(crate) smoke_tool_id: String,
    pub(crate) smoke_path_kind: String,
    pub(crate) smoke_runtime: String,
    pub(crate) smoke_command: String,
    pub(crate) smoke_support_path: Option<String>,
    pub(crate) execution_status: VcfMicroSmokeExecutionStatus,
    pub(crate) reason: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) evidence_format: Option<String>,
    pub(crate) parsed_schema_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfMicroSmokeSubsetReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) family_count: usize,
    pub(crate) local_smoke_count: usize,
    pub(crate) container_needed_count: usize,
    pub(crate) unavailable_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<VcfMicroSmokeFamilyRow>,
}

pub(crate) fn run_vcf_micro_smoke_subset(
    args: &parse::BenchLocalRunVcfMicroSmokeSubsetArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_micro_smoke_subset(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_MICRO_SMOKE_SUMMARY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_micro_smoke_subset(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfMicroSmokeSubsetReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let smoke_rows = collect_vcf_local_container_smoke_rows(repo_root)?;
    let mut rows = Vec::with_capacity(VCF_STAGE_FAMILIES.len());
    for family in VCF_STAGE_FAMILIES {
        let representative = select_family_representative(family, &smoke_rows)?;
        rows.push(materialize_family_row(repo_root, family, representative)?);
    }

    let local_smoke_count = rows
        .iter()
        .filter(|row| row.execution_status == VcfMicroSmokeExecutionStatus::LocalSmoke)
        .count();
    let container_needed_count = rows
        .iter()
        .filter(|row| row.execution_status == VcfMicroSmokeExecutionStatus::ContainerNeeded)
        .count();
    let unavailable_count = rows
        .iter()
        .filter(|row| row.execution_status == VcfMicroSmokeExecutionStatus::Unavailable)
        .count();

    let report = VcfMicroSmokeSubsetReport {
        schema_version: VCF_MICRO_SMOKE_SUMMARY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        family_count: rows.len(),
        local_smoke_count,
        container_needed_count,
        unavailable_count,
        passes_behavior_test: false,
        rows,
    };
    let report = ensure_vcf_micro_smoke_subset_contract(repo_root, report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn select_family_representative<'a>(
    family: &VcfStageFamily,
    smoke_rows: &'a [VcfLocalContainerSmokeRow],
) -> Result<&'a VcfLocalContainerSmokeRow> {
    let mut matching_rows = smoke_rows
        .iter()
        .filter(|row| family.stage_ids.contains(&row.stage_id.as_str()))
        .collect::<Vec<_>>();
    if matching_rows.is_empty() {
        bail!(
            "VCF micro smoke subset found no retained smoke rows for family `{}`",
            family.family_id
        );
    }

    matching_rows.sort_by(|left, right| {
        family_priority(left)
            .cmp(&family_priority(right))
            .then_with(|| {
                family_stage_order(family, &left.stage_id)
                    .cmp(&family_stage_order(family, &right.stage_id))
            })
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    matching_rows.into_iter().next().ok_or_else(|| {
        anyhow!(
            "VCF micro smoke subset could not choose a representative for family `{}`",
            family.family_id
        )
    })
}

fn materialize_family_row(
    repo_root: &Path,
    family: &VcfStageFamily,
    representative: &VcfLocalContainerSmokeRow,
) -> Result<VcfMicroSmokeFamilyRow> {
    let support_path = normalize_optional_string(&representative.smoke_support_path);
    let default_tool_id = default_tool_id(&representative.stage_id);

    let (execution_status, evidence_path, evidence_format, parsed_schema_version) =
        if default_tool_id.as_deref() == Some(representative.tool_id.as_str())
            && representative.smoke_path_kind == "host_stage_smoke"
        {
            let manifest_path = run_local_vcf_smoke_manifest(
                repo_root,
                &representative.stage_id,
                &representative.tool_id,
            )
            .with_context(|| {
                format!(
                    "materialize VCF micro smoke subset artifact for family `{}` via `{}` / `{}`",
                    family.family_id, representative.stage_id, representative.tool_id
                )
            })?;
            let evidence_path = path_relative_to_repo(repo_root, &manifest_path);
            let (evidence_format, parsed_schema_version) =
                describe_evidence_artifact(&manifest_path)?;
            (
                VcfMicroSmokeExecutionStatus::LocalSmoke,
                Some(evidence_path),
                Some(evidence_format),
                parsed_schema_version,
            )
        } else if matches!(
            representative.smoke_path_kind.as_str(),
            "docker_container_smoke" | "apptainer_container_smoke"
        ) {
            (VcfMicroSmokeExecutionStatus::ContainerNeeded, None, None, None)
        } else {
            (VcfMicroSmokeExecutionStatus::Unavailable, None, None, None)
        };

    Ok(VcfMicroSmokeFamilyRow {
        family_id: family.family_id.to_string(),
        surface_label: family.surface_label.to_string(),
        stage_ids: family.stage_ids.iter().map(|stage_id| (*stage_id).to_string()).collect(),
        representative_stage_id: representative.stage_id.clone(),
        representative_tool_id: representative.tool_id.clone(),
        registered_binary: representative.registered_binary.clone(),
        smoke_tool_id: representative.smoke_tool_id.clone(),
        smoke_path_kind: representative.smoke_path_kind.clone(),
        smoke_runtime: representative.smoke_runtime.clone(),
        smoke_command: representative.smoke_command.clone(),
        smoke_support_path: support_path,
        execution_status,
        reason: representative.reason.clone(),
        evidence_path,
        evidence_format,
        parsed_schema_version,
    })
}

fn ensure_vcf_micro_smoke_subset_contract(
    repo_root: &Path,
    mut report: VcfMicroSmokeSubsetReport,
) -> Result<VcfMicroSmokeSubsetReport> {
    if report.family_count != VCF_STAGE_FAMILIES.len()
        || report.rows.len() != VCF_STAGE_FAMILIES.len()
    {
        return Err(anyhow!(
            "VCF micro smoke subset must keep exactly {} family rows, found family_count={} rows={}",
            VCF_STAGE_FAMILIES.len(),
            report.family_count,
            report.rows.len()
        ));
    }
    if report.local_smoke_count + report.container_needed_count + report.unavailable_count
        != report.family_count
    {
        return Err(anyhow!(
            "VCF micro smoke subset status counts must sum to family_count, found local={} container={} unavailable={} family_count={}",
            report.local_smoke_count,
            report.container_needed_count,
            report.unavailable_count,
            report.family_count
        ));
    }

    let family_ids = report.rows.iter().map(|row| row.family_id.as_str()).collect::<BTreeSet<_>>();
    let expected_family_ids =
        VCF_STAGE_FAMILIES.iter().map(|family| family.family_id).collect::<BTreeSet<_>>();
    if family_ids != expected_family_ids {
        return Err(anyhow!(
            "VCF micro smoke subset family ids drifted: observed={family_ids:?} expected={expected_family_ids:?}"
        ));
    }

    for family in VCF_STAGE_FAMILIES {
        let row =
            report.rows.iter().find(|row| row.family_id == family.family_id).ok_or_else(|| {
                anyhow!("VCF micro smoke subset is missing family `{}`", family.family_id)
            })?;
        let expected_stage_ids =
            family.stage_ids.iter().map(|stage_id| (*stage_id).to_string()).collect::<Vec<_>>();
        if row.stage_ids != expected_stage_ids {
            return Err(anyhow!(
                "VCF micro smoke subset family `{}` drifted stage ids: observed={:?} expected={:?}",
                family.family_id,
                row.stage_ids,
                expected_stage_ids
            ));
        }
        if !family.stage_ids.contains(&row.representative_stage_id.as_str()) {
            return Err(anyhow!(
                "VCF micro smoke subset family `{}` chose stage `{}` outside its family slice",
                family.family_id,
                row.representative_stage_id
            ));
        }
        if row.smoke_command.trim().is_empty() {
            return Err(anyhow!(
                "VCF micro smoke subset family `{}` is missing a smoke command",
                family.family_id
            ));
        }
        if row.reason.trim().is_empty() {
            return Err(anyhow!(
                "VCF micro smoke subset family `{}` is missing a selection rationale",
                family.family_id
            ));
        }
        if let Some(support_path) = &row.smoke_support_path {
            let absolute_support_path = repo_root.join(support_path);
            if !absolute_support_path.exists() {
                return Err(anyhow!(
                    "VCF micro smoke subset family `{}` support path `{support_path}` is missing",
                    family.family_id
                ));
            }
        }

        match row.execution_status {
            VcfMicroSmokeExecutionStatus::LocalSmoke => {
                if row.smoke_path_kind != "host_stage_smoke" || row.smoke_runtime != "host" {
                    return Err(anyhow!(
                        "VCF micro smoke subset family `{}` must keep host smoke metadata for local execution, found kind=`{}` runtime=`{}`",
                        family.family_id,
                        row.smoke_path_kind,
                        row.smoke_runtime
                    ));
                }
                let evidence_path = row.evidence_path.as_ref().ok_or_else(|| {
                    anyhow!(
                        "VCF micro smoke subset family `{}` is missing evidence_path for local smoke",
                        family.family_id
                    )
                })?;
                let absolute_evidence_path = repo_root.join(evidence_path);
                if !absolute_evidence_path.is_file() {
                    return Err(anyhow!(
                        "VCF micro smoke subset family `{}` evidence path `{evidence_path}` is missing",
                        family.family_id
                    ));
                }
                if row.evidence_format.is_none() {
                    return Err(anyhow!(
                        "VCF micro smoke subset family `{}` is missing evidence_format for local smoke",
                        family.family_id
                    ));
                }
            }
            VcfMicroSmokeExecutionStatus::ContainerNeeded => {
                if row.evidence_path.is_some() || row.evidence_format.is_some() {
                    return Err(anyhow!(
                        "VCF micro smoke subset family `{}` must not claim local evidence for container-needed status",
                        family.family_id
                    ));
                }
                if !matches!(
                    row.smoke_path_kind.as_str(),
                    "docker_container_smoke" | "apptainer_container_smoke"
                ) {
                    return Err(anyhow!(
                        "VCF micro smoke subset family `{}` must keep a container smoke path kind for container-needed status, found `{}`",
                        family.family_id,
                        row.smoke_path_kind
                    ));
                }
            }
            VcfMicroSmokeExecutionStatus::Unavailable => {
                if row.evidence_path.is_some() || row.evidence_format.is_some() {
                    return Err(anyhow!(
                        "VCF micro smoke subset family `{}` must not claim evidence for unavailable status",
                        family.family_id
                    ));
                }
            }
        }
    }

    let calling = report
        .rows
        .iter()
        .find(|row| row.family_id == "vcf.calling")
        .ok_or_else(|| anyhow!("VCF micro smoke subset is missing the `vcf.calling` family"))?;
    if calling.execution_status != VcfMicroSmokeExecutionStatus::LocalSmoke {
        return Err(anyhow!(
            "VCF micro smoke subset must report `vcf.calling` as local_smoke, found {:?}",
            calling.execution_status
        ));
    }

    report.passes_behavior_test = true;
    Ok(report)
}

fn family_priority(row: &VcfLocalContainerSmokeRow) -> (u8, u8, String) {
    if default_tool_id(&row.stage_id).as_deref() == Some(row.tool_id.as_str())
        && row.smoke_path_kind == "host_stage_smoke"
    {
        return (0, 0, row.tool_id.clone());
    }
    if default_tool_id(&row.stage_id).as_deref() == Some(row.tool_id.as_str()) {
        return (1, smoke_priority(&row.smoke_path_kind), row.tool_id.clone());
    }
    (2, smoke_priority(&row.smoke_path_kind), row.tool_id.clone())
}

fn default_tool_id(stage_id: &str) -> Option<String> {
    build_vcf_stage_matrix_rows()
        .ok()?
        .into_iter()
        .find(|row| row.stage_id == stage_id)
        .map(|row| row.tool_id)
}

fn family_stage_order(family: &VcfStageFamily, stage_id: &str) -> usize {
    family
        .stage_ids
        .iter()
        .position(|candidate| *candidate == stage_id)
        .unwrap_or(family.stage_ids.len())
}

fn smoke_priority(smoke_path_kind: &str) -> u8 {
    match smoke_path_kind {
        "host_stage_smoke" => 0,
        "apptainer_container_smoke" => 1,
        "docker_container_smoke" => 2,
        _ => 3,
    }
}

fn normalize_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn describe_evidence_artifact(artifact_path: &Path) -> Result<(String, Option<String>)> {
    match artifact_path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => {
            let payload = fs::read_to_string(artifact_path)
                .with_context(|| format!("read {}", artifact_path.display()))?;
            let parsed: Value = serde_json::from_str(&payload)
                .with_context(|| format!("parse {}", artifact_path.display()))?;
            let schema_version = parsed
                .get("schema_version")
                .and_then(Value::as_str)
                .map(std::string::ToString::to_string);
            Ok(("json".to_string(), schema_version))
        }
        Some("vcf") => Ok(("vcf".to_string(), None)),
        Some("gz") => {
            let path = artifact_path.to_string_lossy();
            if path.ends_with(".vcf.gz") {
                Ok(("vcf.gz".to_string(), None))
            } else {
                Ok(("gz".to_string(), None))
            }
        }
        Some("tsv") => Ok(("tsv".to_string(), None)),
        Some(other) => Ok((other.to_string(), None)),
        None => Ok(("path".to_string(), None)),
    }
}

fn run_local_vcf_smoke_manifest(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
) -> Result<PathBuf> {
    let manifest_path = match stage_id {
        "vcf.admixture" => {
            local_vcf_admixture_smoke::run_local_vcf_admixture_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.call" => {
            local_vcf_call_smoke::run_local_vcf_call_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.call_diploid" => {
            local_vcf_call_diploid_smoke::run_local_vcf_call_diploid_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.call_gl" => {
            local_vcf_call_gl_smoke::run_local_vcf_call_gl_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.call_pseudohaploid" => {
            local_vcf_call_pseudohaploid_smoke::run_local_vcf_call_pseudohaploid_smoke(
                repo_root, tool_id,
            )?
            .stage_result_manifest_path
        }
        "vcf.damage_filter" => {
            local_vcf_damage_filter_smoke::run_local_vcf_damage_filter_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.demography" => {
            local_vcf_demography_smoke::run_local_vcf_demography_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.filter" => {
            local_vcf_filter_smoke::run_local_vcf_filter_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.gl_propagation" => {
            local_vcf_gl_propagation_smoke::run_local_vcf_gl_propagation_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.ibd" => {
            local_vcf_ibd_smoke::run_local_vcf_ibd_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.imputation_metrics" => {
            local_vcf_imputation_metrics_smoke::run_local_vcf_imputation_metrics_smoke(
                repo_root, tool_id,
            )?
            .stage_result_manifest_path
        }
        "vcf.impute" => {
            local_vcf_impute_smoke::run_local_vcf_impute_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.pca" => {
            local_vcf_pca_smoke::run_local_vcf_pca_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.phasing" => {
            local_vcf_phasing_smoke::run_local_vcf_phasing_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.population_structure" => {
            local_vcf_population_structure_smoke::run_local_vcf_population_structure_smoke(
                repo_root, tool_id,
            )?
            .stage_result_manifest_path
        }
        "vcf.postprocess" => {
            local_vcf_postprocess_smoke::run_local_vcf_postprocess_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.prepare_reference_panel" => {
            local_vcf_prepare_reference_panel_smoke::run_local_vcf_prepare_reference_panel_smoke(
                repo_root, tool_id,
            )?
            .stage_result_manifest_path
        }
        "vcf.qc" => {
            local_vcf_qc_smoke::run_local_vcf_qc_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.roh" => {
            local_vcf_roh_smoke::run_local_vcf_roh_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        "vcf.stats" => {
            local_vcf_stats_smoke::run_local_vcf_stats_smoke(repo_root, tool_id)?
                .stage_result_manifest_path
        }
        other => bail!(
            "VCF micro smoke subset does not have a governed local smoke wrapper for `{other}`"
        ),
    };
    Ok(repo_root.join(manifest_path))
}
