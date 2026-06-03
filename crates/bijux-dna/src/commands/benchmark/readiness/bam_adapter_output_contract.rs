use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::stage_specs::stage_output_ids_in_manifest_order;
use bijux_dna_planner_bam::stage_api::load_bam_domain_tool_stage_output_contract;
use serde::Serialize;

use super::tool_serving_map::{
    render_bam_tool_serving_map, ToolServingMapRow, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
};
use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH: &str =
    "target/bench-readiness/bam-adapter-output-contract.tsv";
const BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_adapter_output_contract.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamAdapterOutputContractStatus {
    Complete,
    Incomplete,
    MissingAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamAdapterOutputContractRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) adapter_status: String,
    pub(crate) output_contract_status: BamAdapterOutputContractStatus,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) stage_expected_artifact_ids: Vec<String>,
    pub(crate) declared_output_ids: Vec<String>,
    pub(crate) execution_expected_output_ids: Vec<String>,
    pub(crate) missing_declarations: Vec<String>,
    pub(crate) raw_output_artifact_ids: Vec<String>,
    pub(crate) normalized_metrics_output_id: Option<String>,
    pub(crate) stdout_path_template: Option<String>,
    pub(crate) stderr_path_template: Option<String>,
    pub(crate) stage_result_manifest_path_template: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamAdapterOutputContractReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) adapter_row_count: usize,
    pub(crate) complete_adapter_row_count: usize,
    pub(crate) incomplete_adapter_row_count: usize,
    pub(crate) missing_adapter_row_count: usize,
    pub(crate) rows: Vec<BamAdapterOutputContractRow>,
}

pub(crate) fn run_render_bam_adapter_output_contract(
    args: &parse::BenchReadinessRenderBamAdapterOutputContractArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_adapter_output_contract(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_adapter_output_contract(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamAdapterOutputContractReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let tool_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;
    let rows = tool_map
        .rows
        .iter()
        .map(|row| render_output_contract_row(repo_root, row))
        .collect::<Vec<_>>();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_adapter_output_contract_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let adapter_row_count = rows.iter().filter(|row| row_has_adapter(&row.adapter_status)).count();
    let complete_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row_has_adapter(&row.adapter_status)
                && row.output_contract_status == BamAdapterOutputContractStatus::Complete
        })
        .count();
    let incomplete_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row_has_adapter(&row.adapter_status)
                && row.output_contract_status == BamAdapterOutputContractStatus::Incomplete
        })
        .count();
    let missing_adapter_row_count = rows
        .iter()
        .filter(|row| row.output_contract_status == BamAdapterOutputContractStatus::MissingAdapter)
        .count();

    Ok(BamAdapterOutputContractReport {
        schema_version: BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        adapter_row_count,
        complete_adapter_row_count,
        incomplete_adapter_row_count,
        missing_adapter_row_count,
        rows,
    })
}

