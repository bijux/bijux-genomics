/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_pipelines::{merge_effective_defaults, EffectiveDefaults};
use bijux_dna_testkit::snapshot_name;

#[test]
fn override_precedence_is_stable() {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    let _guard = settings.bind_to_scope();

    let mut base_tools = BTreeMap::new();
    base_tools.insert(
        StageId::from_static("fastq.trim"),
        ToolId::from_static("fastp"),
    );

    let mut profile_tools = BTreeMap::new();
    profile_tools.insert(
        StageId::from_static("fastq.trim"),
        ToolId::from_static("cutadapt"),
    );

    let mut cli_tools = BTreeMap::new();
    cli_tools.insert(
        StageId::from_static("fastq.trim"),
        ToolId::from_static("bbduk"),
    );

    let mut api_tools = BTreeMap::new();
    api_tools.insert(
        StageId::from_static("fastq.trim"),
        ToolId::from_static("trimmomatic"),
    );

    let base = EffectiveDefaults {
        tools: base_tools,
        params: BTreeMap::new(),
        rationales: BTreeMap::new(),
    };
    let profile = EffectiveDefaults {
        tools: profile_tools,
        params: BTreeMap::new(),
        rationales: BTreeMap::new(),
    };
    let cli = EffectiveDefaults {
        tools: cli_tools,
        params: BTreeMap::new(),
        rationales: BTreeMap::new(),
    };
    let api = EffectiveDefaults {
        tools: api_tools,
        params: BTreeMap::new(),
        rationales: BTreeMap::new(),
    };

    let snapshot = merge_effective_defaults(&base, Some(&profile), Some(&cli), Some(&api))
        .expect("merge defaults");
    let name = snapshot_name("contracts", "override_precedence");
    let json = serde_json::to_value(&snapshot).expect("serialize defaults");
    insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}
