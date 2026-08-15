use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_dna_bench::BenchmarkStagePluginRegistry;
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::{
    canonical_tool_bindings, PlanDecisionReason, StageInvocationV1, StagePlanV1, StagePlugin,
    StagePluginOutputV1, ToolBinding,
};

struct PrefixPlugin {
    prefix: &'static str,
    fail_semantics: bool,
}

impl StagePlugin for PrefixPlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with(self.prefix)
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        Ok(StageInvocationV1 {
            command: plan.command.template.clone(),
            env: BTreeMap::new(),
            expected_outputs: plan.io.outputs.clone(),
        })
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        let artifacts = if outputs.is_empty() { plan.io.outputs.clone() } else { outputs.to_vec() };
        let status =
            if self.fail_semantics { InvariantStatusV1::Fail } else { InvariantStatusV1::Pass };
        Ok(StagePluginOutputV1 {
            metrics: MetricsEnvelope {
                schema_version: "bijux.metrics_envelope.v2".to_string(),
                contract_version: bijux_dna_core::contract::ContractVersion::v1(),
                stage_id: plan.stage_id.as_str().to_string(),
                stage_version: plan.stage_version.0,
                tool_id: plan.tool_id.as_str().to_string(),
                tool_version: plan.tool_version.clone(),
                image_digest: plan.image.digest.clone().unwrap_or_else(|| plan.image.image.clone()),
                parameters_fingerprint: "parameters".to_string(),
                input_fingerprint: "inputs".to_string(),
                parameters_json_normalized: plan.params.clone(),
                input_hashes: Vec::new(),
                metric_provenance: None,
                metrics: serde_json::json!({"semantic_observation_count": 1}),
            },
            artifacts,
            operating_mode: plan.operating_mode,
            report_parts: Vec::new(),
            warnings: Vec::new(),
            invariants: vec![InvariantResultV1 {
                id: "injected_plugin_semantics".to_string(),
                status: status.clone(),
                message: "injected plugin semantic result".to_string(),
                remediation: None,
            }],
            verdict: Some(StageVerdictV1 {
                stage_id: plan.stage_id.as_str().to_string(),
                verdict: status,
                reasons: vec!["injected plugin semantic result".to_string()],
                key_metrics: serde_json::json!({"semantic_observation_count": 1}),
            }),
            event_hints: Vec::new(),
        })
    }
}

#[test]
fn injected_plugins_close_all_71_binding_backed_stages() -> Result<()> {
    let fastq = PrefixPlugin { prefix: "fastq.", fail_semantics: false };
    let bam = PrefixPlugin { prefix: "bam.", fail_semantics: false };
    let vcf = PrefixPlugin { prefix: "vcf.", fail_semantics: false };
    let plugins = BenchmarkStagePluginRegistry::new(vec![&fastq, &bam, &vcf]);
    plugins.validate_for_bindings(&canonical_tool_bindings()?)
}

#[test]
fn plugin_ownership_must_be_complete_and_unambiguous() -> Result<()> {
    let bindings = canonical_tool_bindings()?;
    let fastq = PrefixPlugin { prefix: "fastq.", fail_semantics: false };
    assert!(BenchmarkStagePluginRegistry::new(vec![&fastq])
        .validate_for_bindings(&bindings)
        .is_err_and(|error| error.to_string().contains("no injected stage plugin")));

    let all = PrefixPlugin { prefix: "", fail_semantics: false };
    assert!(BenchmarkStagePluginRegistry::new(vec![&all, &all])
        .validate_for_bindings(&bindings)
        .is_err_and(|error| error.to_string().contains("ambiguous")));
    Ok(())
}

#[test]
fn generic_normalization_preserves_scientific_failure_without_dispatch_error() -> Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("campaign-plugin-normalization");
    let bindings = canonical_tool_bindings()?;
    let plugin = PrefixPlugin { prefix: "vcf.", fail_semantics: true };
    let registry = BenchmarkStagePluginRegistry::new(vec![&plugin]);
    let binding = bindings
        .binding(&StageId::new("vcf.filter"), &ToolId::new("bcftools"))
        .ok_or_else(|| anyhow!("missing canonical VCF filter binding"))?;
    let plan = plan_for_binding(binding, sandbox.path());
    let normalized = registry.normalize(&bindings, &plan, &plan.io.outputs)?;
    assert!(!normalized.downstream.passed);
    assert_eq!(
        normalized.output.verdict.ok_or_else(|| anyhow!("missing verdict"))?.verdict,
        InvariantStatusV1::Fail
    );
    Ok(())
}

#[test]
fn generic_materialization_rejects_output_authority_drift_before_plugin_call() -> Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("campaign-plugin-materialization");
    let bindings = canonical_tool_bindings()?;
    let plugin = PrefixPlugin { prefix: "vcf.", fail_semantics: false };
    let registry = BenchmarkStagePluginRegistry::new(vec![&plugin]);
    let binding = bindings
        .binding(&StageId::new("vcf.filter"), &ToolId::new("bcftools"))
        .ok_or_else(|| anyhow!("missing canonical VCF filter binding"))?;
    let mut plan = plan_for_binding(binding, sandbox.path());
    plan.io.outputs[0].artifact_contract_id = Some("bijux.invalid.artifact.v1".to_string());
    assert!(registry
        .materialize(&bindings, &plan)
        .is_err_and(|error| error.to_string().contains("differ from binding authority")));
    Ok(())
}

fn plan_for_binding(binding: &impl ToolBinding, output_root: &std::path::Path) -> StagePlanV1 {
    let outputs = binding
        .outputs()
        .iter()
        .map(|output| {
            ArtifactRef::required(
                ArtifactId::new(&output.name),
                output_root.join(&output.name),
                ArtifactRole::Unknown,
            )
            .with_artifact_contract(&output.artifact_contract_id)
        })
        .collect();
    StagePlanV1 {
        stage_id: binding.stage_id().clone(),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: binding.tool_id().clone(),
        tool_version: "test-version".to_string(),
        image: ContainerImageRefV1 {
            image: binding.tool_id().as_str().to_string(),
            digest: Some("sha256:test".to_string()),
        },
        command: CommandSpecV1 { template: binding.command_template().to_vec() },
        resources: ToolConstraints::default(),
        io: StageIO { inputs: Vec::new(), outputs },
        out_dir: output_root.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::default(),
        aux_images: BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    }
}
