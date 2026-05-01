mod artifact_catalog;
mod planner_contract;
mod stage_builder;

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::{
    run_dir, CanonicalStageContractV1, CompressionSupport, Profile, ReadLayoutMode, RunSpec,
    StageOperatingMode, StageRefusalCode,
};
use bijux_dna_core::contract::{ToolExecutionSpecV1, ToolRegistry};
use bijux_dna_core::ids::RunId;

use crate::stage_plan::{PlannedArtifactV1, StagePlanV1};
use bijux_dna_core::contract::ArtifactRef;
use bijux_dna_core::ids::ArtifactId;

pub use artifact_catalog::artifact_kind_schema;
pub use planner_contract::*;
pub use stage_builder::{
    build_stage_plan, build_tool_execution_spec, validate_stage_contract, validate_stage_outputs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunExecutionPlan {
    pub run_id: RunId,
    pub run_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub planned_artifacts: Vec<PlannedArtifactV1>,
    pub stage: StagePlanV1,
    pub tool: ToolExecutionSpecV1,
}

pub trait Executor {
    /// # Errors
    /// Returns an error if execution fails.
    fn run(&self, plan: &RunExecutionPlan) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct DryRunExecutor;

impl Executor for DryRunExecutor {
    fn run(&self, _plan: &RunExecutionPlan) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StageAdmissionRequestV1 {
    pub requested_mode: StageOperatingMode,
    #[serde(default)]
    pub read_layout: Option<ReadLayoutMode>,
    #[serde(default)]
    pub compression: Option<CompressionSupport>,
    #[serde(default)]
    pub requires_reference: bool,
    #[serde(default)]
    pub requires_index: bool,
    #[serde(default)]
    pub allow_unsafe_override: bool,
    #[serde(default = "default_backend_available")]
    pub backend_available: bool,
    #[serde(default = "default_scientific_ready")]
    pub scientifically_coherent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StageRefusalV1 {
    pub code: StageRefusalCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StageAdmissionOutcomeV1 {
    pub admitted: bool,
    pub selected_mode: StageOperatingMode,
    #[serde(default)]
    pub refusals: Vec<StageRefusalV1>,
}

/// # Errors
/// Returns an error if the registry is missing the requested stage or tool.
pub fn build_run_execution_plan(
    run_spec: &RunSpec,
    registry: &ToolRegistry,
    profile: &Profile,
    run_id: RunId,
) -> Result<RunExecutionPlan> {
    let stage_spec = registry
        .stages()
        .get(&run_spec.stage)
        .ok_or_else(|| anyhow!("missing stage {}", run_spec.stage.0))?;
    let tool_manifest = registry
        .tool_by_id(&run_spec.stage, &run_spec.tool)
        .ok_or_else(|| anyhow!("missing tool {} for {}", run_spec.tool.0, run_spec.stage.0))?;

    let run_dir = run_dir(&profile.run_base_dir, &run_id, &run_spec.stage, &run_spec.tool);
    let logs_dir = run_dir.join("logs");
    let artifacts_dir = run_dir.join("artifacts");
    validate_stage_outputs(stage_spec, run_spec)?;

    let inputs = stage_spec
        .inputs
        .iter()
        .map(|port| {
            ArtifactRef::required(
                ArtifactId::new(port.name.clone()),
                PathBuf::from(&port.name),
                port.artifact_role,
            )
        })
        .collect();
    let outputs = stage_spec
        .outputs
        .iter()
        .map(|port| {
            ArtifactRef::required(
                ArtifactId::new(port.name.clone()),
                PathBuf::from(&port.name),
                port.artifact_role,
            )
        })
        .collect();

    let stage = build_stage_plan(run_spec, tool_manifest, stage_spec, &run_dir, inputs, outputs)?;

    let planned_artifacts = stage
        .io
        .outputs
        .iter()
        .map(|artifact| {
            let role = artifact.role.as_str().to_string();
            let (kind, schema) = artifact_kind_schema(&role);
            PlannedArtifactV1 {
                artifact_id: artifact.name.0.to_string(),
                role,
                path: artifact.path.to_string_lossy().to_string(),
                kind: kind.to_string(),
                schema: schema.to_string(),
            }
        })
        .collect();

    let tool = build_tool_execution_spec(run_spec, tool_manifest);

    Ok(RunExecutionPlan {
        run_id,
        run_dir,
        logs_dir,
        artifacts_dir,
        planned_artifacts,
        stage,
        tool,
    })
}

#[must_use]
pub fn evaluate_stage_admission(
    contract: &CanonicalStageContractV1,
    request: &StageAdmissionRequestV1,
) -> StageAdmissionOutcomeV1 {
    let mut refusals = Vec::new();

    if !matches!(
        request.requested_mode,
        StageOperatingMode::Simulation
            | StageOperatingMode::Advisory
            | StageOperatingMode::Enforced
    ) {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::UnsupportedMode,
            message: format!(
                "stage {} requested an unsupported operating mode",
                contract.stage_id.0
            ),
        });
    }
    if matches!(contract.operating_mode, StageOperatingMode::Simulation)
        && !matches!(request.requested_mode, StageOperatingMode::Simulation)
    {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::UnsupportedMode,
            message: format!(
                "stage {} is simulation-only and cannot be presented as {:?}",
                contract.stage_id.0, request.requested_mode
            ),
        });
    }
    if matches!(request.requested_mode, StageOperatingMode::Enforced)
        && matches!(contract.operating_mode, StageOperatingMode::Advisory)
    {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::UnsupportedMode,
            message: format!(
                "stage {} is advisory-only and cannot be promoted to enforced without governed policy",
                contract.stage_id.0
            ),
        });
    }
    if request.requires_reference
        && matches!(
            contract.capability_contract.reference,
            bijux_dna_core::contract::ReferenceRequirement::None
        )
    {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::MissingReference,
            message: format!(
                "stage {} requires a governed reference contract",
                contract.stage_id.0
            ),
        });
    }
    if request.requires_index
        && matches!(
            contract.capability_contract.index,
            bijux_dna_core::contract::IndexRequirement::None
        )
    {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::MissingIndex,
            message: format!("stage {} requires a governed index contract", contract.stage_id.0),
        });
    }
    if let Some(layout) = request.read_layout {
        if !contract.capability_contract.layouts.is_empty()
            && !contract.capability_contract.layouts.contains(&layout)
        {
            refusals.push(StageRefusalV1 {
                code: StageRefusalCode::UnsupportedLayout,
                message: format!(
                    "stage {} backend {} does not support read layout {:?}",
                    contract.stage_id.0, contract.backend_tool_id.0, layout
                ),
            });
        }
    }
    if let Some(compression) = request.compression {
        if !contract.capability_contract.compression.is_empty()
            && !contract.capability_contract.compression.contains(&compression)
        {
            refusals.push(StageRefusalV1 {
                code: StageRefusalCode::IncompatibleInputs,
                message: format!(
                    "stage {} backend {} does not support input compression {:?}",
                    contract.stage_id.0, contract.backend_tool_id.0, compression
                ),
            });
        }
    }
    if !request.allow_unsafe_override
        && contract
            .capability_contract
            .unsupported_parameter_combinations
            .iter()
            .any(|combination| !combination.parameters.is_empty())
    {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::UnsafeOverride,
            message: format!(
                "stage {} declares governed unsupported parameter combinations that require audited override",
                contract.stage_id.0
            ),
        });
    }
    if !request.backend_available {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::BackendUnavailable,
            message: format!(
                "stage {} backend {} is not available in this environment",
                contract.stage_id.0, contract.backend_tool_id.0
            ),
        });
    }
    if !request.scientifically_coherent {
        refusals.push(StageRefusalV1 {
            code: StageRefusalCode::ScientificIncoherence,
            message: format!(
                "stage {} request violates governed scientific assumptions",
                contract.stage_id.0
            ),
        });
    }

    StageAdmissionOutcomeV1 {
        admitted: refusals.is_empty(),
        selected_mode: request.requested_mode,
        refusals,
    }
}

