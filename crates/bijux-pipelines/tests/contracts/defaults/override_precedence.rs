/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_pipelines::{merge_effective_defaults, EffectiveDefaults};
use bijux_testkit::snapshot_name;

#[test]
fn override_precedence_is_stable() {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    let _guard = settings.bind_to_scope();

    let mut base_tools = BTreeMap::new();
    base_tools.insert("fastq.trim".to_string(), "fastp".to_string());

    let mut profile_tools = BTreeMap::new();
    profile_tools.insert("fastq.trim".to_string(), "cutadapt".to_string());

    let mut cli_tools = BTreeMap::new();
    cli_tools.insert("fastq.trim".to_string(), "bbduk".to_string());

    let mut api_tools = BTreeMap::new();
    api_tools.insert("fastq.trim".to_string(), "trimmomatic".to_string());

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
    insta::assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}
