#![allow(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::contract::canonical::to_canonical_json_bytes;
use crate::contract::{
    ArtifactRef, ArtifactRole, ArtifactRoleFamily, CompressionSupport, ExecutionGraph,
    ExecutionStep, PlanPolicy, ReadLayoutMode, ToolConstraints,
};
use crate::foundation::{BijuxError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerParameterSourceV1 {
    DomainDefault,
    PolicyDefault,
    UserManifest,
    WorkflowTemplate,
    BackendConstraint,
    PlannerInferred,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerRefusalCodeV1 {
    MissingStageSupport,
    MissingReference,
    IncompatibleLayout,
    ImpossibleFanIn,
    ImpossibleFanOut,
    UnsupportedBackend,
    UnsupportedArtifactHandoff,
    SampleMetadataMismatch,
    ReferenceAssetMismatch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerWarningCodeV1 {
    AdvisoryStage,
    DefaultPolicyApplied,
    StageSkipped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStageDecisionKindV1 {
    Included,
    Skipped,
    Refused,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowInputArtifactV1 {
    pub artifact_id: String,
    pub role: ArtifactRole,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<ReadLayoutMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionSupport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReferenceAssetV1 {
    pub asset_id: String,
    pub role: ArtifactRole,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowStageRequestV1 {
    pub stage_id: String,
    #[serde(default)]
    pub advisory_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEvidenceExpectationV1 {
    pub artifact_role: ArtifactRole,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicySurfaceV1 {
    #[serde(default)]
    pub scientific: serde_json::Value,
    #[serde(default)]
    pub operational: serde_json::Value,
}

impl Default for WorkflowPolicySurfaceV1 {
    fn default() -> Self {
        Self {
            scientific: serde_json::Value::Object(serde_json::Map::new()),
            operational: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct WorkflowExecutorPreferencesV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_mode: Option<String>,
    #[serde(default)]
    pub allow_advisory_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowManifestV1 {
    pub schema_version: String,
    pub domain: String,
    pub profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<WorkflowInputArtifactV1>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sample_metadata: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_assets: Vec<WorkflowReferenceAssetV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_stages: Vec<WorkflowStageRequestV1>,
    #[serde(default)]
    pub policies: WorkflowPolicySurfaceV1,
    #[serde(default)]
    pub executor_preferences: WorkflowExecutorPreferencesV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_expectations: Vec<WorkflowEvidenceExpectationV1>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl WorkflowManifestV1 {
    #[must_use]
    pub fn new(domain: impl Into<String>, profile_id: impl Into<String>) -> Self {
        Self {
            schema_version: "bijux.workflow_manifest.v1".to_string(),
            domain: domain.into(),
            profile_id: profile_id.into(),
            inputs: Vec::new(),
            sample_metadata: BTreeMap::new(),
            reference_assets: Vec::new(),
            requested_stages: Vec::new(),
            policies: WorkflowPolicySurfaceV1::default(),
            executor_preferences: WorkflowExecutorPreferencesV1::default(),
            evidence_expectations: Vec::new(),
            labels: BTreeMap::new(),
            notes: None,
        }
    }

    /// # Errors
    /// Returns an error when required fields are empty or identifiers are duplicated.
    pub fn validate(&self) -> Result<()> {
        if self.domain.trim().is_empty() {
            return Err(BijuxError::validation("workflow manifest domain must not be empty"));
        }
        if self.profile_id.trim().is_empty() {
            return Err(BijuxError::validation("workflow manifest profile_id must not be empty"));
        }
        ensure_unique(
            self.inputs.iter().map(|artifact| artifact.artifact_id.as_str()),
            "workflow input artifact_id",
        )?;
        ensure_unique(
            self.reference_assets.iter().map(|asset| asset.asset_id.as_str()),
            "workflow reference asset_id",
        )?;
        ensure_unique(
            self.requested_stages.iter().map(|stage| stage.stage_id.as_str()),
            "workflow requested stage_id",
        )?;
        Ok(())
    }

    /// # Errors
    /// Returns an error when validation or canonical JSON serialization fails.
    pub fn normalized(&self) -> Result<Self> {
        self.validate()?;
        let mut normalized = self.clone();
        normalized.inputs.sort_by(|a, b| {
            key_artifact(&a.artifact_id, a.role, &a.path).cmp(&key_artifact(
                &b.artifact_id,
                b.role,
                &b.path,
            ))
        });
        for input in &mut normalized.inputs {
            input.path = PathBuf::from(stable_path_identity(&input.path));
            if let Some(format_id) = input.format_id.as_mut() {
                *format_id = stable_command_fragment_identity(format_id);
            }
        }
        normalized.reference_assets.sort_by(|a, b| {
            key_artifact(&a.asset_id, a.role, &a.path).cmp(&key_artifact(
                &b.asset_id,
                b.role,
                &b.path,
            ))
        });
        for asset in &mut normalized.reference_assets {
            asset.path = PathBuf::from(stable_path_identity(&asset.path));
            if let Some(build_id) = asset.build_id.as_mut() {
                *build_id = stable_command_fragment_identity(build_id);
            }
            if let Some(alias_group) = asset.alias_group.as_mut() {
                *alias_group = stable_command_fragment_identity(alias_group);
            }
        }
        normalized.requested_stages.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));
        normalized
            .evidence_expectations
            .sort_by(|a, b| (a.artifact_role.as_str(), a.schema_id.as_deref().unwrap_or(""))
                .cmp(&(b.artifact_role.as_str(), b.schema_id.as_deref().unwrap_or(""))));
        Ok(normalized)
    }

    /// # Errors
    /// Returns an error when validation or canonical JSON serialization fails.
    pub fn fingerprint(&self) -> Result<String> {
        let mut normalized = self.normalized()?;
        normalized.labels.clear();
        normalized.notes = None;
        semantic_hash(&normalized)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ParameterResolutionTraceV1 {
    pub step_id: String,
    pub stage_id: String,
    pub parameter: String,
    pub source: PlannerParameterSourceV1,
    pub resolved_value: serde_json::Value,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlannerRefusalRecordV1 {
    pub code: PlannerRefusalCodeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlannerWarningRecordV1 {
    pub code: PlannerWarningCodeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowStageDecisionV1 {
    pub stage_id: String,
    pub decision: WorkflowStageDecisionKindV1,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossDomainHandoffCheckV1 {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrossDomainHandoffV1 {
    pub from_step_id: String,
    pub to_step_id: String,
    pub from_stage_id: String,
    pub to_stage_id: String,
    pub from_domain: String,
    pub to_domain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_role: Option<ArtifactRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_family: Option<ArtifactRoleFamily>,
    pub compatible: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<CrossDomainHandoffCheckV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlanArtifactPromiseV1 {
    pub artifact_id: String,
    pub role: ArtifactRole,
    pub path: PathBuf,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlanEnvironmentContractV1 {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    pub command: Vec<String>,
    pub resources: ToolConstraints,
    pub out_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanManifestStepV1 {
    pub step_id: String,
    pub stage_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_contract_ref: Option<String>,
    #[serde(default)]
    pub effective_parameters_json: serde_json::Value,
    pub environment: PlanEnvironmentContractV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_promises: Vec<PlanArtifactPromiseV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_asset_ids: Vec<String>,
    pub cache_key: String,
    #[serde(default)]
    pub advisory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanManifestV1 {
    pub schema_version: String,
    pub domain: String,
    pub profile_id: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub policy: PlanPolicy,
    pub workflow_fingerprint: String,
    pub graph_hash: String,
    pub plan_fingerprint: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ordered_steps: Vec<PlanManifestStepV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_decisions: Vec<WorkflowStageDecisionV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_records: Vec<PlannerRefusalRecordV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warning_records: Vec<PlannerWarningRecordV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_traces: Vec<ParameterResolutionTraceV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cross_domain_handoffs: Vec<CrossDomainHandoffV1>,
}

impl PlanManifestV1 {
    /// # Errors
    /// Returns an error when canonical JSON serialization fails.
    pub fn normalized(&self) -> Result<Self> {
        let mut normalized = self.clone();
        normalized.ordered_steps.sort_by(|a, b| a.step_id.cmp(&b.step_id));
        for step in &mut normalized.ordered_steps {
            step.dependencies.sort();
            step.artifact_promises.sort_by(|a, b| {
                key_artifact(&a.artifact_id, a.role, &a.path).cmp(&key_artifact(
                    &b.artifact_id,
                    b.role,
                    &b.path,
                ))
            });
            step.reference_asset_ids.sort();
            step.environment.out_dir = PathBuf::from(stable_path_identity(&step.environment.out_dir));
            step.environment.command = step
                .environment
                .command
                .iter()
                .map(|fragment| stable_command_fragment_identity(fragment))
                .collect();
            normalize_json_value_paths(&mut step.effective_parameters_json);
            for artifact in &mut step.artifact_promises {
                artifact.path = PathBuf::from(stable_path_identity(&artifact.path));
            }
        }
        normalized.stage_decisions.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));
        normalized.refusal_records.sort_by(|a, b| {
            (a.stage_id.as_deref().unwrap_or(""), a.message.as_str())
                .cmp(&(b.stage_id.as_deref().unwrap_or(""), b.message.as_str()))
        });
        normalized.warning_records.sort_by(|a, b| {
            (a.stage_id.as_deref().unwrap_or(""), a.message.as_str())
                .cmp(&(b.stage_id.as_deref().unwrap_or(""), b.message.as_str()))
        });
        normalized.parameter_traces.sort_by(|a, b| {
            (&a.step_id, &a.parameter).cmp(&(&b.step_id, &b.parameter))
        });
        for trace in &mut normalized.parameter_traces {
            normalize_json_value_paths(&mut trace.resolved_value);
            trace.detail = stable_command_fragment_identity(&trace.detail);
        }
        normalized.cross_domain_handoffs.sort_by(|a, b| {
            (&a.from_step_id, &a.to_step_id).cmp(&(&b.from_step_id, &b.to_step_id))
        });
        Ok(normalized)
    }

    /// # Errors
    /// Returns an error when canonical JSON serialization fails.
    pub fn refresh_fingerprint(&mut self) -> Result<()> {
        self.plan_fingerprint.clear();
        let normalized = self.normalized()?;
        self.plan_fingerprint = semantic_hash(&normalized)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanFieldChangeV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    pub message: String,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanManifestDiffV1 {
    pub schema_version: String,
    pub from_plan_fingerprint: String,
    pub to_plan_fingerprint: String,
    pub semantically_equal: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graph_changes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_changes: Vec<PlanFieldChangeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_changes: Vec<PlanFieldChangeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backend_changes: Vec<PlanFieldChangeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_changes: Vec<PlanFieldChangeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_changes: Vec<PlanFieldChangeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanManifestBuildInputV1 {
    pub workflow_manifest: WorkflowManifestV1,
    pub graph: ExecutionGraph,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_contract_refs: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub effective_parameters_by_step: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_traces: Vec<ParameterResolutionTraceV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_records: Vec<PlannerRefusalRecordV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warning_records: Vec<PlannerWarningRecordV1>,
}

/// # Errors
/// Returns an error when the workflow or graph is invalid.
pub fn build_plan_manifest(input: PlanManifestBuildInputV1) -> Result<PlanManifestV1> {
    let workflow_fingerprint = input.workflow_manifest.fingerprint()?;
    let graph_hash = semantic_hash(&graph_identity_contract(&input.graph))?;
    let stage_contract_refs = input.stage_contract_refs.into_iter().collect::<BTreeMap<_, _>>();
    let dependency_map = dependency_map(&input.graph);
    let order = input.graph.topological_step_ids()?;
    let advisory_steps = advisory_step_ids(&input.warning_records);
    let stage_set = input
        .graph
        .steps()
        .iter()
        .map(|step| step.stage_id.to_string())
        .collect::<BTreeSet<_>>();

    let mut ordered_steps = Vec::with_capacity(order.len());
    for step_id in order {
        let step = input
            .graph
            .step_by_id(step_id.as_str())
            .ok_or_else(|| BijuxError::validation(format!("missing graph step {}", step_id.0)))?;
        ordered_steps.push(step_manifest(
            step,
            dependency_map.get(step_id.as_str()),
            stage_contract_refs.get(step.stage_id.as_str()),
            input.effective_parameters_by_step.get(step_id.as_str()).cloned(),
            &input.workflow_manifest.reference_assets,
            advisory_steps.contains(step_id.as_str()),
        )?);
    }

    let stage_decisions = workflow_stage_decisions(
        &input.workflow_manifest,
        &stage_set,
        &input.refusal_records,
        &advisory_steps,
    );
    let cross_domain_handoffs = validate_cross_domain_handoffs(&input.graph);
    let domain = input.workflow_manifest.domain.clone();
    let profile_id = input.workflow_manifest.profile_id.clone();
    let planner_version = input.graph.planner_version().to_string();
    let pipeline_id = input.graph.pipeline_id().to_string();
    let policy = input.graph.policy();
    let mut manifest = PlanManifestV1 {
        schema_version: "bijux.plan_manifest.v1".to_string(),
        domain,
        profile_id,
        pipeline_id,
        planner_version,
        policy,
        workflow_fingerprint,
        graph_hash,
        plan_fingerprint: String::new(),
        ordered_steps,
        stage_decisions,
        refusal_records: input.refusal_records,
        warning_records: input.warning_records,
        parameter_traces: input.parameter_traces,
        cross_domain_handoffs,
    };
    manifest.refresh_fingerprint()?;
    Ok(manifest)
}

#[must_use]
pub fn planner_refusal_from_message(
    stage_id: Option<&str>,
    message: &str,
) -> PlannerRefusalRecordV1 {
    let lower = message.to_ascii_lowercase();
    let code = if lower.contains("reference") {
        PlannerRefusalCodeV1::MissingReference
    } else if lower.contains("layout") {
        PlannerRefusalCodeV1::IncompatibleLayout
    } else if lower.contains("fan-in") {
        PlannerRefusalCodeV1::ImpossibleFanIn
    } else if lower.contains("fan-out") {
        PlannerRefusalCodeV1::ImpossibleFanOut
    } else if lower.contains("tool") || lower.contains("backend") {
        PlannerRefusalCodeV1::UnsupportedBackend
    } else {
        PlannerRefusalCodeV1::MissingStageSupport
    };
    PlannerRefusalRecordV1 {
        code,
        stage_id: stage_id.map(std::string::ToString::to_string),
        message: message.to_string(),
        remediation: None,
    }
}

#[must_use]
pub fn diff_plan_manifests(
    before: &PlanManifestV1,
    after: &PlanManifestV1,
    workflow_before: Option<&WorkflowManifestV1>,
    workflow_after: Option<&WorkflowManifestV1>,
) -> PlanManifestDiffV1 {
    let semantically_equal = before.plan_fingerprint == after.plan_fingerprint;
    let mut diff = PlanManifestDiffV1 {
        schema_version: "bijux.plan_manifest_diff.v1".to_string(),
        from_plan_fingerprint: before.plan_fingerprint.clone(),
        to_plan_fingerprint: after.plan_fingerprint.clone(),
        semantically_equal,
        graph_changes: Vec::new(),
        parameter_changes: Vec::new(),
        reference_changes: Vec::new(),
        backend_changes: Vec::new(),
        artifact_changes: Vec::new(),
        policy_changes: Vec::new(),
        ignored_changes: Vec::new(),
    };

    if before.policy != after.policy {
        diff.policy_changes.push(PlanFieldChangeV1 {
            step_id: None,
            message: "plan policy changed".to_string(),
            before: serde_json::to_value(before.policy).unwrap_or(serde_json::Value::Null),
            after: serde_json::to_value(after.policy).unwrap_or(serde_json::Value::Null),
        });
    }

    let before_steps = before
        .ordered_steps
        .iter()
        .map(|step| (step.step_id.as_str(), step))
        .collect::<BTreeMap<_, _>>();
    let after_steps = after
        .ordered_steps
        .iter()
        .map(|step| (step.step_id.as_str(), step))
        .collect::<BTreeMap<_, _>>();
    let step_ids = before_steps
        .keys()
        .chain(after_steps.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    for step_id in step_ids {
        match (before_steps.get(step_id), after_steps.get(step_id)) {
            (Some(before_step), Some(after_step)) => {
                if before_step.dependencies != after_step.dependencies {
                    diff.graph_changes.push(format!(
                        "dependencies changed for {step_id}: {:?} -> {:?}",
                        before_step.dependencies, after_step.dependencies
                    ));
                }
                if before_step.environment != after_step.environment {
                    diff.backend_changes.push(PlanFieldChangeV1 {
                        step_id: Some(step_id.to_string()),
                        message: "execution backend changed".to_string(),
                        before: serde_json::to_value(&before_step.environment)
                            .unwrap_or(serde_json::Value::Null),
                        after: serde_json::to_value(&after_step.environment)
                            .unwrap_or(serde_json::Value::Null),
                    });
                }
                if before_step.effective_parameters_json != after_step.effective_parameters_json {
                    diff.parameter_changes.push(PlanFieldChangeV1 {
                        step_id: Some(step_id.to_string()),
                        message: "effective parameters changed".to_string(),
                        before: before_step.effective_parameters_json.clone(),
                        after: after_step.effective_parameters_json.clone(),
                    });
                }
                if before_step.reference_asset_ids != after_step.reference_asset_ids {
                    diff.reference_changes.push(PlanFieldChangeV1 {
                        step_id: Some(step_id.to_string()),
                        message: "reference asset selection changed".to_string(),
                        before: serde_json::to_value(&before_step.reference_asset_ids)
                            .unwrap_or(serde_json::Value::Null),
                        after: serde_json::to_value(&after_step.reference_asset_ids)
                            .unwrap_or(serde_json::Value::Null),
                    });
                }
                if before_step.artifact_promises != after_step.artifact_promises {
                    diff.artifact_changes.push(PlanFieldChangeV1 {
                        step_id: Some(step_id.to_string()),
                        message: "artifact promises changed".to_string(),
                        before: serde_json::to_value(&before_step.artifact_promises)
                            .unwrap_or(serde_json::Value::Null),
                        after: serde_json::to_value(&after_step.artifact_promises)
                            .unwrap_or(serde_json::Value::Null),
                    });
                }
            }
            (Some(_), None) => diff.graph_changes.push(format!("step removed: {step_id}")),
            (None, Some(_)) => diff.graph_changes.push(format!("step added: {step_id}")),
            (None, None) => {}
        }
    }

    if let (Some(before_workflow), Some(after_workflow)) = (workflow_before, workflow_after) {
        let mut before_noise = before_workflow.clone();
        let mut after_noise = after_workflow.clone();
        let notes_changed = before_noise.notes != after_noise.notes;
        let labels_changed = before_noise.labels != after_noise.labels;
        before_noise.notes = None;
        after_noise.notes = None;
        before_noise.labels.clear();
        after_noise.labels.clear();
        if notes_changed || labels_changed {
            if before_noise == after_noise {
                diff.ignored_changes
                    .push("workflow notes or labels changed without changing semantics".to_string());
            } else {
                diff.graph_changes
                    .push("workflow authoring metadata changed alongside semantic content".to_string());
            }
        }
    }

    diff
}

#[must_use]
pub fn validate_cross_domain_handoffs(graph: &ExecutionGraph) -> Vec<CrossDomainHandoffV1> {
    let steps = graph
        .steps()
        .iter()
        .map(|step| (step.step_id.as_str(), step))
        .collect::<BTreeMap<_, _>>();
    graph.edges()
        .iter()
        .filter_map(|edge| {
            let from_step = steps.get(edge.from().as_str())?;
            let to_step = steps.get(edge.to().as_str())?;
            let from_domain = stage_domain(from_step.stage_id.as_str());
            let to_domain = stage_domain(to_step.stage_id.as_str());
            if from_domain == to_domain {
                return None;
            }
            Some(cross_domain_handoff(from_step, to_step))
        })
        .collect()
}

fn cross_domain_handoff(from_step: &ExecutionStep, to_step: &ExecutionStep) -> CrossDomainHandoffV1 {
    let from_domain = stage_domain(from_step.stage_id.as_str()).to_string();
    let to_domain = stage_domain(to_step.stage_id.as_str()).to_string();
    let shared_role = first_shared_role(from_step, to_step);
    let artifact_family = shared_role.map(ArtifactRole::family);
    let mut checks = Vec::new();
    checks.push(CrossDomainHandoffCheckV1 {
        name: "typed_artifact_role".to_string(),
        passed: shared_role.is_some_and(ArtifactRole::is_typed),
        detail: shared_role
            .map(|role| format!("shared role {}", role.as_str()))
            .unwrap_or_else(|| "no typed artifact role shared across boundary".to_string()),
    });
    let family_ok = match (from_domain.as_str(), to_domain.as_str(), artifact_family) {
        ("fastq", "bam", Some(ArtifactRoleFamily::Reads)) => true,
        ("bam", "vcf", Some(ArtifactRoleFamily::Alignment)) => true,
        _ => false,
    };
    checks.push(CrossDomainHandoffCheckV1 {
        name: "role_family_compatibility".to_string(),
        passed: family_ok,
        detail: artifact_family.map_or_else(
            || "no compatible artifact family found".to_string(),
            |family| format!("{from_domain} -> {to_domain} uses {:?}", family),
        ),
    });
    let compatible = checks.iter().all(|check| check.passed);
    CrossDomainHandoffV1 {
        from_step_id: from_step.step_id.to_string(),
        to_step_id: to_step.step_id.to_string(),
        from_stage_id: from_step.stage_id.to_string(),
        to_stage_id: to_step.stage_id.to_string(),
        from_domain,
        to_domain,
        artifact_role: shared_role,
        artifact_family,
        compatible,
        checks,
    }
}

fn workflow_stage_decisions(
    workflow: &WorkflowManifestV1,
    stage_ids: &BTreeSet<String>,
    refusals: &[PlannerRefusalRecordV1],
    advisory_steps: &BTreeSet<String>,
) -> Vec<WorkflowStageDecisionV1> {
    workflow
        .requested_stages
        .iter()
        .map(|request| {
            let included = stage_ids.contains(&request.stage_id);
            let refusal = refusals
                .iter()
                .find(|record| record.stage_id.as_deref() == Some(request.stage_id.as_str()));
            if let Some(refusal) = refusal {
                return WorkflowStageDecisionV1 {
                    stage_id: request.stage_id.clone(),
                    decision: WorkflowStageDecisionKindV1::Refused,
                    reason: refusal.message.clone(),
                };
            }
            if request.advisory_only || advisory_steps.contains(request.stage_id.as_str()) {
                return WorkflowStageDecisionV1 {
                    stage_id: request.stage_id.clone(),
                    decision: WorkflowStageDecisionKindV1::Advisory,
                    reason: if included {
                        "included with advisory-only semantics".to_string()
                    } else {
                        "requested advisory stage was not selected".to_string()
                    },
                };
            }
            if included {
                WorkflowStageDecisionV1 {
                    stage_id: request.stage_id.clone(),
                    decision: WorkflowStageDecisionKindV1::Included,
                    reason: "included in deterministic plan".to_string(),
                }
            } else {
                WorkflowStageDecisionV1 {
                    stage_id: request.stage_id.clone(),
                    decision: WorkflowStageDecisionKindV1::Skipped,
                    reason: "not present in resolved execution graph".to_string(),
                }
            }
        })
        .collect()
}

fn advisory_step_ids(warnings: &[PlannerWarningRecordV1]) -> BTreeSet<String> {
    warnings
        .iter()
        .filter(|warning| warning.code == PlannerWarningCodeV1::AdvisoryStage)
        .filter_map(|warning| warning.stage_id.clone())
        .collect()
}

fn dependency_map(graph: &ExecutionGraph) -> BTreeMap<String, Vec<String>> {
    let mut dependencies = BTreeMap::<String, Vec<String>>::new();
    for edge in graph.edges() {
        dependencies
            .entry(edge.to().to_string())
            .or_default()
            .push(edge.from().to_string());
    }
    for deps in dependencies.values_mut() {
        deps.sort();
    }
    dependencies
}

fn step_manifest(
    step: &ExecutionStep,
    dependencies: Option<&Vec<String>>,
    stage_contract_ref: Option<&String>,
    effective_parameters_json: Option<serde_json::Value>,
    reference_assets: &[WorkflowReferenceAssetV1],
    advisory: bool,
) -> Result<PlanManifestStepV1> {
    let effective_parameters_json = effective_parameters_json
        .unwrap_or_else(|| serde_json::json!({ "command_template": step.command.template }));
    let mut normalized_effective_parameters_json = effective_parameters_json.clone();
    normalize_json_value_paths(&mut normalized_effective_parameters_json);
    let reference_asset_ids = if step
        .io
        .inputs
        .iter()
        .any(|input| matches!(input.role.family(), ArtifactRoleFamily::Reference | ArtifactRoleFamily::Index))
    {
        reference_assets.iter().map(|asset| asset.asset_id.clone()).collect()
    } else {
        Vec::new()
    };
    let artifact_promises = step
        .io
        .outputs
        .iter()
        .map(|output| PlanArtifactPromiseV1 {
            artifact_id: output.name.to_string(),
            role: output.role,
            path: output.path.clone(),
            optional: output.optional,
        })
        .collect::<Vec<_>>();
    let cache_key = semantic_hash(&serde_json::json!({
        "stage_id": step.stage_id,
        "image": {
            "image": step.image.image,
            "digest": step.image.digest,
        },
        "resources": step.resources,
        "inputs": step
            .io
            .inputs
            .iter()
            .map(artifact_cache_identity)
            .collect::<Vec<_>>(),
        "outputs": step
            .io
            .outputs
            .iter()
            .map(artifact_cache_identity)
            .collect::<Vec<_>>(),
        "out_dir": stable_path_identity(&step.out_dir),
        "effective_parameters_json": normalized_effective_parameters_json,
        "reference_asset_ids": reference_asset_ids,
    }))?;
    Ok(PlanManifestStepV1 {
        step_id: step.step_id.to_string(),
        stage_id: step.stage_id.to_string(),
        dependencies: dependencies.cloned().unwrap_or_default(),
        stage_contract_ref: stage_contract_ref.cloned(),
        effective_parameters_json,
        environment: PlanEnvironmentContractV1 {
            image: step.image.image.clone(),
            image_digest: step.image.digest.clone(),
            command: step.command.template.clone(),
            resources: step.resources.clone(),
            out_dir: step.out_dir.clone(),
        },
        artifact_promises,
        reference_asset_ids,
        cache_key,
        advisory,
    })
}

fn first_shared_role(from_step: &ExecutionStep, to_step: &ExecutionStep) -> Option<ArtifactRole> {
    from_step.io.outputs.iter().find_map(|output| {
        to_step
            .io
            .inputs
            .iter()
            .find(|input| input.role.family() == output.role.family() && output.role.is_typed())
            .map(|_| output.role)
    })
}

fn key_artifact(id: &str, role: ArtifactRole, path: &PathBuf) -> (String, &'static str, String) {
    (id.to_string(), role.as_str(), path.display().to_string())
}

fn graph_identity_contract(graph: &ExecutionGraph) -> serde_json::Value {
    serde_json::json!({
        "pipeline_id": graph.pipeline_id(),
        "planner_version": graph.planner_version(),
        "policy": graph.policy(),
        "deterministic_scheduler": graph.deterministic_scheduler(),
        "retry_policy": graph.retry_policy(),
        "step_timeout_s": graph.step_timeout_s(),
        "steps": graph
            .steps()
            .iter()
            .map(|step| serde_json::json!({
                "step_id": step.step_id,
                "stage_id": step.stage_id,
                "image": {
                    "image": step.image.image,
                    "digest": step.image.digest,
                },
                "resources": step.resources,
                "inputs": step.io.inputs.iter().map(artifact_cache_identity).collect::<Vec<_>>(),
                "outputs": step.io.outputs.iter().map(artifact_cache_identity).collect::<Vec<_>>(),
                "out_dir": stable_path_identity(&step.out_dir),
            }))
            .collect::<Vec<_>>(),
        "edges": graph.edges(),
    })
}

fn artifact_cache_identity(artifact: &ArtifactRef) -> serde_json::Value {
    serde_json::json!({
        "artifact_id": artifact.name.to_string(),
        "role": artifact.role.as_str(),
        "optional": artifact.optional,
        "path_identity": stable_path_identity(&artifact.path),
    })
}

fn stable_path_identity(path: &std::path::Path) -> String {
    if path.is_absolute() {
        path.file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

fn stable_command_fragment_identity(fragment: &str) -> String {
    let absolute_path_pattern =
        Regex::new(r"/[A-Za-z0-9._~:@%+=,-]+(?:/[A-Za-z0-9._~:@%+=,-]+)*").expect("valid regex");
    absolute_path_pattern
        .replace_all(fragment, |captures: &regex::Captures<'_>| {
            stable_path_identity(std::path::Path::new(&captures[0]))
        })
        .into_owned()
}

fn normalize_json_value_paths(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => {
            *text = stable_command_fragment_identity(text);
        }
        serde_json::Value::Array(values) => {
            for child in values {
                normalize_json_value_paths(child);
            }
        }
        serde_json::Value::Object(map) => {
            for child in map.values_mut() {
                normalize_json_value_paths(child);
            }
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_) => {}
    }
}

fn stage_domain(stage_id: &str) -> String {
    stage_id.split('.').next().unwrap_or("unknown").to_string()
}

fn ensure_unique<'a>(
    values: impl Iterator<Item = &'a str>,
    label: &str,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.to_string()) {
            return Err(BijuxError::validation(format!("duplicate {label}: {value}")));
        }
    }
    Ok(())
}

fn semantic_hash<T: Serialize>(value: &T) -> Result<String> {
    let bytes = to_canonical_json_bytes(value)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(hex)
}
