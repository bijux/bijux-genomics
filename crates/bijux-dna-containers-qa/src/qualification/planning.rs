use anyhow::{bail, Result};
use bijux_dna_containers::api::ContainerPipelineContract;
use serde_json::{json, Value};

use super::{
    ContainerQualificationPlan, QualificationInputBind, QualificationMetricPlan,
    QualificationProbeKind, QualificationProbePlan,
};

/// Derive the QA-owned probe and acceptance plan from an admitted container pipeline.
///
/// # Errors
/// Returns an error when metric operators cannot be serialized or benchmark measurement authority
/// is presented as container qualification.
pub fn plan_container_qualification(
    contract: ContainerPipelineContract,
) -> Result<ContainerQualificationPlan> {
    match contract {
        ContainerPipelineContract::Legacy(pipeline) => Ok(ContainerQualificationPlan {
            pipeline_schema: pipeline.schema_version,
            image_path: pipeline.image.path,
            image_sha256: pipeline.image.sha256,
            inputs: serde_json::to_value(&pipeline.data.inputs)?,
            input_binds: pipeline
                .data
                .inputs
                .iter()
                .map(|input| QualificationInputBind {
                    source: input.path.clone(),
                    target: input.path.clone(),
                })
                .collect(),
            probes: probe_plan(
                pipeline.commands.version,
                pipeline.commands.help,
                pipeline.commands.real,
                pipeline.commands.negative,
            ),
            output_mount: pipeline.outputs.result_root,
            required_files: pipeline.outputs.required_files,
            version_regex: pipeline.acceptance.version_regex,
            metrics: Vec::new(),
            execution: serde_json::to_value(pipeline.execution)?,
            dependencies: json!([]),
            parameters: Value::Null,
            parameter_files: Vec::new(),
            interface: Value::Null,
        }),
        ContainerPipelineContract::Current(pipeline) => {
            if pipeline.measurement.is_some() {
                bail!("qualification planning cannot carry benchmark measurement authority");
            }
            let metrics = pipeline
                .acceptance
                .metrics
                .iter()
                .map(|metric| -> Result<QualificationMetricPlan> {
                    Ok(QualificationMetricPlan {
                        id: metric.id.clone(),
                        path: metric.path.clone(),
                        json_pointer: metric.json_pointer.clone(),
                        operator: serde_json::to_value(metric.operator)?
                            .as_str()
                            .ok_or_else(|| {
                                anyhow::anyhow!("serialize qualification metric operator")
                            })?
                            .to_string(),
                        expected: metric.expected.clone(),
                        unit: metric.unit.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let parameter_files =
                pipeline.parameters.as_ref().map_or_else(Vec::new, |parameters| {
                    vec![parameters.overrides_file.clone(), parameters.effective_file.clone()]
                });
            Ok(ContainerQualificationPlan {
                pipeline_schema: pipeline.schema_version,
                image_path: pipeline.image.path,
                image_sha256: pipeline.image.sha256,
                inputs: serde_json::to_value(&pipeline.data.inputs)?,
                input_binds: pipeline
                    .data
                    .inputs
                    .iter()
                    .map(|input| QualificationInputBind {
                        source: input.path.clone(),
                        target: input.mount_path.clone(),
                    })
                    .collect(),
                probes: probe_plan(
                    pipeline.commands.version,
                    pipeline.commands.help,
                    pipeline.commands.real,
                    pipeline.commands.negative,
                ),
                output_mount: pipeline.outputs.mount_path,
                required_files: pipeline.outputs.required_files,
                version_regex: pipeline.acceptance.version_regex,
                metrics,
                execution: serde_json::to_value(pipeline.execution)?,
                dependencies: serde_json::to_value(pipeline.dependencies)?,
                parameters: serde_json::to_value(pipeline.parameters)?,
                parameter_files,
                interface: serde_json::to_value(pipeline.interface)?,
            })
        }
    }
}

fn probe_plan(
    version: Vec<String>,
    help: Vec<String>,
    real: Vec<String>,
    negative: Vec<String>,
) -> Vec<QualificationProbePlan> {
    [
        (QualificationProbeKind::Version, version),
        (QualificationProbeKind::Help, help),
        (QualificationProbeKind::Real, real),
        (QualificationProbeKind::Negative, negative),
    ]
    .into_iter()
    .map(|(kind, arguments)| QualificationProbePlan { kind, arguments })
    .collect()
}
