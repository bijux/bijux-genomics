use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use bijux_dna_core::prelude::invariants::InvariantStatusV1;

use super::{
    registry::validate_probe, DownstreamProbe, DownstreamProbeAssertionV1, DownstreamProbeCheck,
    DownstreamProbeEvaluation,
};
use crate::{validate_report_parts, StagePluginOutputV1, ToolBinding};

/// Applies a tool-independent consumer probe to one normalized producer output.
///
/// Contract-authority mismatches are errors. Scientific or output failures are preserved in a
/// deterministic failed outcome for the benchmark evidence owner without weakening checks.
///
/// # Errors
/// Returns an error when the probe, binding, or producer identities do not describe one contract.
pub fn evaluate_downstream_probe(
    probe: &impl DownstreamProbe,
    binding: &impl ToolBinding,
    output: &StagePluginOutputV1,
) -> Result<DownstreamProbeEvaluation> {
    validate_probe(probe)?;
    if probe.producer_stage_id() != binding.stage_id()
        || probe.probe_id() != binding.downstream_probe_id()
        || probe.input_contract_id() != binding.normalized_output_contract_id()
    {
        bail!("downstream probe authority differs from the selected tool binding");
    }

    let checks = vec![
        normalized_identity_check(binding, output),
        scientific_metrics_check(output),
        declared_artifacts_check(binding, output),
        scientific_invariants_check(binding, output),
    ];
    Ok(DownstreamProbeEvaluation {
        probe_id: probe.probe_id().to_string(),
        binding_id: binding.binding_id().to_string(),
        producer_stage_id: binding.stage_id().clone(),
        tool_id: binding.tool_id().clone(),
        passed: checks.iter().all(|check| check.passed),
        checks,
    })
}

fn normalized_identity_check(
    binding: &impl ToolBinding,
    output: &StagePluginOutputV1,
) -> DownstreamProbeCheck {
    let metrics = &output.metrics;
    let values = [
        metrics.schema_version.as_str(),
        metrics.tool_version.as_str(),
        metrics.image_digest.as_str(),
        metrics.parameters_fingerprint.as_str(),
        metrics.input_fingerprint.as_str(),
    ];
    let passed = metrics.stage_id == binding.stage_id().as_str()
        && metrics.tool_id == binding.tool_id().as_str()
        && values.iter().all(|value| !value.trim().is_empty());
    check(
        DownstreamProbeAssertionV1::NormalizedIdentity,
        passed,
        "normalized metrics retain stage, tool, image, parameter, and input identities",
    )
}

fn scientific_metrics_check(output: &StagePluginOutputV1) -> DownstreamProbeCheck {
    let passed = output.metrics.metrics.as_object().is_some_and(|metrics| !metrics.is_empty());
    check(
        DownstreamProbeAssertionV1::ScientificMetrics,
        passed,
        "normalized scientific metrics are a non-empty named object",
    )
}

fn declared_artifacts_check(
    binding: &impl ToolBinding,
    output: &StagePluginOutputV1,
) -> DownstreamProbeCheck {
    let mut observed = BTreeMap::new();
    let mut duplicates = BTreeSet::new();
    for artifact in &output.artifacts {
        let name = artifact.name.as_str();
        if observed.insert(name, artifact.artifact_contract_id.as_deref()).is_some() {
            duplicates.insert(name);
        }
    }
    let passed = duplicates.is_empty()
        && observed.len() == binding.outputs().len()
        && binding.outputs().iter().all(|expected| {
            observed.get(expected.name.as_str()).is_some_and(|contract_id| {
                *contract_id == Some(expected.artifact_contract_id.as_str())
            })
        })
        && validate_report_parts(&output.report_parts).is_ok();
    check(
        DownstreamProbeAssertionV1::DeclaredArtifacts,
        passed,
        "every declared output has one exact artifact contract and report payloads conform",
    )
}

fn scientific_invariants_check(
    binding: &impl ToolBinding,
    output: &StagePluginOutputV1,
) -> DownstreamProbeCheck {
    let invariant_pass = !output.invariants.is_empty()
        && output
            .invariants
            .iter()
            .all(|invariant| !matches!(invariant.status, InvariantStatusV1::Fail));
    let verdict_pass = output.verdict.as_ref().is_some_and(|verdict| {
        verdict.stage_id == binding.stage_id().as_str()
            && !matches!(verdict.verdict, InvariantStatusV1::Fail)
    });
    check(
        DownstreamProbeAssertionV1::ScientificInvariants,
        invariant_pass && verdict_pass,
        "scientific invariants are present and the stage verdict is non-failing",
    )
}

fn check(
    assertion: DownstreamProbeAssertionV1,
    passed: bool,
    detail: &str,
) -> DownstreamProbeCheck {
    DownstreamProbeCheck { assertion, passed, detail: detail.to_string() }
}
