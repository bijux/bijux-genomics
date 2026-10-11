use std::collections::BTreeMap;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
use bijux_dna_core::prelude::invariants::InvariantStatusV1;
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::{
    canonical_downstream_probes, canonical_tool_bindings, evaluate_downstream_probe,
    PlanDecisionReason, StagePlanV1, StagePlugin, ToolBinding,
};
use bijux_dna_stages_vcf::VcfStagePlugin;

const MINIMAL_VCF: &[u8] = b"##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n1\t1\t.\tA\tG\t30\tPASS\t.\n";

fn plugin_stage_plan(
    stage_id: &str,
    tool_id: &str,
    output_id: &str,
    output_path: std::path::PathBuf,
) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::new(tool_id),
        tool_version: "1.20".to_string(),
        image: ContainerImageRefV1 {
            image: format!("{tool_id}.sif"),
            digest: Some(format!("sha256:{tool_id}")),
        },
        command: CommandSpecV1 {
            template: vec![tool_id.to_string(), "filter".to_string()],
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: Vec::new(),
            outputs: vec![ArtifactRef::required(
                ArtifactId::new(output_id),
                output_path,
                ArtifactRole::Variant,
            )],
        },
        out_dir: std::path::PathBuf::from("artifacts/vcf-plugin"),
        params: serde_json::json!({"minimum_quality": 30}),
        effective_params: serde_json::json!({"minimum_quality": 30}),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::default(),
        aux_images: BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    }
}

#[test]
fn vcf_stage_plugin_handles_every_domain_stage_without_tool_dispatch() {
    let plugin = VcfStagePlugin;
    assert!(bijux_dna_domain_vcf::VcfDomainStage::all()
        .iter()
        .all(|stage| plugin.handles_stage(stage.as_str())));
    assert!(!plugin.handles_stage("vcf.unknown"));
    assert!(!plugin.handles_stage("bam.align"));
}

#[test]
fn vcf_stage_plugin_materializes_declared_invocation() -> anyhow::Result<()> {
    let plugin = VcfStagePlugin;
    let plan = plugin_stage_plan(
        "vcf.filter",
        "bcftools",
        "filtered_vcf",
        "filtered.vcf.gz".into(),
    );
    let invocation = plugin.materialize(&plan)?;
    assert_eq!(invocation.command, plan.command.template);
    assert_eq!(invocation.expected_outputs, plan.io.outputs);
    assert!(invocation.env.is_empty());
    Ok(())
}

#[test]
fn vcf_stage_plugin_rejects_invalid_stage_and_command_authority() {
    let plugin = VcfStagePlugin;
    let mut unsupported = plugin_stage_plan(
        "vcf.filter",
        "bcftools",
        "filtered_vcf",
        "filtered.vcf.gz".into(),
    );
    unsupported.stage_id = StageId::new("vcf.unknown");
    assert!(plugin
        .materialize(&unsupported)
        .is_err_and(|error| error.to_string().contains("unsupported VCF stage")));

    let mut blank = plugin_stage_plan(
        "vcf.filter",
        "bcftools",
        "filtered_vcf",
        "filtered.vcf.gz".into(),
    );
    blank.command.template.push(" ".to_string());
    assert!(plugin
        .materialize(&blank)
        .is_err_and(|error| error.to_string().contains("blank command template argument")));
}

#[test]
fn vcf_stage_plugin_normalizes_materialized_outputs_and_semantic_verdict() -> anyhow::Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("vcf-stage-plugin-output");
    let output_path = sandbox.path().join("filtered.vcf");
    std::fs::write(&output_path, MINIMAL_VCF)?;
    let plugin = VcfStagePlugin;
    let plan = plugin_stage_plan("vcf.filter", "bcftools", "filtered_vcf", output_path);
    let normalized = plugin.parse_outputs(&plan, &plan.io.outputs)?;

    assert_eq!(normalized.metrics.stage_id, "vcf.filter");
    assert_eq!(normalized.metrics.tool_id, "bcftools");
    assert_eq!(normalized.metrics.metrics["parser_id"], "vcf.parser.vcf_output");
    assert!(normalized.invariants.iter().all(|invariant| invariant.status == InvariantStatusV1::Pass));
    assert_eq!(normalized.verdict.expect("stage verdict").verdict, InvariantStatusV1::Pass);
    assert_eq!(normalized.artifacts, plan.io.outputs);
    Ok(())
}

#[test]
fn vcf_stage_plugin_preserves_missing_outputs_as_failed_semantics() -> anyhow::Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("vcf-stage-plugin-missing-output");
    let plugin = VcfStagePlugin;
    let plan = plugin_stage_plan(
        "vcf.filter",
        "bcftools",
        "filtered_vcf",
        sandbox.path().join("missing-filtered.vcf.gz"),
    );
    let normalized = plugin.parse_outputs(&plan, &[])?;
    assert_eq!(normalized.verdict.expect("stage verdict").verdict, InvariantStatusV1::Fail);
    assert!(!normalized.warnings.is_empty());
    Ok(())
}

