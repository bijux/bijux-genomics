use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageOperatingMode};
use bijux_dna_core::ids::{ArtifactId, StageId, ToolId};
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{
    canonical_downstream_probes, canonical_tool_bindings, evaluate_downstream_probe,
    DownstreamProbeAssertionV1, StagePluginOutputV1, ToolBinding,
};

#[test]
fn canonical_probes_close_every_stage_without_tool_dispatch() -> Result<()> {
    let bindings = canonical_tool_bindings()?;
    let probes = canonical_downstream_probes()?;
    assert_eq!(bindings.bindings().len(), 159);
    assert_eq!(probes.probes().len(), 71);

    for binding in bindings.bindings() {
        let probe = probes
            .probe_for_stage(binding.stage_id())
            .with_context(|| format!("missing probe for {}", binding.stage_id()))?;
        assert_eq!(probe.probe_id, binding.downstream_probe_id());
        assert_eq!(probe.input_contract_id, binding.normalized_output_contract_id());
    }

    let coverage = probes
        .probe_for_stage(&StageId::new("bam.coverage"))
        .context("missing BAM coverage probe")?;
    let coverage_bindings = bindings.bindings_for_stage(&StageId::new("bam.coverage"));
    assert_eq!(coverage_bindings.len(), 3);
    assert!(coverage_bindings
        .iter()
        .all(|binding| binding.downstream_probe_id() == coverage.probe_id));
    Ok(())
}

#[test]
fn semantic_probe_accepts_a_complete_tool_independent_output() -> Result<()> {
    let bindings = canonical_tool_bindings()?;
    let probes = canonical_downstream_probes()?;
    let binding = bindings
        .binding(&StageId::new("fastq.filter_reads"), &ToolId::new("fastp"))
        .context("missing FASTQ filter binding")?;
    let probe = probes.probe_for_stage(binding.stage_id()).context("missing FASTQ filter probe")?;

    let evaluation = evaluate_downstream_probe(probe, binding, &passing_output(binding))?;
    assert!(evaluation.passed);
    assert_eq!(evaluation.checks.len(), 4);
    assert!(evaluation.checks.iter().all(|check| check.passed));
    Ok(())
}

#[test]
fn semantic_probe_preserves_metric_artifact_and_verdict_failures() -> Result<()> {
    let bindings = canonical_tool_bindings()?;
    let probes = canonical_downstream_probes()?;
    let binding = bindings
        .binding(&StageId::new("fastq.filter_reads"), &ToolId::new("fastp"))
        .context("missing FASTQ filter binding")?;
    let probe = probes.probe_for_stage(binding.stage_id()).context("missing FASTQ filter probe")?;
    let mut output = passing_output(binding);
    output.metrics.metrics = serde_json::Value::Null;
    output.artifacts.clear();
    output.invariants[0].status = InvariantStatusV1::Fail;
    output.verdict.as_mut().context("missing verdict")?.verdict = InvariantStatusV1::Fail;

    let evaluation = evaluate_downstream_probe(probe, binding, &output)?;
    assert!(!evaluation.passed);
    for assertion in [
        DownstreamProbeAssertionV1::ScientificMetrics,
        DownstreamProbeAssertionV1::DeclaredArtifacts,
        DownstreamProbeAssertionV1::ScientificInvariants,
    ] {
        assert!(evaluation
            .checks
            .iter()
            .any(|check| check.assertion == assertion && !check.passed));
    }
    Ok(())
}

#[test]
fn semantic_probe_rejects_incomplete_authority_and_undeclared_artifacts() -> Result<()> {
    let bindings = canonical_tool_bindings()?;
    let probes = canonical_downstream_probes()?;
    let binding = bindings
        .binding(&StageId::new("fastq.filter_reads"), &ToolId::new("fastp"))
        .context("missing FASTQ filter binding")?;
    let mut probe =
        probes.probe_for_stage(binding.stage_id()).context("missing FASTQ filter probe")?.clone();
    probe.assertions.pop();
    assert!(evaluate_downstream_probe(&probe, binding, &passing_output(binding))
        .is_err_and(|error| error.to_string().contains("complete semantic assertion set")));

    let probe = probes.probe_for_stage(binding.stage_id()).context("missing FASTQ filter probe")?;
    let mut output = passing_output(binding);
    output.artifacts.push(
        ArtifactRef::required(
            ArtifactId::new("undeclared_output"),
            PathBuf::from("artifacts/undeclared-output"),
            ArtifactRole::Unknown,
        )
        .with_artifact_contract("bijux.auxiliary.artifact.json.v1"),
    );
    let evaluation = evaluate_downstream_probe(probe, binding, &output)?;
    assert!(!evaluation.passed);
    assert!(evaluation.checks.iter().any(|check| {
        check.assertion == DownstreamProbeAssertionV1::DeclaredArtifacts && !check.passed
    }));
    Ok(())
}

fn passing_output(binding: &impl ToolBinding) -> StagePluginOutputV1 {
    let artifacts = binding
        .outputs()
        .iter()
        .map(|output| {
            ArtifactRef::required(
                ArtifactId::new(&output.name),
                PathBuf::from(format!("artifacts/{}", output.name)),
                ArtifactRole::Unknown,
            )
            .with_artifact_contract(&output.artifact_contract_id)
        })
        .collect();
    StagePluginOutputV1 {
        metrics: MetricsEnvelope {
            schema_version: "bijux.metrics.envelope.v1".to_string(),
            contract_version: bijux_dna_core::contract::ContractVersion::v1(),
            stage_id: binding.stage_id().as_str().to_string(),
            stage_version: 1,
            tool_id: binding.tool_id().as_str().to_string(),
            tool_version: "test-version".to_string(),
            image_digest: "sha256:test".to_string(),
            parameters_fingerprint: "parameters".to_string(),
            input_fingerprint: "inputs".to_string(),
            parameters_json_normalized: serde_json::json!({}),
            input_hashes: vec!["input-sha256".to_string()],
            metric_provenance: None,
            metrics: serde_json::json!({"retained_reads": 1}),
        },
        artifacts,
        operating_mode: StageOperatingMode::default(),
        report_parts: Vec::new(),
        warnings: Vec::new(),
        invariants: vec![InvariantResultV1 {
            id: "retained_reads_nonnegative".to_string(),
            status: InvariantStatusV1::Pass,
            message: "retained reads are nonnegative".to_string(),
            remediation: None,
        }],
        verdict: Some(StageVerdictV1 {
            stage_id: binding.stage_id().as_str().to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: vec!["semantic invariants passed".to_string()],
            key_metrics: serde_json::json!({"retained_reads": 1}),
        }),
        event_hints: Vec::new(),
    }
}
