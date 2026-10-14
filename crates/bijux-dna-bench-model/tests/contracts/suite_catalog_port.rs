use std::collections::BTreeSet;

use bijux_dna_bench_model::contract::{validate_suite_with_catalog, BenchmarkSuiteCatalog};
use bijux_dna_bench_model::{
    AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageEdge, BenchmarkStageSpec,
    BenchmarkSuiteSpec, DatasetSpec, DiversityRequirements, ReplicatePolicy,
};
use bijux_dna_core::ids::{StageId, ToolId};

struct FixtureCatalog {
    stage: bool,
    ports: bool,
    parameters: bool,
    missing_contract: Option<&'static str>,
}

impl BenchmarkSuiteCatalog for FixtureCatalog {
    fn declares_stage(&self, stage: &str) -> bool {
        self.stage && stage == "fastq.fixture"
    }
    fn stage_is_plannable(&self, _: &StageId) -> Option<bool> {
        Some(true)
    }
    fn declares_binding(&self, _: &StageId, tool: &ToolId) -> bool {
        tool.as_str() == "fixture"
    }
    fn admitted_tools(&self, _: &StageId) -> Vec<ToolId> {
        match bijux_dna_core::ids::parse_tool_id("fixture") {
            Ok(tool) => vec![tool],
            Err(error) => panic!("invalid fixture tool: {error}"),
        }
    }
    fn input_ids(&self, _: &str) -> Option<BTreeSet<String>> {
        if self.missing_contract == Some("input") {
            return None;
        }
        Some(if self.ports { BTreeSet::from(["reads".to_string()]) } else { BTreeSet::new() })
    }
    fn output_ids(&self, _: &str) -> Option<BTreeSet<String>> {
        if self.missing_contract == Some("output") {
            return None;
        }
        Some(if self.ports { BTreeSet::from(["reads".to_string()]) } else { BTreeSet::new() })
    }
    fn parameter_ids(&self, _: &str) -> Option<BTreeSet<String>> {
        if self.missing_contract == Some("parameter") {
            return None;
        }
        Some(if self.parameters { BTreeSet::from(["limit".to_string()]) } else { BTreeSet::new() })
    }
}

fn suite() -> BenchmarkSuiteSpec {
    let stages = ["source", "sink"].map(|instance| BenchmarkStageSpec {
        stage: "fastq.fixture".to_string(),
        stage_instance_id: Some(instance.to_string()),
        tools: vec!["fixture".to_string()],
        params: Vec::new(),
        upstream_stage_instance_ids: Vec::new(),
        param_bindings: vec![BenchmarkParamBinding {
            stage_instance_id: None,
            tool: None,
            values: std::collections::BTreeMap::from([("limit".to_string(), serde_json::json!(1))]),
        }],
    });
    let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
        "fixture-suite".to_string(),
        vec![DatasetSpec {
            id: "fixture-data".to_string(),
            hash: "fixture-hash".to_string(),
            size: 1,
            origin: "synthetic".to_string(),
            class_label: "fixture".to_string(),
            read_layout: "paired".to_string(),
        }],
        stages.to_vec(),
        ReplicatePolicy { count: 1, warmup: 0, seeds: vec![1] },
        DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
        Vec::new(),
        AnalysisRequirements {
            require_bootstrap: false,
            require_outlier_detection: false,
            min_replicates_for_bootstrap: 5,
        },
    );
    suite.edges.push(BenchmarkStageEdge {
        from: "source".to_string(),
        to: "sink".to_string(),
        from_output_id: Some("reads".to_string()),
        to_input_id: Some("reads".to_string()),
    });
    suite
}

#[test]
fn suite_catalog_port_admits_explicit_stage_tool_ports_and_parameters() -> anyhow::Result<()> {
    validate_suite_with_catalog(
        &suite(),
        &FixtureCatalog { stage: true, ports: true, parameters: true, missing_contract: None },
    )?;
    Ok(())
}

#[test]
fn suite_catalog_port_rejects_missing_stage_observations() {
    assert!(validate_suite_with_catalog(
        &suite(),
        &FixtureCatalog { stage: false, ports: true, parameters: true, missing_contract: None }
    )
    .is_err());
}

#[test]
fn suite_catalog_port_rejects_incompatible_ports_and_parameters() {
    for catalog in [
        FixtureCatalog { stage: true, ports: false, parameters: true, missing_contract: None },
        FixtureCatalog { stage: true, ports: true, parameters: false, missing_contract: None },
    ] {
        assert!(validate_suite_with_catalog(&suite(), &catalog).is_err());
    }
}

#[test]
fn suite_catalog_port_rejects_missing_contracts_for_declared_stages() {
    for (missing, diagnostic) in [
        ("input", "governed input port contract"),
        ("output", "governed output port contract"),
        ("parameter", "governed parameter contract"),
    ] {
        let catalog = FixtureCatalog {
            stage: true,
            ports: true,
            parameters: true,
            missing_contract: Some(missing),
        };
        match validate_suite_with_catalog(&suite(), &catalog) {
            Err(error) => assert!(error.to_string().contains(diagnostic), "{error}"),
            Ok(()) => panic!("missing {missing} contract was admitted"),
        }
    }
}

#[test]
fn suite_catalog_port_does_not_require_unused_contracts() -> anyhow::Result<()> {
    let mut request = suite();
    request.edges.clear();
    for stage in &mut request.stages {
        stage.param_bindings.clear();
    }
    for missing in ["input", "output", "parameter"] {
        validate_suite_with_catalog(
            &request,
            &FixtureCatalog {
                stage: true,
                ports: true,
                parameters: true,
                missing_contract: Some(missing),
            },
        )?;
    }
    Ok(())
}

#[test]
fn suite_catalog_port_rejects_stage_and_tool_nodes_sharing_one_identity() -> anyhow::Result<()> {
    let catalog =
        FixtureCatalog { stage: true, ports: true, parameters: true, missing_contract: None };
    let mut request = suite();
    validate_suite_with_catalog(&request, &catalog)?;
    let collision = request.stages[0].tool_node_id("fixture");
    request.stages[1].stage_instance_id = Some(collision.clone());
    request.edges[0].to = collision.clone();
    let nodes = request.graph_nodes();
    assert!(nodes.iter().any(|node| {
        node.node_id == collision
            && node.kind == bijux_dna_bench_model::BenchmarkGraphNodeKind::Stage
    }));
    assert!(nodes.iter().any(|node| {
        node.node_id == collision
            && node.kind == bijux_dna_bench_model::BenchmarkGraphNodeKind::StageTool
    }));
    assert!(
        validate_suite_with_catalog(&request, &catalog).is_err(),
        "suite admitted one identity for a declared stage and another stage's tool node"
    );
    Ok(())
}
