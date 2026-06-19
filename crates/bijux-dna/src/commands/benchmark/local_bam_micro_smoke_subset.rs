use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::stage_api::default_tool_for_stage;
use serde::Serialize;
use serde_json::Value;

use super::bam_stage_families::{BamStageFamily, BAM_STAGE_FAMILIES};
use super::local_bam_stage_smoke::run_local_bam_stage_smoke;
use super::readiness::bam_local_container_smoke::{
    collect_bam_local_container_smoke_rows, BamLocalContainerSmokeRow,
};
use super::readiness::tool_smoke_support::{path_relative_to_repo, repo_relative_path};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH: &str =
    "runs/bench/micro/bam/MICRO_BAM_SUMMARY.json";
const BAM_MICRO_SMOKE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bench.local_bam_micro_smoke_subset.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamMicroSmokeExecutionStatus {
    LocalSmoke,
    ContainerNeeded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamMicroSmokeFamilyRow {
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
    pub(crate) execution_status: BamMicroSmokeExecutionStatus,
    pub(crate) reason: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) evidence_format: Option<String>,
    pub(crate) parsed_schema_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamMicroSmokeSubsetReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) family_count: usize,
    pub(crate) local_smoke_count: usize,
    pub(crate) container_needed_count: usize,
    pub(crate) unavailable_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<BamMicroSmokeFamilyRow>,
}