fn render_output_contract_row(
    repo_root: &Path,
    row: &ToolServingMapRow,
) -> BamAdapterOutputContractRow {
    let stage_output_ids = stage_output_ids_in_manifest_order(&row.stage_id).unwrap_or_default();
    if !row_has_adapter(&row.adapter_status) {
        return BamAdapterOutputContractRow {
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            adapter_status: row.adapter_status.clone(),
            output_contract_status: BamAdapterOutputContractStatus::MissingAdapter,
            stage_output_ids,
            stage_expected_artifact_ids: Vec::new(),
            declared_output_ids: Vec::new(),
            execution_expected_output_ids: Vec::new(),
            missing_declarations: vec!["adapter".to_string()],
            raw_output_artifact_ids: Vec::new(),
            normalized_metrics_output_id: None,
            stdout_path_template: None,
            stderr_path_template: None,
            stage_result_manifest_path_template: None,
            reason: format!(
                "row `{}` / `{}` has no runnable or plannable BAM adapter (`{}`)",
                row.stage_id, row.tool_id, row.adapter_status
            ),
        };
    }

    let stage_id = StageId::new(row.stage_id.clone());
    let tool_id = ToolId::new(row.tool_id.clone());
    match load_bam_domain_tool_stage_output_contract(repo_root, &stage_id, &tool_id) {
        Ok(contract) => {
            let declared_output_ids = contract.declared_output_ids;
            let execution_expected_output_ids = contract.execution_expected_output_ids;
            let stage_expected_artifact_ids = contract.stage_expected_artifact_ids;
            let normalized_metrics_output_id =
                normalized_metrics_output_id(&stage_output_ids, &stage_expected_artifact_ids);
            let raw_output_artifact_ids = raw_output_artifact_ids(
                &stage_expected_artifact_ids,
                normalized_metrics_output_id.as_deref(),
            );
            let missing_declarations = collect_missing_declarations(
                &stage_expected_artifact_ids,
                &declared_output_ids,
                &execution_expected_output_ids,
                normalized_metrics_output_id.as_deref(),
            );
            let output_contract_status = if missing_declarations.is_empty() {
                BamAdapterOutputContractStatus::Complete
            } else {
                BamAdapterOutputContractStatus::Incomplete
            };
            let path_template_root = path_template_root(&row.stage_id, &row.tool_id);
            let stdout_path_template = Some(format!("{path_template_root}/stdout.log"));
            let stderr_path_template = Some(format!("{path_template_root}/stderr.log"));
            let stage_result_manifest_path_template =
                Some(format!("{path_template_root}/stage-result.json"));
            let reason = if missing_declarations.is_empty() {
                format!(
                    "row `{}` / `{}` declares all governed BAM stage outputs, a normalized metrics artifact, and deterministic stdout/stderr/result-manifest paths",
                    row.stage_id, row.tool_id
                )
            } else {
                format!(
                    "row `{}` / `{}` is missing governed adapter output declarations: {}",
                    row.stage_id,
                    row.tool_id,
                    missing_declarations.join(", ")
                )
            };

            BamAdapterOutputContractRow {
                tool_id: row.tool_id.clone(),
                stage_id: row.stage_id.clone(),
                adapter_status: row.adapter_status.clone(),
                output_contract_status,
                stage_output_ids,
                stage_expected_artifact_ids,
                declared_output_ids,
                execution_expected_output_ids,
                missing_declarations,
                raw_output_artifact_ids,
                normalized_metrics_output_id,
                stdout_path_template,
                stderr_path_template,
                stage_result_manifest_path_template,
                reason,
            }
        }
        Err(error) => BamAdapterOutputContractRow {
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            adapter_status: row.adapter_status.clone(),
            output_contract_status: BamAdapterOutputContractStatus::Incomplete,
            stage_output_ids,
            stage_expected_artifact_ids: Vec::new(),
            declared_output_ids: Vec::new(),
            execution_expected_output_ids: Vec::new(),
            missing_declarations: vec!["stage_contract".to_string()],
            raw_output_artifact_ids: Vec::new(),
            normalized_metrics_output_id: None,
            stdout_path_template: None,
            stderr_path_template: None,
            stage_result_manifest_path_template: None,
            reason: format!(
                "row `{}` / `{}` could not load the governed BAM output contract: {error}",
                row.stage_id, row.tool_id
            ),
        },
    }
}

fn row_has_adapter(adapter_status: &str) -> bool {
    matches!(adapter_status, "runnable" | "plannable")
}

