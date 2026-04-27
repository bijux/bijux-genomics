#![allow(clippy::too_many_arguments)]

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId,
    StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    qc_post::{
        QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams, REPORT_QC_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::{
    GovernedQcContributorV1, ReportQcReportV1, REPORT_QC_REPORT_SCHEMA_VERSION, STAGE_REPORT_QC,
};
use bijux_dna_stage_contract::{StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_REPORT_QC;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
const GOVERNED_QC_INPUTS_SCHEMA_VERSION: &str = "bijux.fastq.report_qc.inputs.v1";

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GovernedQcContributor {
    contributor_id: String,
    stage_id: String,
    tool_id: String,
    artifact_id: String,
    artifact_role: ArtifactRole,
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct GovernedQcInputsManifest {
    schema_version: String,
    qc_inputs: Vec<ArtifactRef>,
    contributors: Vec<GovernedQcContributor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw_fastqc_dir: Option<std::path::PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lineage_hash: Option<String>,
}

pub fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

#[must_use]
pub fn aux_tool_ids() -> Vec<String> {
    crate::qc_contract::governed_qc_default_tool_ids()
}

#[must_use]
pub fn aux_tool_ids_for_qc_inputs(qc_inputs: &[ArtifactRef]) -> Vec<String> {
    let mut tool_ids = qc_inputs
        .iter()
        .filter_map(|artifact| qc_contributor_identity_from_artifact_name(artifact.name.as_str()))
        .map(|(_stage_id, tool_id)| tool_id)
        .collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    if tool_ids.is_empty() {
        return aux_tool_ids();
    }
    tool_ids
}

/// Build a QC reporting plan from governed upstream QC artifacts.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_qc_post_with_qc_inputs(
    tool: &ToolExecutionSpecV1,
    qc_inputs: &[ArtifactRef],
    out_dir: &Path,
    aux_images: std::collections::BTreeMap<String, ContainerImageRefV1>,
    paired_mode: PairedMode,
    aggregation_engine: QcAggregationEngine,
    aggregation_scope: QcAggregationScope,
    raw_r1: Option<&Path>,
    raw_r2: Option<&Path>,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    if normalize_qc_post_tool_list(std::slice::from_ref(&tool_id))?.is_empty() {
        return Err(anyhow!("unsupported report_qc tool"));
    }
    if qc_inputs.is_empty() {
        return Err(anyhow!(
            "fastq.report_qc requires governed QC artifacts and cannot aggregate raw FASTQ inputs"
        ));
    }
    let qc_contributor_stage_ids = qc_contributor_stage_ids(qc_inputs);
    let qc_contributor_tool_ids = aux_tool_ids_for_qc_inputs(qc_inputs);
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "qc_input_paths": qc_inputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>(),
        "qc_input_count": qc_inputs.len(),
        "qc_contributor_stage_ids": qc_contributor_stage_ids,
        "qc_contributor_tool_ids": qc_contributor_tool_ids,
        "aggregation_engine": aggregation_engine,
        "aggregation_scope": aggregation_scope,
        "out_dir": out_dir
    });
    if let Some(raw) = raw_r1 {
        params["raw_r1"] = serde_json::json!(raw);
    }
    if let Some(raw) = raw_r2 {
        params["raw_r2"] = serde_json::json!(raw);
    }
    let effective_params = QcPostEffectiveParams {
        schema_version: REPORT_QC_SCHEMA_VERSION.to_string(),
        paired_mode,
        aggregation_engine,
        aggregation_scope,
    };
    let multiqc_data = out_dir.join("multiqc_data");
    let governed_qc_manifest = out_dir.join("governed_qc_inputs_manifest.json");
    let report_json = out_dir.join("report_qc_report.json");
    params["governed_qc_inputs_manifest"] = serde_json::json!(governed_qc_manifest.clone());
    params["report_json"] = serde_json::json!(report_json.clone());
    let command_template = qc_post_command(
        &tool.tool_id.0,
        qc_inputs,
        &multiqc_data,
        &governed_qc_manifest,
        &report_json,
        &effective_params,
    )?;
    let outputs = if tool.tool_id.0 == "multiqc" {
        vec![
            ArtifactRef::required(
                ArtifactId::from_static("multiqc_report"),
                out_dir.join("multiqc_report.html"),
                ArtifactRole::ReportHtml,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("multiqc_data"),
                multiqc_data.clone(),
                ArtifactRole::Index,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("governed_qc_inputs_manifest"),
                governed_qc_manifest,
                ArtifactRole::SummaryJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("report_json"),
                report_json,
                ArtifactRole::ReportJson,
            ),
        ]
    } else {
        Vec::new()
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 { template: command_template },
        resources: tool.resources.clone(),
        io: StageIO { inputs: qc_inputs.to_vec(), outputs },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize report_qc effective params: {error}"))?,
        aux_images,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn qc_post_command(
    tool_id: &str,
    qc_inputs: &[ArtifactRef],
    multiqc_data: &Path,
    governed_qc_manifest: &Path,
    report_json: &Path,
    effective_params: &QcPostEffectiveParams,
) -> Result<Vec<String>> {
    match tool_id {
        "multiqc" => {
            let mut multiqc_inputs =
                qc_inputs.iter().map(|artifact| artifact.path.clone()).collect::<Vec<_>>();
            multiqc_inputs.sort();
            multiqc_inputs.dedup();
            let manifest =
                serde_json::to_string(&governed_qc_inputs_manifest_payload(qc_inputs))
                    .map_err(|error| anyhow!("serialize governed QC inputs manifest: {error}"))?;
            let report = serde_json::to_string(&governed_report_qc_report(
                tool_id,
                effective_params,
                qc_inputs,
                multiqc_data,
                &out_dir_multiqc_report(multiqc_data),
                governed_qc_manifest,
            ))
            .map_err(|error| anyhow!("serialize governed report_qc report: {error}"))?;
            let mut script = format!(
                "set -eu\nprintf '%s\\n' {} > {}\nmultiqc -o {} -n multiqc_report.html",
                shell_quote_str(&manifest),
                shell_quote_path(governed_qc_manifest),
                shell_quote_path(multiqc_data),
            );
            for input in multiqc_inputs {
                script.push(' ');
                script.push_str(&shell_quote_str(&input.display().to_string()));
            }
            script
                .write_fmt(format_args!(
                    "\nprintf '%s\\n' {} > {}\n",
                    shell_quote_str(&report),
                    shell_quote_path(report_json),
                ))
                .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!("unsupported report_qc tool: {tool_id}")),
    }
}

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}

fn qc_contributor_identity_from_artifact_name(name: &str) -> Option<(String, String)> {
    let parts = name.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts[2] == "tool" {
        return Some((format!("{}.{}", parts[0], parts[1]), parts[3].to_string()));
    }
    if parts.len() >= 4 {
        return Some((format!("{}.{}", parts[0], parts[1]), parts[2].to_string()));
    }
    None
}

fn qc_contributor_stage_ids(qc_inputs: &[ArtifactRef]) -> Vec<String> {
    let mut stage_ids = qc_inputs
        .iter()
        .filter_map(|artifact| qc_contributor_identity_from_artifact_name(artifact.name.as_str()))
        .map(|(stage_id, _tool_id)| stage_id)
        .collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

fn governed_qc_inputs_manifest_payload(qc_inputs: &[ArtifactRef]) -> GovernedQcInputsManifest {
    let contributors = governed_qc_contributors(qc_inputs);
    GovernedQcInputsManifest {
        schema_version: GOVERNED_QC_INPUTS_SCHEMA_VERSION.to_string(),
        qc_inputs: qc_inputs.to_vec(),
        raw_fastqc_dir: None,
        lineage_hash: derived_governed_qc_lineage_hash(&contributors),
        contributors,
    }
}

fn governed_qc_contributors(qc_inputs: &[ArtifactRef]) -> Vec<GovernedQcContributor> {
    let mut contributors = qc_inputs.iter().filter_map(governed_qc_contributor).collect::<Vec<_>>();
    contributors.sort_by(|left, right| {
        left.contributor_id
            .cmp(&right.contributor_id)
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
            .then_with(|| left.artifact_role.as_str().cmp(right.artifact_role.as_str()))
            .then_with(|| left.path.cmp(&right.path))
    });
    contributors.dedup_by(|left, right| {
        left.contributor_id == right.contributor_id
            && left.artifact_id == right.artifact_id
            && left.artifact_role == right.artifact_role
            && left.path == right.path
    });
    contributors
}

fn governed_qc_contributor(artifact: &ArtifactRef) -> Option<GovernedQcContributor> {
    let artifact_name = artifact.name.as_str();
    let (contributor_id, artifact_id) = artifact_name.rsplit_once('.')?;
    let contributor_parts = contributor_id.split('.').collect::<Vec<_>>();
    if contributor_parts.len() < 3 {
        return None;
    }
    let tool_id = if contributor_parts.get(2) == Some(&"tool") {
        contributor_parts.get(3..)?.join(".")
    } else {
        contributor_parts[2..].join(".")
    };
    Some(GovernedQcContributor {
        contributor_id: contributor_id.to_string(),
        stage_id: format!("{}.{}", contributor_parts[0], contributor_parts[1]),
        tool_id,
        artifact_id: artifact_id.to_string(),
        artifact_role: artifact.role,
        path: artifact.path.clone(),
    })
}

fn derived_governed_qc_lineage_hash(contributors: &[GovernedQcContributor]) -> Option<String> {
    if contributors.is_empty() {
        return None;
    }
    Some(
        contributors
            .iter()
            .map(|contributor| {
                format!(
                    "{}:{}:{}={}",
                    contributor.contributor_id,
                    contributor.artifact_id,
                    contributor.artifact_role.as_str(),
                    contributor.path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("|"),
    )
}

fn governed_report_qc_report(
    tool_id: &str,
    effective_params: &QcPostEffectiveParams,
    qc_inputs: &[ArtifactRef],
    multiqc_data: &Path,
    multiqc_report: &Path,
    governed_qc_manifest: &Path,
) -> ReportQcReportV1 {
    let contributors = governed_qc_contributors(qc_inputs);
    let governed_contributors = contributors
        .iter()
        .map(|contributor| GovernedQcContributorV1 {
            contributor_id: contributor.contributor_id.clone(),
            stage_id: contributor.stage_id.clone(),
            tool_id: contributor.tool_id.clone(),
            artifact_id: contributor.artifact_id.clone(),
            artifact_role: contributor.artifact_role.as_str().to_string(),
            path: contributor.path.display().to_string(),
        })
        .collect::<Vec<_>>();
    let mut contributor_stage_ids = governed_contributors
        .iter()
        .map(|contributor| contributor.stage_id.clone())
        .collect::<Vec<_>>();
    contributor_stage_ids.sort();
    contributor_stage_ids.dedup();
    let mut contributor_tool_ids = governed_contributors
        .iter()
        .map(|contributor| contributor.tool_id.clone())
        .collect::<Vec<_>>();
    contributor_tool_ids.sort();
    contributor_tool_ids.dedup();
    ReportQcReportV1 {
        schema_version: REPORT_QC_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool_id.to_string(),
        paired_mode: effective_params.paired_mode,
        aggregation_engine: effective_params.aggregation_engine.clone(),
        aggregation_scope: effective_params.aggregation_scope.clone(),
        reads_in: 0,
        reads_out: 0,
        bases_in: 0,
        bases_out: 0,
        pairs_in: None,
        pairs_out: None,
        mean_q: 0.0,
        contamination_rate: 0.0,
        adapter_content_max: None,
        adapter_content_mean: None,
        duplication_rate: None,
        n_rate: None,
        kmer_warning_count: None,
        overrepresented_sequence_count: None,
        multiqc_sample_count: None,
        multiqc_module_count: None,
        raw_fastqc_dir: None,
        trimmed_fastqc_dir: None,
        multiqc_report: Some(multiqc_report.display().to_string()),
        multiqc_data: Some(multiqc_data.display().to_string()),
        governed_qc_input_count: qc_inputs.len() as u64,
        governed_qc_contributor_stage_ids: contributor_stage_ids,
        governed_qc_contributor_tool_ids: contributor_tool_ids,
        governed_qc_contributors: governed_contributors,
        governed_qc_lineage_hash: derived_governed_qc_lineage_hash(&contributors),
        governed_qc_inputs_manifest: Some(governed_qc_manifest.display().to_string()),
        runtime_s: None,
        memory_mb: None,
        exit_code: None,
    }
}

fn out_dir_multiqc_report(multiqc_data: &Path) -> std::path::PathBuf {
    multiqc_data.parent().map_or_else(
        || Path::new(".").join("multiqc_report.html"),
        |parent| parent.join("multiqc_report.html"),
    )
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::{
        aux_tool_ids_for_qc_inputs, governed_qc_inputs_manifest_payload,
        plan_qc_post_with_qc_inputs, qc_post_command,
    };
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, ToolConstraints,
        ToolExecutionSpecV1, ToolId,
    };
    use bijux_dna_domain_fastq::params::{
        qc_post::{QcAggregationEngine, QcAggregationScope, REPORT_QC_SCHEMA_VERSION},
        PairedMode,
    };
    use std::path::PathBuf;

    #[test]
    fn qc_post_command_sorts_and_deduplicates_governed_inputs() {
        let command = qc_post_command(
            "multiqc",
            &[
                ArtifactRef::required(
                    ArtifactId::from_static("artifact_b"),
                    PathBuf::from("zeta/fastqc"),
                    ArtifactRole::StageReport,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("artifact_a"),
                    PathBuf::from("alpha/fastqc"),
                    ArtifactRole::StageReport,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("artifact_dup"),
                    PathBuf::from("alpha/fastqc"),
                    ArtifactRole::StageReport,
                ),
            ],
            std::path::Path::new("out/multiqc_data"),
            std::path::Path::new("out/governed_qc_inputs_manifest.json"),
            std::path::Path::new("out/report_qc_report.json"),
            &bijux_dna_domain_fastq::params::qc_post::QcPostEffectiveParams {
                schema_version: REPORT_QC_SCHEMA_VERSION.to_string(),
                paired_mode: PairedMode::SingleEnd,
                aggregation_engine: QcAggregationEngine::Multiqc,
                aggregation_scope: QcAggregationScope::GovernedQcArtifacts,
            },
        )
        .expect("multiqc command should build");

        assert_eq!(command[0..2], ["sh".to_string(), "-lc".to_string()]);
        let script = &command[2];
        assert!(script.contains("out/governed_qc_inputs_manifest.json"));
        assert!(script.contains("out/report_qc_report.json"));
        assert!(script.contains("multiqc -o 'out/multiqc_data' -n multiqc_report.html"));
        assert!(script.contains("set -eu\n"));
        assert!(!script.contains("pipefail"));
        assert!(script.contains("'alpha/fastqc' 'zeta/fastqc'"));
    }

    #[test]
    fn qc_aux_tools_follow_qc_input_lineage() {
        let tool_ids = aux_tool_ids_for_qc_inputs(&[
            ArtifactRef::required(
                ArtifactId::from_static(
                    "fastq.validate_reads.tool.fastqvalidator.validation_report",
                ),
                PathBuf::from("validate/report.json"),
                ArtifactRole::StageReport,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("fastq.detect_adapters.tool.fastqc.adapter_report"),
                PathBuf::from("detect/report.json"),
                ArtifactRole::StageReport,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("fastq.detect_adapters.tool.fastqc.adapter_evidence_dir"),
                PathBuf::from("detect/evidence"),
                ArtifactRole::Index,
            ),
        ]);

        assert_eq!(tool_ids, vec!["fastqc".to_string(), "fastqvalidator".to_string()]);
    }

    #[test]
    fn governed_qc_manifest_payload_tracks_contributors_and_lineage() {
        let manifest = governed_qc_inputs_manifest_payload(&[
            ArtifactRef::required(
                ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
                PathBuf::from("trim/report.json"),
                ArtifactRole::ReportJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static(
                    "fastq.validate_reads.fastqvalidator.validated_reads_manifest",
                ),
                PathBuf::from("validate/lineage.json"),
                ArtifactRole::StageReport,
            ),
        ]);

        assert_eq!(manifest.contributors.len(), 2);
        assert_eq!(manifest.contributors[0].stage_id, "fastq.trim_reads");
        assert_eq!(manifest.contributors[0].artifact_id, "report_json");
        assert!(manifest.lineage_hash.is_some_and(|lineage| lineage.contains(
            "fastq.validate_reads.fastqvalidator:validated_reads_manifest:stage_report"
        )));
    }

    #[test]
    fn report_qc_plan_params_record_contributor_stage_and_tool_ids() {
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("multiqc"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
            resources: ToolConstraints::default(),
        };
        let plan = plan_qc_post_with_qc_inputs(
            &tool,
            &[
                ArtifactRef::required(
                    ArtifactId::from_static(
                        "fastq.validate_reads.fastqvalidator.validation_report",
                    ),
                    PathBuf::from("validate/report.json"),
                    ArtifactRole::StageReport,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("fastq.trim_reads.fastp.report_json"),
                    PathBuf::from("trim/report.json"),
                    ArtifactRole::ReportJson,
                ),
            ],
            std::path::Path::new("out"),
            std::collections::BTreeMap::new(),
            PairedMode::SingleEnd,
            QcAggregationEngine::Multiqc,
            QcAggregationScope::GovernedQcArtifacts,
            None,
            None,
        )
        .expect("plan");

        assert_eq!(
            plan.params["qc_contributor_stage_ids"],
            serde_json::json!(["fastq.trim_reads", "fastq.validate_reads"])
        );
        assert_eq!(
            plan.params["qc_contributor_tool_ids"],
            serde_json::json!(["fastp", "fastqvalidator"])
        );
        assert_eq!(plan.params["aggregation_engine"], serde_json::json!("multiqc"));
        assert_eq!(plan.params["aggregation_scope"], serde_json::json!("governed_qc_artifacts"));
        assert_eq!(
            plan.io.outputs.iter().map(|artifact| artifact.name.as_str()).collect::<Vec<_>>(),
            vec!["multiqc_report", "multiqc_data", "governed_qc_inputs_manifest", "report_json"]
        );
    }
}
