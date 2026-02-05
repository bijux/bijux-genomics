use std::collections::BTreeSet;
use std::path::Path;

use bijux_analyze::StageMetricRegistry;
use bijux_core::{Cardinality, PortSpec};
use bijux_runtime::manifests::load_manifests;

fn domain_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain")
}

fn port_map(ports: &[PortSpec]) -> std::collections::BTreeMap<String, &PortSpec> {
    ports.iter().map(|port| (port.name.clone(), port)).collect()
}

fn cardinality_eq(a: Cardinality, b: Cardinality) -> bool {
    matches!(
        (a, b),
        (Cardinality::One, Cardinality::One) | (Cardinality::Many, Cardinality::Many)
    )
}

#[test]
fn tool_contracts_match_stage_inputs_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    for (stage_id, stage) in registry.stages() {
        let stage_inputs = port_map(&stage.inputs);
        let stage_outputs = port_map(&stage.outputs);
        for tool in registry.tools_for_stage(stage_id) {
            for required in &tool.execution_contract.required_inputs {
                if stage_inputs.contains_key(required) {
                    continue;
                }
                let is_r1_r2 = required.ends_with("_r1") || required.ends_with("_r2");
                let has_single_fastq = stage_inputs
                    .values()
                    .any(|port| port.data_type == "fastq" && stage.inputs.len() == 1);
                assert!(
                    is_r1_r2 && has_single_fastq,
                    "tool {} in {} requires unknown input {}",
                    tool.tool_id,
                    stage_id,
                    required
                );
            }
            for output in &tool.outputs {
                let Some(stage_output) = stage_outputs.get(&output.name) else {
                    panic!(
                        "tool {} in {} declares unknown output {}",
                        tool.tool_id, stage_id, output.name
                    );
                };
                assert_eq!(
                    stage_output.data_type, output.data_type,
                    "tool {} output {} data_type mismatch",
                    tool.tool_id, output.name
                );
                assert!(
                    cardinality_eq(stage_output.cardinality, output.cardinality),
                    "tool {} output {} cardinality mismatch",
                    tool.tool_id,
                    output.name
                );
            }
        }
    }
    Ok(())
}

#[test]
fn stage_metrics_align_with_bench_schema() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    for stage in registry.stages().values() {
        let Some(spec) = StageMetricRegistry::spec_for_stage(&stage.stage_id) else {
            continue;
        };
        let stage_metrics: BTreeSet<String> =
            stage.metrics.iter().map(|m| m.name.clone()).collect();
        let expected: BTreeSet<String> = spec
            .metrics
            .iter()
            .map(|metric_id| bijux_analyze::metric_spec(*metric_id).name.to_string())
            .collect();
        assert_eq!(
            stage_metrics, expected,
            "stage {} metrics mismatch",
            stage.stage_id
        );
    }
    Ok(())
}