const fn default_backend_available() -> bool {
    true
}

const fn default_scientific_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::ArtifactRef;
    use bijux_dna_core::contract::{
        ArtifactKind, ArtifactRole, Cardinality, ExecutionContract, PathSpec, PortSpec,
        ReadLayoutMode, RunSpec, RuntimeScale, StageCapabilitySpec, StageFamily,
        StageOperatingMode, StageSemanticKind, StageSpec, ToolConstraints, ToolManifest, ToolRole,
        UnsupportedParameterCombination,
    };
    use bijux_dna_core::id_catalog;
    use bijux_dna_core::ids::{ArtifactId, StageId, ToolId};
    use bijux_dna_core::prelude::tooling::StageBehavior;

    fn tool_manifest() -> ToolManifest {
        ToolManifest {
            tool_id: ToolId::from_static("fastp"),
            stage_id: StageId::from_static(id_catalog::FASTQ_TRIM),
            role: ToolRole::Authoritative,
            command_template: vec!["fastp".to_string()],
            outputs: Vec::new(),
            metrics_parser: None,
            constraints: ToolConstraints::default(),
            execution_contract: ExecutionContract {
                requires_provenance: true,
                ..ExecutionContract::default()
            },
            supported_modes: vec![StageOperatingMode::Advisory, StageOperatingMode::Enforced],
            backend_version_policy: bijux_dna_core::contract::BackendVersionPolicy::Pinned,
            capability_contract: StageCapabilitySpec {
                layouts: vec![ReadLayoutMode::PairedEnd],
                compression: vec![bijux_dna_core::contract::CompressionSupport::Gzip],
                reference: bijux_dna_core::contract::ReferenceRequirement::Optional,
                index: bijux_dna_core::contract::IndexRequirement::Optional,
                output_formats: vec!["fastq.gz".to_string(), "json".to_string()],
                unsupported_parameter_combinations: Vec::new(),
            },
        }
    }

    fn stage_spec(stage_id: StageId) -> StageSpec {
        StageSpec {
            stage_id,
            stage_family: StageFamily::Fastq,
            semantic_kind: StageSemanticKind::Transform,
            input_kind: ArtifactKind::Fastq,
            output_kind: ArtifactKind::Fastq,
            produced_artifacts: vec!["trimmed_reads".to_string()],
            stage_semver: "1.0.0".to_string(),
            runtime_scale: RuntimeScale::Small,
            inputs: vec![PortSpec {
                name: "reads_r1".to_string(),
                data_type: "fastq".to_string(),
                cardinality: Cardinality::One,
                artifact_role: ArtifactRole::Reads,
            }],
            outputs: vec![PortSpec {
                name: "trimmed_reads".to_string(),
                data_type: "fastq".to_string(),
                cardinality: Cardinality::One,
                artifact_role: ArtifactRole::TrimmedReads,
            }],
            parameters: vec![
                bijux_dna_core::contract::StageParameterSpec {
                    name: "min_quality".to_string(),
                    param_type: "integer".to_string(),
                    default: Some("20".to_string()),
                    aliases: vec!["quality".to_string()],
                },
                bijux_dna_core::contract::StageParameterSpec {
                    name: "trim_poly_g".to_string(),
                    param_type: "bool".to_string(),
                    default: Some("false".to_string()),
                    aliases: Vec::new(),
                },
            ],
            metrics: Vec::new(),
            description: None,
            environment_requirements: Default::default(),
            report_contracts: vec![bijux_dna_core::contract::StageReportContract {
                report_id: "fastq.trim_reads.report".to_string(),
                kind: bijux_dna_core::contract::StageReportKind::Qc,
                schema_version: "bijux.fastq.trim_reads.report.v1".to_string(),
                required_fields: vec!["stage_id".to_string(), "tool_id".to_string()],
                advisory_fields: vec!["notes".to_string()],
                severity: bijux_dna_core::contract::ReportSeverity::Warning,
            }],
            capability_contract: StageCapabilitySpec {
                layouts: vec![ReadLayoutMode::PairedEnd],
                compression: vec![bijux_dna_core::contract::CompressionSupport::Gzip],
                reference: bijux_dna_core::contract::ReferenceRequirement::None,
                index: bijux_dna_core::contract::IndexRequirement::None,
                output_formats: vec!["fastq.gz".to_string(), "json".to_string()],
                unsupported_parameter_combinations: vec![UnsupportedParameterCombination {
                    parameters: BTreeMap::from([(
                        "adapter_preset".to_string(),
                        "legacy-single-end".to_string(),
                    )]),
                    reason: Some(
                        "legacy preset is not admitted for paired-end governed runs".to_string(),
                    ),
                }],
            },
            refusal_codes: vec![
                bijux_dna_core::contract::StageRefusalCode::UnsupportedLayout,
                bijux_dna_core::contract::StageRefusalCode::UnsafeOverride,
            ],
            operating_mode: StageOperatingMode::Enforced,
            behavior: StageBehavior::default(),
            image_requirements: None,
            extends: None,
        }
    }

    #[test]
    fn build_stage_plan_copies_run_params_into_effective_params() {
        let run_spec = RunSpec {
            stage: StageId::from_static(id_catalog::FASTQ_TRIM),
            tool: ToolId::from_static("fastp"),
            paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::from("work") },
            params: BTreeMap::from([
                ("quality".to_string(), "20".to_string()),
                ("trim_poly_g".to_string(), "true".to_string()),
            ]),
        };
        let tool_manifest = tool_manifest();
        let stage_spec = stage_spec(StageId::from_static(id_catalog::FASTQ_TRIM));

        let plan = match super::build_stage_plan(
            &run_spec,
            &tool_manifest,
            &stage_spec,
            &PathBuf::from("runs/run-1"),
            vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                PathBuf::from("reads.fastq.gz"),
                ArtifactRole::Reads,
            )],
            vec![ArtifactRef::required(
                ArtifactId::new("trimmed_reads"),
                PathBuf::from("trimmed.fastq.gz"),
                ArtifactRole::TrimmedReads,
            )],
        ) {
            Ok(plan) => plan,
            Err(error) => panic!("build stage plan failed: {error}"),
        };

        assert_eq!(plan.params, serde_json::json!({"quality": "20", "trim_poly_g": "true"}));
        assert_eq!(
            plan.effective_params,
            serde_json::json!({"min_quality": "20", "trim_poly_g": "true"})
        );
        assert_eq!(plan.operating_mode, StageOperatingMode::Enforced);
        assert!(plan.canonical_contract.is_some());
        assert!(plan.provenance.is_some());
    }

    #[test]
    fn validate_stage_outputs_rejects_stage_id_mismatch() {
        let run_spec = RunSpec {
            stage: StageId::from_static(id_catalog::FASTQ_TRIM),
            tool: ToolId::from_static("fastp"),
            paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::from("work") },
            params: BTreeMap::new(),
        };
        let stage_spec = stage_spec(StageId::from_static(id_catalog::FASTQ_FILTER));

        let error = match super::validate_stage_outputs(&stage_spec, &run_spec) {
            Ok(()) => panic!("stage output contract must match the requested run stage"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("output contract belongs to"));
    }

    #[test]
    fn validate_stage_outputs_rejects_duplicate_output_names() {
        let run_spec = RunSpec {
            stage: StageId::from_static(id_catalog::FASTQ_TRIM),
            tool: ToolId::from_static("fastp"),
            paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::from("work") },
            params: BTreeMap::new(),
        };
        let mut stage_spec = stage_spec(StageId::from_static(id_catalog::FASTQ_TRIM));
        stage_spec.outputs.push(PortSpec {
            name: "trimmed_reads".to_string(),
            data_type: "fastq".to_string(),
            cardinality: Cardinality::One,
            artifact_role: ArtifactRole::TrimmedReads,
        });

        let error = match super::validate_stage_outputs(&stage_spec, &run_spec) {
            Ok(()) => panic!("duplicate output names must fail validation"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("duplicate output contract entry"));
    }

    #[test]
    fn evaluate_stage_admission_returns_stable_refusal_codes() {
        let mut contract = stage_spec(StageId::from_static(id_catalog::FASTQ_TRIM))
            .canonical_contract(&tool_manifest());
        contract.capability_contract.reference =
            bijux_dna_core::contract::ReferenceRequirement::None;
        contract.capability_contract.index = bijux_dna_core::contract::IndexRequirement::None;
        let outcome = super::evaluate_stage_admission(
            &contract,
            &super::StageAdmissionRequestV1 {
                requested_mode: StageOperatingMode::Enforced,
                read_layout: Some(ReadLayoutMode::SingleEnd),
                compression: Some(bijux_dna_core::contract::CompressionSupport::Bgzf),
                requires_reference: true,
                requires_index: true,
                allow_unsafe_override: false,
                backend_available: false,
                scientifically_coherent: false,
            },
        );

        let codes = outcome.refusals.iter().map(|refusal| refusal.code).collect::<Vec<_>>();
        assert!(!outcome.admitted);
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::MissingReference));
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::MissingIndex));
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::UnsupportedLayout));
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::IncompatibleInputs));
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::UnsafeOverride));
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::BackendUnavailable));
        assert!(codes.contains(&bijux_dna_core::contract::StageRefusalCode::ScientificIncoherence));
    }
}