#[test]
fn vcf_stage_plugin_rejects_materialized_but_semantically_invalid_vcf() -> anyhow::Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("vcf-stage-plugin-invalid-output");
    let output_path = sandbox.path().join("invalid.vcf");
    std::fs::write(&output_path, b"not-a-vcf\n")?;
    let plan = plugin_stage_plan("vcf.filter", "bcftools", "filtered_vcf", output_path);
    let normalized = VcfStagePlugin.parse_outputs(&plan, &plan.io.outputs)?;

    assert!(normalized.invariants.iter().any(|invariant| {
        invariant.id == "vcf_stage_output_semantics"
            && invariant.status == InvariantStatusV1::Fail
    }));
    assert_eq!(normalized.verdict.expect("stage verdict").verdict, InvariantStatusV1::Fail);
    Ok(())
}

#[test]
fn vcf_stage_plugin_keeps_normalization_stable_across_tool_substitution() -> anyhow::Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("vcf-stage-plugin-substitution");
    let output_path = sandbox.path().join("imputed.vcf");
    std::fs::write(&output_path, MINIMAL_VCF)?;
    let plugin = VcfStagePlugin;
    let beagle = plugin.parse_outputs(
        &plugin_stage_plan("vcf.impute", "beagle", "imputed_vcf", output_path.clone()),
        &[ArtifactRef::required(
            ArtifactId::new("imputed_vcf"),
            output_path.clone(),
            ArtifactRole::Variant,
        )],
    )?;
    let glimpse = plugin.parse_outputs(
        &plugin_stage_plan("vcf.impute", "glimpse", "imputed_vcf", output_path.clone()),
        &[ArtifactRef::required(
            ArtifactId::new("imputed_vcf"),
            output_path,
            ArtifactRole::Variant,
        )],
    )?;
    assert_eq!(beagle.metrics.metrics, glimpse.metrics.metrics);
    assert_eq!(
        beagle.report_parts[0].payload["parser_id"],
        glimpse.report_parts[0].payload["parser_id"]
    );
    assert_eq!(
        beagle.report_parts[0].payload["artifacts"],
        glimpse.report_parts[0].payload["artifacts"]
    );
    assert_ne!(beagle.metrics.tool_id, glimpse.metrics.tool_id);
    Ok(())
}

#[test]
fn vcf_stage_plugin_output_closes_the_stage_owned_downstream_probe() -> anyhow::Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("vcf-stage-plugin-downstream-probe");
    let output_path = sandbox.path().join("filtered.vcf");
    std::fs::write(&output_path, MINIMAL_VCF)?;
    let bindings = canonical_tool_bindings()?;
    let binding = bindings
        .binding(&StageId::new("vcf.filter"), &ToolId::new("bcftools"))
        .expect("canonical VCF filter binding");
    let probes = canonical_downstream_probes()?;
    let probe = probes
        .probe_for_stage(binding.stage_id())
        .expect("canonical VCF filter downstream probe");
    let plan = plugin_stage_plan("vcf.filter", "bcftools", "filtered_vcf", output_path);
    let normalized = VcfStagePlugin.parse_outputs(&plan, &plan.io.outputs)?;
    let evaluation = evaluate_downstream_probe(probe, binding, &normalized)?;
    assert!(evaluation.passed);
    Ok(())
}

#[test]
fn vcf_stage_plugin_executes_every_declared_parser_family() -> anyhow::Result<()> {
    let sandbox = bijux_dna_testkit::tempdir_for("vcf-stage-plugin-parser-families");
    let cases = [
        (
            "vcf.call",
            "bcftools",
            "called_vcf",
            "called.vcf",
            MINIMAL_VCF,
            ArtifactRole::Variant,
        ),
        (
            "vcf.stats",
            "bcftools",
            "stats_report",
            "stats.tsv",
            b"variants_total\t1\nsample_count\t1\nsnps\t1\n",
            ArtifactRole::Report,
        ),
        (
            "vcf.qc",
            "plink2",
            "qc_report",
            "qc.json",
            b"{\"sample_count\":1}",
            ArtifactRole::ReportJson,
        ),
        (
            "vcf.imputation_metrics",
            "beagle",
            "imputation_metrics_json",
            "imputation-metrics.json",
            b"{\"masked_accuracy\":0.99}",
            ArtifactRole::ReportJson,
        ),
    ];

    for (stage_id, tool_id, output_id, file_name, bytes, role) in cases {
        let output_path = sandbox.path().join(file_name);
        std::fs::write(&output_path, bytes)?;
        let mut plan = plugin_stage_plan(stage_id, tool_id, output_id, output_path.clone());
        plan.io.outputs = vec![ArtifactRef::required(ArtifactId::new(output_id), output_path, role)];
        let normalized = VcfStagePlugin.parse_outputs(&plan, &plan.io.outputs)?;
        assert_eq!(
            normalized.verdict.expect("stage verdict").verdict,
            InvariantStatusV1::Pass,
            "declared parser family must accept {stage_id}"
        );
    }
    Ok(())
}