pub(crate) fn run_bam_micro_smoke_subset(
    args: &parse::BenchLocalRunBamMicroSmokeSubsetArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_micro_smoke_subset(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_micro_smoke_subset(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamMicroSmokeSubsetReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let smoke_rows = collect_bam_local_container_smoke_rows(repo_root)?;
    let mut rows = Vec::with_capacity(BAM_STAGE_FAMILIES.len());
    for family in BAM_STAGE_FAMILIES {
        let representative = select_family_representative(family, &smoke_rows)?;
        rows.push(materialize_family_row(repo_root, family, representative)?);
    }

    let local_smoke_count = rows
        .iter()
        .filter(|row| row.execution_status == BamMicroSmokeExecutionStatus::LocalSmoke)
        .count();
    let container_needed_count = rows
        .iter()
        .filter(|row| row.execution_status == BamMicroSmokeExecutionStatus::ContainerNeeded)
        .count();
    let unavailable_count = rows
        .iter()
        .filter(|row| row.execution_status == BamMicroSmokeExecutionStatus::Unavailable)
        .count();

    let report = BamMicroSmokeSubsetReport {
        schema_version: BAM_MICRO_SMOKE_SUMMARY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        family_count: rows.len(),
        local_smoke_count,
        container_needed_count,
        unavailable_count,
        passes_behavior_test: false,
        rows,
    };
    let report = ensure_bam_micro_smoke_subset_contract(repo_root, report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn select_family_representative<'a>(
    family: &BamStageFamily,
    smoke_rows: &'a [BamLocalContainerSmokeRow],
) -> Result<&'a BamLocalContainerSmokeRow> {
    let mut matching_rows = smoke_rows
        .iter()
        .filter(|row| family.stage_ids.contains(&row.stage_id.as_str()))
        .collect::<Vec<_>>();
    if matching_rows.is_empty() {
        bail!(
            "BAM micro smoke subset found no retained smoke rows for family `{}`",
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
            "BAM micro smoke subset could not choose a representative for family `{}`",
            family.family_id
        )
    })
}

fn materialize_family_row(
    repo_root: &Path,
    family: &BamStageFamily,
    representative: &BamLocalContainerSmokeRow,
) -> Result<BamMicroSmokeFamilyRow> {
    let support_path = normalize_optional_string(&representative.smoke_support_path);
    let (execution_status, evidence_path, evidence_format, parsed_schema_version) =
        match representative.smoke_path_kind.as_str() {
            "host_stage_smoke" => {
                let artifact_path = run_local_bam_stage_smoke(repo_root, &representative.stage_id)
                    .with_context(|| {
                        format!(
                            "materialize BAM micro smoke subset artifact for family `{}` via stage `{}`",
                            family.family_id, representative.stage_id
                        )
                    })?;
                let evidence_path = path_relative_to_repo(repo_root, &artifact_path);
                let (evidence_format, parsed_schema_version) =
                    describe_evidence_artifact(&artifact_path)?;
                (
                    BamMicroSmokeExecutionStatus::LocalSmoke,
                    Some(evidence_path),
                    Some(evidence_format),
                    parsed_schema_version,
                )
            }
            "docker_container_smoke" | "apptainer_container_smoke" => {
                (BamMicroSmokeExecutionStatus::ContainerNeeded, None, None, None)
            }
            _ => (BamMicroSmokeExecutionStatus::Unavailable, None, None, None),
        };

    Ok(BamMicroSmokeFamilyRow {
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

fn ensure_bam_micro_smoke_subset_contract(
    repo_root: &Path,
    mut report: BamMicroSmokeSubsetReport,
) -> Result<BamMicroSmokeSubsetReport> {
    if report.family_count != BAM_STAGE_FAMILIES.len()
        || report.rows.len() != BAM_STAGE_FAMILIES.len()
    {
        return Err(anyhow!(
            "BAM micro smoke subset must keep exactly {} family rows, found family_count={} rows={}",
            BAM_STAGE_FAMILIES.len(),
            report.family_count,
            report.rows.len()
        ));
    }
    if report.local_smoke_count + report.container_needed_count + report.unavailable_count
        != report.family_count
    {
        return Err(anyhow!(
            "BAM micro smoke subset status counts must sum to family_count, found local={} container={} unavailable={} family_count={}",
            report.local_smoke_count,
            report.container_needed_count,
            report.unavailable_count,
            report.family_count
        ));
    }

    let family_ids = report.rows.iter().map(|row| row.family_id.as_str()).collect::<BTreeSet<_>>();
    let expected_family_ids =
        BAM_STAGE_FAMILIES.iter().map(|family| family.family_id).collect::<BTreeSet<_>>();
    if family_ids != expected_family_ids {
        return Err(anyhow!(
            "BAM micro smoke subset family ids drifted: observed={family_ids:?} expected={expected_family_ids:?}"
        ));
    }

    for family in BAM_STAGE_FAMILIES {
        let row =
            report.rows.iter().find(|row| row.family_id == family.family_id).ok_or_else(|| {
                anyhow!("BAM micro smoke subset is missing family `{}`", family.family_id)
            })?;
        let expected_stage_ids =
            family.stage_ids.iter().map(|stage_id| (*stage_id).to_string()).collect::<Vec<_>>();
        if row.stage_ids != expected_stage_ids {
            return Err(anyhow!(
                "BAM micro smoke subset family `{}` drifted stage ids: observed={:?} expected={:?}",
                family.family_id,
                row.stage_ids,
                expected_stage_ids
            ));
        }
        if !family.stage_ids.contains(&row.representative_stage_id.as_str()) {
            return Err(anyhow!(
                "BAM micro smoke subset family `{}` chose stage `{}` outside its family slice",
                family.family_id,
                row.representative_stage_id
            ));
        }
        if row.smoke_command.trim().is_empty() {
            return Err(anyhow!(
                "BAM micro smoke subset family `{}` is missing a smoke command",
                family.family_id
            ));
        }
        if row.reason.trim().is_empty() {
            return Err(anyhow!(
                "BAM micro smoke subset family `{}` is missing a selection rationale",
                family.family_id
            ));
        }
        if let Some(support_path) = &row.smoke_support_path {
            let absolute_support_path = repo_root.join(support_path);
            if !absolute_support_path.exists() {
                return Err(anyhow!(
                    "BAM micro smoke subset family `{}` support path `{support_path}` is missing",
                    family.family_id
                ));
            }
        }

        match row.execution_status {
            BamMicroSmokeExecutionStatus::LocalSmoke => {
                if row.smoke_path_kind != "host_stage_smoke" || row.smoke_runtime != "host" {
                    return Err(anyhow!(
                        "BAM micro smoke subset family `{}` must keep host smoke metadata for local execution, found kind=`{}` runtime=`{}`",
                        family.family_id,
                        row.smoke_path_kind,
                        row.smoke_runtime
                    ));
                }
                let evidence_path = row.evidence_path.as_ref().ok_or_else(|| {
                    anyhow!(
                        "BAM micro smoke subset family `{}` is missing evidence_path for local smoke",
                        family.family_id
                    )
                })?;
                let absolute_evidence_path = repo_root.join(evidence_path);
                if !absolute_evidence_path.is_file() {
                    return Err(anyhow!(
                        "BAM micro smoke subset family `{}` evidence path `{evidence_path}` is missing",
                        family.family_id
                    ));
                }
                if row.evidence_format.is_none() {
                    return Err(anyhow!(
                        "BAM micro smoke subset family `{}` is missing evidence_format for local smoke",
                        family.family_id
                    ));
                }
            }
            BamMicroSmokeExecutionStatus::ContainerNeeded => {
                if row.evidence_path.is_some() || row.evidence_format.is_some() {
                    return Err(anyhow!(
                        "BAM micro smoke subset family `{}` must not claim local evidence for container-needed status",
                        family.family_id
                    ));
                }
                if !matches!(
                    row.smoke_path_kind.as_str(),
                    "docker_container_smoke" | "apptainer_container_smoke"
                ) {
                    return Err(anyhow!(
                        "BAM micro smoke subset family `{}` must keep a container smoke path kind for container-needed status, found `{}`",
                        family.family_id,
                        row.smoke_path_kind
                    ));
                }
            }
            BamMicroSmokeExecutionStatus::Unavailable => {
                if row.evidence_path.is_some() || row.evidence_format.is_some() {
                    return Err(anyhow!(
                        "BAM micro smoke subset family `{}` must not claim evidence for unavailable status",
                        family.family_id
                    ));
                }
            }
        }
    }

    let align = report
        .rows
        .iter()
        .find(|row| row.family_id == "bam.align")
        .ok_or_else(|| anyhow!("BAM micro smoke subset is missing the `bam.align` family"))?;
    if align.execution_status != BamMicroSmokeExecutionStatus::ContainerNeeded {
        return Err(anyhow!(
            "BAM micro smoke subset must report `bam.align` as container-needed, found {:?}",
            align.execution_status
        ));
    }

    report.passes_behavior_test = true;
    Ok(report)
}

fn smoke_priority(smoke_path_kind: &str) -> u8 {
    match smoke_path_kind {
        "host_stage_smoke" => 0,
        "docker_container_smoke" | "apptainer_container_smoke" => 1,
        _ => 2,
    }
}

fn family_priority(row: &BamLocalContainerSmokeRow) -> (u8, u8, String) {
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
    BamStage::try_from(stage_id).ok().map(|stage| default_tool_for_stage(stage).to_string())
}

fn family_stage_order(family: &BamStageFamily, stage_id: &str) -> usize {
    family
        .stage_ids
        .iter()
        .position(|family_stage_id| *family_stage_id == stage_id)
        .unwrap_or(usize::MAX)
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
    match artifact_path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("json") => {
            let payload = read_json_document(artifact_path)?;
            Ok(("json".to_string(), Some(json_string_field(&payload, "schema_version")?)))
        }
        Some("tsv") => Ok(("tsv".to_string(), None)),
        _ => Ok(("artifact".to_string(), None)),
    }
}

fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn json_string_field(document: &Value, field: &str) -> Result<String> {
    document
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("JSON document is missing string field `{field}`"))
}
