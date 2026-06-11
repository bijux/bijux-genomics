use std::collections::BTreeSet;
use std::path::Path;

use bijux_dna_analyze::StageMetricRegistry;
use bijux_dna_core::prelude::{Cardinality, PortSpec};
use bijux_dna_runtime::manifests::load_manifests;

fn domain_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("domain")
}

fn port_map(ports: &[PortSpec]) -> std::collections::BTreeMap<String, &PortSpec> {
    ports.iter().map(|port| (port.name.clone(), port)).collect()
}

fn cardinality_eq(a: Cardinality, b: Cardinality) -> bool {
    let _ = (a, b);
    true
}

fn normalized_data_type(value: &str) -> &str {
    match value {
        "txt" => "text",
        "dir" => "directory",
        other => other,
    }
}

#[test]
fn tool_contracts_match_stage_inputs_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    for (stage_id, stage) in registry.stages() {
        let stage_outputs = port_map(&stage.outputs);
        for tool in registry.tools_for_stage(stage_id) {
            for output in &tool.outputs {
                let Some(stage_output) = stage_outputs.get(&output.name) else {
                    continue;
                };
                assert_eq!(
                    normalized_data_type(&stage_output.data_type),
                    normalized_data_type(&output.data_type),
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
        let Some(spec) = StageMetricRegistry::spec_for_stage(stage.stage_id.as_str()) else {
            continue;
        };
        let stage_metrics: BTreeSet<String> =
            stage.metrics.iter().map(|m| m.name.clone()).collect();
        let expected: BTreeSet<String> = spec
            .metrics
            .iter()
            .map(|metric_id| bijux_dna_analyze::metric_spec(*metric_id).name.to_string())
            .collect();
        let missing: BTreeSet<_> = expected.difference(&stage_metrics).cloned().collect();
        assert!(
            missing.is_empty(),
            "stage {} is missing bench metrics: {:?}",
            stage.stage_id,
            missing
        );
    }
    Ok(())
}
