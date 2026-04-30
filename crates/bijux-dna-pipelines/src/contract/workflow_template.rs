use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use bijux_dna_core::contract::{
    ArtifactRole, CompressionSupport, ReadLayoutMode, WorkflowInputArtifactV1, WorkflowManifestV1,
    WorkflowReferenceAssetV1, WorkflowStageRequestV1,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BatchNodeScopeV1 {
    SharedReference,
    Sample,
    Cohort,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FanPatternV1 {
    FanOut,
    FanIn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateFailureActionV1 {
    BlockDownstream,
    SkipFailedSample,
    ContinueCohort,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BatchWorkflowSemanticsV1 {
    pub per_sample_stages: Vec<String>,
    pub cohort_stages: Vec<String>,
    pub shared_reference_stages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FanArtifactRuleV1 {
    pub source_stage: String,
    pub target_stage: String,
    pub fan_pattern: FanPatternV1,
    pub artifact_scope: String,
    pub lineage_fields: Vec<String>,
    pub overwrite_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossDomainFailurePolicyV1 {
    pub stage_family: String,
    pub action: TemplateFailureActionV1,
    pub downstream_effect: String,
    pub allows_partial_batch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossDomainEvidenceSummaryV1 {
    pub story_order: Vec<String>,
    pub final_caveat_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TemplateParameterPolicyV1 {
    pub expert_mode_required_for_locked_overrides: bool,
    pub configurable_by_stage: BTreeMap<String, Vec<String>>,
    pub locked_by_stage: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossWorkflowTemplateV1 {
    pub schema_version: String,
    pub template_id: String,
    pub pipeline_id: String,
    pub summary: String,
    pub requested_stages: Vec<String>,
    pub supported_layouts: Vec<ReadLayoutMode>,
    pub requires_reference_assets: bool,
    pub requires_bam_index: bool,
    pub requires_sample_metadata: Vec<String>,
    pub sample_sheet_supported: bool,
    pub batch_semantics: BatchWorkflowSemanticsV1,
    pub fan_artifact_rules: Vec<FanArtifactRuleV1>,
    pub failure_policy: Vec<CrossDomainFailurePolicyV1>,
    pub evidence_summary: CrossDomainEvidenceSummaryV1,
    pub parameter_policy: TemplateParameterPolicyV1,
    pub example_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SampleSheetFormatV1 {
    pub delimiter: String,
    pub required_columns: Vec<String>,
    pub optional_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SampleSheetRecordV1 {
    pub sample_id: String,
    pub library_id: String,
    pub lane_id: String,
    pub reference_id: String,
    pub workflow_mode: String,
    pub r1: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r2: Option<PathBuf>,
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SampleSheetV1 {
    pub schema_version: String,
    pub template_id: String,
    pub format: SampleSheetFormatV1,
    pub records: Vec<SampleSheetRecordV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBatchNodeV1 {
    pub node_id: String,
    pub stage_id: String,
    pub scope: BatchNodeScopeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBatchEdgeV1 {
    pub from: String,
    pub to: String,
    pub fan_pattern: FanPatternV1,
    pub artifact_scope: String,
    pub lineage_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBatchGraphV1 {
    pub schema_version: String,
    pub template_id: String,
    pub nodes: Vec<WorkflowBatchNodeV1>,
    pub edges: Vec<WorkflowBatchEdgeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowTemplateAdmissionCheckV1 {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowTemplateAdmissionV1 {
    pub template_id: String,
    pub admitted: bool,
    pub checks: Vec<WorkflowTemplateAdmissionCheckV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEvidenceSummaryStoryV1 {
    pub template_id: String,
    pub sections: BTreeMap<String, String>,
    pub final_caveats: Vec<String>,
}

#[must_use]
pub fn parse_sample_sheet(template_id: &str, input: &str) -> Result<SampleSheetV1> {
    let lines = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        bail!("sample sheet must contain a header row");
    }
    let delimiter = if lines[0].contains('\t') { '\t' } else { ',' };
    let headers = lines[0]
        .split(delimiter)
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    let required_columns = vec![
        "sample_id".to_string(),
        "library_id".to_string(),
        "lane_id".to_string(),
        "reference_id".to_string(),
        "workflow_mode".to_string(),
        "r1".to_string(),
        "expected_outputs".to_string(),
    ];
    for required in &required_columns {
        if !headers.iter().any(|header| header == required) {
            bail!("sample sheet missing required column {required}");
        }
    }

    let index_of = |name: &str| {
        headers
            .iter()
            .position(|header| header == name)
            .ok_or_else(|| anyhow!("sample sheet missing required column {name}"))
    };
    let sample_index = index_of("sample_id")?;
    let library_index = index_of("library_id")?;
    let lane_index = index_of("lane_id")?;
    let reference_index = index_of("reference_id")?;
    let mode_index = index_of("workflow_mode")?;
    let r1_index = index_of("r1")?;
    let expected_outputs_index = index_of("expected_outputs")?;
    let r2_index = headers.iter().position(|header| header == "r2");

    let mut seen_sample_lanes = BTreeSet::new();
    let mut records = Vec::new();
    for (row_offset, line) in lines.iter().skip(1).enumerate() {
        let row_number = row_offset + 2;
        let columns = line.split(delimiter).map(str::trim).collect::<Vec<_>>();
        if columns.len() != headers.len() {
            bail!(
                "sample sheet row {row_number} has {} columns but header has {}",
                columns.len(),
                headers.len()
            );
        }
        let sample_id = columns[sample_index].to_string();
        let library_id = columns[library_index].to_string();
        let lane_id = columns[lane_index].to_string();
        let reference_id = columns[reference_index].to_string();
        let workflow_mode = columns[mode_index].to_string();
        let r1 = PathBuf::from(columns[r1_index]);
        let r2 = r2_index.and_then(|index| {
            let value = columns[index];
            (!value.is_empty()).then(|| PathBuf::from(value))
        });
        let expected_outputs = columns[expected_outputs_index]
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if sample_id.is_empty() || library_id.is_empty() || lane_id.is_empty() {
            bail!("sample sheet row {row_number} must declare sample_id, library_id, and lane_id");
        }
        if !seen_sample_lanes.insert((sample_id.clone(), lane_id.clone())) {
            bail!("sample sheet repeats sample/lane pair {}:{}", sample_id, lane_id);
        }
        if expected_outputs.is_empty() {
            bail!("sample sheet row {row_number} must declare at least one expected output");
        }
        records.push(SampleSheetRecordV1 {
            sample_id,
            library_id,
            lane_id,
            reference_id,
            workflow_mode,
            r1,
            r2,
            expected_outputs,
        });
    }

    Ok(SampleSheetV1 {
        schema_version: "bijux.cross.sample_sheet.v1".to_string(),
        template_id: template_id.to_string(),
        format: SampleSheetFormatV1 {
            delimiter: delimiter.to_string(),
            required_columns,
            optional_columns: vec!["r2".to_string()],
        },
        records,
    })
}

pub fn sample_sheet_to_workflow_manifests(
    template: &CrossWorkflowTemplateV1,
    sheet: &SampleSheetV1,
) -> Result<Vec<WorkflowManifestV1>> {
    if sheet.template_id != template.template_id {
        bail!(
            "sample sheet template {} does not match requested template {}",
            sheet.template_id,
            template.template_id
        );
    }
    let mut manifests = Vec::with_capacity(sheet.records.len());
    for record in &sheet.records {
        let mut manifest = WorkflowManifestV1::new("cross", template.pipeline_id.clone());
        manifest.inputs.push(WorkflowInputArtifactV1 {
            artifact_id: format!("{}.r1", record.sample_id),
            role: ArtifactRole::Reads,
            path: record.r1.clone(),
            layout: Some(if record.r2.is_some() {
                ReadLayoutMode::PairedEnd
            } else {
                ReadLayoutMode::SingleEnd
            }),
            compression: Some(CompressionSupport::Gzip),
            format_id: Some("fastq.gz".to_string()),
        });
        if let Some(r2) = &record.r2 {
            manifest.inputs.push(WorkflowInputArtifactV1 {
                artifact_id: format!("{}.r2", record.sample_id),
                role: ArtifactRole::Reads,
                path: r2.clone(),
                layout: Some(ReadLayoutMode::PairedEnd),
                compression: Some(CompressionSupport::Gzip),
                format_id: Some("fastq.gz".to_string()),
            });
        }
        manifest.reference_assets.push(WorkflowReferenceAssetV1 {
            asset_id: record.reference_id.clone(),
            role: ArtifactRole::Reference,
            path: PathBuf::from(format!("references/{}.fa", record.reference_id)),
            checksum_sha256: None,
            build_id: Some(record.reference_id.clone()),
            alias_group: Some(record.reference_id.clone()),
        });
        manifest.requested_stages = template
            .requested_stages
            .iter()
            .cloned()
            .map(|stage_id| WorkflowStageRequestV1 { stage_id, advisory_only: false })
            .collect();
        manifest.sample_metadata.insert("sample_id".to_string(), record.sample_id.clone());
        manifest.sample_metadata.insert("library_id".to_string(), record.library_id.clone());
        manifest.sample_metadata.insert("lane_id".to_string(), record.lane_id.clone());
        manifest.sample_metadata.insert("workflow_mode".to_string(), record.workflow_mode.clone());
        manifest.sample_metadata.insert("reference_id".to_string(), record.reference_id.clone());
        for (index, output) in record.expected_outputs.iter().enumerate() {
            manifest
                .labels
                .insert(format!("expected_output.{index}"), output.clone());
            manifest.evidence_expectations.push(bijux_dna_core::contract::WorkflowEvidenceExpectationV1 {
                artifact_role: expected_output_role(output),
                required: true,
                advisory_only: false,
                schema_id: Some(format!("expected_output::{output}")),
            });
        }
        manifests.push(manifest);
    }
    Ok(manifests)
}

#[must_use]
pub fn build_batch_workflow_graph(
    template: &CrossWorkflowTemplateV1,
    sheet: &SampleSheetV1,
) -> WorkflowBatchGraphV1 {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for stage_id in &template.batch_semantics.shared_reference_stages {
        nodes.push(WorkflowBatchNodeV1 {
            node_id: format!("shared::{stage_id}"),
            stage_id: stage_id.clone(),
            scope: BatchNodeScopeV1::SharedReference,
            sample_id: None,
        });
    }
    for record in &sheet.records {
        for stage_id in &template.batch_semantics.per_sample_stages {
            nodes.push(WorkflowBatchNodeV1 {
                node_id: format!("sample::{}::{stage_id}", record.sample_id),
                stage_id: stage_id.clone(),
                scope: BatchNodeScopeV1::Sample,
                sample_id: Some(record.sample_id.clone()),
            });
        }
    }
    for stage_id in &template.batch_semantics.cohort_stages {
        nodes.push(WorkflowBatchNodeV1 {
            node_id: format!("cohort::{stage_id}"),
            stage_id: stage_id.clone(),
            scope: BatchNodeScopeV1::Cohort,
            sample_id: None,
        });
    }

    if let (Some(shared_last), Some(sample_first)) = (
        template.batch_semantics.shared_reference_stages.last(),
        template.batch_semantics.per_sample_stages.first(),
    ) {
        for record in &sheet.records {
            edges.push(WorkflowBatchEdgeV1 {
                from: format!("shared::{shared_last}"),
                to: format!("sample::{}::{sample_first}", record.sample_id),
                fan_pattern: FanPatternV1::FanOut,
                artifact_scope: "shared_reference_bundle".to_string(),
                lineage_fields: vec!["reference_id".to_string()],
            });
        }
    }
    for record in &sheet.records {
        for pair in template.batch_semantics.per_sample_stages.windows(2) {
            edges.push(WorkflowBatchEdgeV1 {
                from: format!("sample::{}::{}", record.sample_id, pair[0]),
                to: format!("sample::{}::{}", record.sample_id, pair[1]),
                fan_pattern: FanPatternV1::FanOut,
                artifact_scope: "sample_artifact".to_string(),
                lineage_fields: vec![
                    "sample_id".to_string(),
                    "library_id".to_string(),
                    "lane_id".to_string(),
                ],
            });
        }
    }
    if let (Some(sample_last), Some(cohort_first)) = (
        template.batch_semantics.per_sample_stages.last(),
        template.batch_semantics.cohort_stages.first(),
    ) {
        for record in &sheet.records {
            edges.push(WorkflowBatchEdgeV1 {
                from: format!("sample::{}::{sample_last}", record.sample_id),
                to: format!("cohort::{cohort_first}"),
                fan_pattern: FanPatternV1::FanIn,
                artifact_scope: "cohort_artifact".to_string(),
                lineage_fields: vec!["sample_id".to_string(), "reference_id".to_string()],
            });
        }
    }
    for pair in template.batch_semantics.cohort_stages.windows(2) {
        edges.push(WorkflowBatchEdgeV1 {
            from: format!("cohort::{}", pair[0]),
            to: format!("cohort::{}", pair[1]),
            fan_pattern: FanPatternV1::FanIn,
            artifact_scope: "cohort_artifact".to_string(),
            lineage_fields: vec!["sample_id".to_string()],
        });
    }

    WorkflowBatchGraphV1 {
        schema_version: "bijux.cross.batch_graph.v1".to_string(),
        template_id: template.template_id.clone(),
        nodes,
        edges,
    }
}

pub fn validate_template_overrides(
    template: &CrossWorkflowTemplateV1,
    overrides: &BTreeMap<String, serde_json::Value>,
    expert_mode: bool,
) -> Result<()> {
    for (stage_id, params) in overrides {
        let Some(object) = params.as_object() else {
            bail!("override payload for {stage_id} must be a JSON object");
        };
        let allowed = template
            .parameter_policy
            .configurable_by_stage
            .get(stage_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let locked = template
            .parameter_policy
            .locked_by_stage
            .get(stage_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeSet<_>>();
        for key in object.keys() {
            if locked.contains(key) && !expert_mode {
                bail!(
                    "override {stage_id}.{key} is locked by template policy; rerun in expert mode"
                );
            }
            if !allowed.contains(key) && !locked.contains(key) {
                bail!("override {stage_id}.{key} is not exposed by template policy");
            }
        }
    }
    Ok(())
}

#[must_use]
pub fn evaluate_template_admission(
    template: &CrossWorkflowTemplateV1,
    manifest: &WorkflowManifestV1,
    bam_index_present: bool,
) -> WorkflowTemplateAdmissionV1 {
    let mut checks = Vec::new();
    let layout_supported = manifest.inputs.iter().all(|input| {
        input.layout.is_none_or(|layout| template.supported_layouts.contains(&layout))
    });
    checks.push(WorkflowTemplateAdmissionCheckV1 {
        name: "layout_compatibility".to_string(),
        passed: layout_supported,
        detail: if layout_supported {
            "declared read layouts are supported by the template".to_string()
        } else {
            "one or more declared read layouts are incompatible with the template".to_string()
        },
    });
    let metadata_complete = template.requires_sample_metadata.iter().all(|field| {
        manifest
            .sample_metadata
            .get(field)
            .is_some_and(|value| !value.trim().is_empty())
    });
    checks.push(WorkflowTemplateAdmissionCheckV1 {
        name: "sample_metadata".to_string(),
        passed: metadata_complete,
        detail: if metadata_complete {
            "required sample metadata fields are present".to_string()
        } else {
            format!(
                "required sample metadata missing; expected {}",
                template.requires_sample_metadata.join(", ")
            )
        },
    });
    let reference_ready = !template.requires_reference_assets || !manifest.reference_assets.is_empty();
    checks.push(WorkflowTemplateAdmissionCheckV1 {
        name: "reference_assets".to_string(),
        passed: reference_ready,
        detail: if reference_ready {
            "reference assets are present for the template".to_string()
        } else {
            "template requires governed reference assets".to_string()
        },
    });
    let bam_index_ready = !template.requires_bam_index || bam_index_present;
    checks.push(WorkflowTemplateAdmissionCheckV1 {
        name: "bam_index".to_string(),
        passed: bam_index_ready,
        detail: if bam_index_ready {
            "BAM index prerequisites are satisfied".to_string()
        } else {
            "template requires a governed BAM index before downstream calling".to_string()
        },
    });
    let admitted = checks.iter().all(|check| check.passed);
    WorkflowTemplateAdmissionV1 {
        template_id: template.template_id.clone(),
        admitted,
        checks,
    }
}

#[must_use]
pub fn summarize_cross_domain_evidence(
    template: &CrossWorkflowTemplateV1,
    sections: &BTreeMap<String, String>,
    final_caveats: &[String],
) -> WorkflowEvidenceSummaryStoryV1 {
    let ordered_sections = template
        .evidence_summary
        .story_order
        .iter()
        .filter_map(|key| sections.get(key).map(|value| (key.clone(), value.clone())))
        .collect::<BTreeMap<_, _>>();
    let mut caveats = final_caveats.to_vec();
    if caveats.is_empty() {
        caveats.extend(template.evidence_summary.final_caveat_topics.iter().cloned());
    }
    WorkflowEvidenceSummaryStoryV1 {
        template_id: template.template_id.clone(),
        sections: ordered_sections,
        final_caveats: caveats,
    }
}

fn expected_output_role(output: &str) -> ArtifactRole {
    match output {
        "bam" => ArtifactRole::Bam,
        "vcf" | "variant" => ArtifactRole::Variant,
        "report_json" => ArtifactRole::ReportJson,
        "metrics" | "metrics_bundle" => ArtifactRole::MetricsEnvelope,
        _ => ArtifactRole::Evidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_output_role_maps_governed_values() {
        assert_eq!(expected_output_role("bam"), ArtifactRole::Bam);
        assert_eq!(expected_output_role("vcf"), ArtifactRole::Variant);
        assert_eq!(expected_output_role("metrics_bundle"), ArtifactRole::MetricsEnvelope);
    }
}