fn normalized_metrics_output_id(
    stage_output_ids: &[String],
    stage_expected_artifact_ids: &[String],
) -> Option<String> {
    let mut candidates = stage_expected_artifact_ids
        .iter()
        .filter(|artifact_id| {
            stage_output_ids.iter().any(|stage_output_id| stage_output_id == *artifact_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates = stage_output_ids.to_vec();
    }

    for preferred in [
        "validation_report",
        "damage_report",
        "authenticity_report",
        "contamination_report",
        "sex_report",
        "bias_report",
        "genotyping_report",
        "kinship_report",
        "haplogroups",
        "coverage_summary",
        "endogenous_report",
        "duplication_report",
        "insert_size_report",
        "gc_bias_report",
        "align_metrics",
        "recal_report",
        "report_json",
        "summary",
        "stage_metrics",
    ] {
        if candidates.iter().any(|artifact_id| artifact_id == preferred) {
            return Some(preferred.to_string());
        }
    }

    candidates.into_iter().find(|artifact_id| {
        artifact_id.ends_with("_report")
            || artifact_id.ends_with("_report_json")
            || artifact_id.ends_with("_estimate")
            || artifact_id.ends_with("_summary")
            || artifact_id.ends_with("_json")
            || artifact_id.ends_with("_tsv")
    })
}

fn raw_output_artifact_ids(
    stage_expected_artifact_ids: &[String],
    normalized_metrics_output_id: Option<&str>,
) -> Vec<String> {
    stage_expected_artifact_ids
        .iter()
        .filter(|artifact_id| Some(artifact_id.as_str()) != normalized_metrics_output_id)
        .cloned()
        .collect()
}

fn collect_missing_declarations(
    stage_expected_artifact_ids: &[String],
    declared_output_ids: &[String],
    execution_expected_output_ids: &[String],
    normalized_metrics_output_id: Option<&str>,
) -> Vec<String> {
    let declared_output_ids_set = declared_output_ids.iter().cloned().collect::<BTreeSet<_>>();
    let execution_expected_output_ids_set =
        execution_expected_output_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut missing = Vec::new();

    if stage_expected_artifact_ids.is_empty() {
        missing.push("stage_contract.expected_artifacts".to_string());
    }
    for artifact_id in stage_expected_artifact_ids {
        if !declared_output_ids_set.contains(artifact_id) {
            missing.push(format!("tool.outputs:{artifact_id}"));
        }
        if !execution_expected_output_ids_set.contains(artifact_id) {
            missing.push(format!("execution_contract.expected_outputs:{artifact_id}"));
        }
    }
    if normalized_metrics_output_id.is_none() {
        missing.push("normalized_metrics_output".to_string());
    }
    missing
}

fn path_template_root(stage_id: &str, tool_id: &str) -> String {
    format!(
        "target/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        LOCAL_SLURM_DRY_RUN_RUN_ID, "{fixture_scope}", stage_id, "{sample_scope}", tool_id
    )
}

fn render_bam_adapter_output_contract_tsv(rows: &[BamAdapterOutputContractRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(output_contract_status_label(row.output_contract_status)),
            sanitize_tsv(&row.stage_output_ids.join(",")),
            sanitize_tsv(&row.stage_expected_artifact_ids.join(",")),
            sanitize_tsv(&row.declared_output_ids.join(",")),
            sanitize_tsv(&row.execution_expected_output_ids.join(",")),
            sanitize_tsv(&row.raw_output_artifact_ids.join(",")),
            sanitize_tsv(row.normalized_metrics_output_id.as_deref().unwrap_or("")),
            sanitize_tsv(row.stdout_path_template.as_deref().unwrap_or("")),
            sanitize_tsv(row.stderr_path_template.as_deref().unwrap_or("")),
            sanitize_tsv(row.stage_result_manifest_path_template.as_deref().unwrap_or("")),
            sanitize_tsv(&row.missing_declarations.join(",")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn output_contract_status_label(status: BamAdapterOutputContractStatus) -> &'static str {
    match status {
        BamAdapterOutputContractStatus::Complete => "complete",
        BamAdapterOutputContractStatus::Incomplete => "incomplete",
        BamAdapterOutputContractStatus::MissingAdapter => "missing_adapter",
    }
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_adapter_output_contract, BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
        DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn bam_adapter_output_contract_reports_governed_adapter_metadata() {
        let root = repo_root();
        let report = render_bam_adapter_output_contract(
            &root,
            PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH),
        )
        .expect("render BAM adapter output contract");

        assert_eq!(report.schema_version, BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION);
        assert_eq!(report.row_count, 51);
        assert_eq!(report.adapter_row_count, 48);
        assert_eq!(report.complete_adapter_row_count, 48);
        assert_eq!(report.incomplete_adapter_row_count, 0);
        assert_eq!(report.missing_adapter_row_count, 3);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && super::output_contract_status_label(row.output_contract_status) == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("validation_report")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.damage"
                && super::output_contract_status_label(row.output_contract_status) == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("damage_report")
                && row.raw_output_artifact_ids.contains(&"damage_profile".to_string())
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "bam.genotyping"
                && super::output_contract_status_label(row.output_contract_status)
                    == "missing_adapter"
                && row.missing_declarations == vec!["adapter".to_string()]
        }));
    }

    #[test]
    fn bam_adapter_output_contract_writes_governed_tsv_columns() {
        let root = repo_root();
        let output_path = PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH);
        let report =
            render_bam_adapter_output_contract(&root, output_path).expect("render contract");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered BAM adapter output contract tsv");
        let rows = rendered.lines().collect::<Vec<_>>();

        assert_eq!(
            rows.first().copied(),
            Some(
                "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason"
            )
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "samtools\tbam.validate\tplannable\tcomplete\tvalidation_report,flagstat,stage_metrics\tvalidation_report,flagstat,stage_metrics\t"
                )
            }),
            "the governed BAM validate adapter row must remain fully declared"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bcftools\tbam.genotyping\tdeclared_only\tmissing_adapter\tgenotyping_report,summary,stage_metrics\t\t\t\t\t\t\t\t\tadapter\t"
                )
            }),
            "the planned bcftools genotyping row must stay explicit as missing an adapter"
        );
    }
}
