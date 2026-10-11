#[cfg(feature = "bam_downstream")]
use std::collections::BTreeMap;
#[cfg(feature = "bam_downstream")]
use std::path::{Path, PathBuf};

#[cfg(feature = "bam_downstream")]
use bijux_dna_core::contract::PlanPolicy;
#[cfg(feature = "bam_downstream")]
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
#[cfg(feature = "bam_downstream")]
use bijux_dna_domain_bam::BamStage;
#[cfg(feature = "bam_downstream")]
use bijux_dna_planner_bam::{plan_bam_to_bam__adna_shotgun__v1, BamPipelineInputs};

#[cfg(feature = "bam_downstream")]
fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-planner-bam__{group}__{name}")
}

#[cfg(feature = "bam_downstream")]
fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["echo".to_string(), tool_id.to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[cfg(feature = "bam_downstream")]
fn normalize_graph_snapshot_paths(
    value: serde_json::Value,
    transient_root: &Path,
) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .into_iter()
                .map(|item| normalize_graph_snapshot_paths(item, transient_root))
                .collect(),
        ),
        serde_json::Value::Object(map) => {
            let normalized = map
                .into_iter()
                .map(|(key, value)| {
                    let value = match (key.as_str(), value) {
                        ("path", serde_json::Value::String(path)) => {
                            serde_json::Value::String(snapshot_leaf(&path))
                        }
                        ("out_dir", serde_json::Value::String(path)) => {
                            serde_json::Value::String(snapshot_leaf(&path))
                        }
                        (_, nested) => normalize_graph_snapshot_paths(nested, transient_root),
                    };
                    (key, value)
                })
                .collect();
            serde_json::Value::Object(normalized)
        }
        serde_json::Value::String(value) => {
            let transient_root = transient_root.to_string_lossy();
            if !value.contains(transient_root.as_ref()) {
                serde_json::Value::String(value)
            } else if value.chars().any(char::is_whitespace) {
                serde_json::Value::String(value.replace(transient_root.as_ref(), "<TMPDIR>/<TMP>"))
            } else {
                serde_json::Value::String(snapshot_leaf(&value))
            }
        }
        other => other,
    }
}

#[cfg(feature = "bam_downstream")]
fn snapshot_leaf(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    PathBuf::from(trimmed)
        .file_name()
        .and_then(|leaf| leaf.to_str())
        .map_or_else(|| trimmed.to_string(), str::to_string)
}

/// Snapshot locks graph structure for the default aDNA shotgun pipeline.
#[test]
#[cfg(feature = "bam_downstream")]
fn bam_adna_shotgun_graph_is_pure() -> anyhow::Result<()> {
    let mut tool_specs = BTreeMap::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog("bam-to-bam__adna_shotgun__v1") {
        let stage = BamStage::try_from(stage_id.as_str())?;
        let tool_id = bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage);
        tool_specs.insert(stage_id, dummy_tool(tool_id.as_str()));
    }

    let temp = bijux_dna_infra::temp_dir("bam-graph-snapshot")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"")?;
    std::fs::write(&reference, b">chrM\nACGT\n")?;

    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs,
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: Some(reference),
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };

    let graph = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    let json = serde_json::to_value(&graph)?;
    let json = normalize_graph_snapshot_paths(json, temp.path());
    let json = bijux_dna_core::contract::canonical::canonicalize_truth_json(&json);
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        let name = snapshot_name("contracts", "bam_adna_shotgun_graph");
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

#[test]
#[cfg(feature = "bam_downstream")]
fn graph_snapshot_path_normalization_covers_command_templates() {
    let transient_root = Path::new("/archive/build/bam-graph-snapshot");
    let input = serde_json::json!({
        "command": {
            "template": [
                "mapdamage2",
                "/archive/build/bam-graph-snapshot/sample.bam",
                "run --input /archive/build/bam-graph-snapshot/sample.bam",
            ],
        },
        "path": "/archive/build/bam-graph-snapshot/sample.bam",
    });

    assert_eq!(
        normalize_graph_snapshot_paths(input, transient_root),
        serde_json::json!({
            "command": {
                "template": [
                    "mapdamage2",
                    "sample.bam",
                    "run --input <TMPDIR>/<TMP>/sample.bam",
                ],
            },
            "path": "sample.bam",
        })
    );
}
