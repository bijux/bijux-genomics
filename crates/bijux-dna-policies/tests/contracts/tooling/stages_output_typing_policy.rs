#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__stages_output_typing_policy__generated_stages_define_output_kinds() {
    let stages_path = support::workspace_root().join("configs/ci/stages/stages.toml");
    let raw = std::fs::read_to_string(&stages_path)
        .unwrap_or_else(|_| panic!("read {}", stages_path.display()));
    let parsed: toml::Value = raw.parse().expect("parse configs/ci/stages/stages.toml");
    let entries = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut offenders = Vec::new();
    for stage in entries {
        let id = stage
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<missing-id>");
        let output_kinds = stage
            .get("output_kinds")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        if output_kinds.is_empty() {
            offenders.push(format!("stage={id}: missing output_kinds"));
        }
    }

    assert!(
        offenders.is_empty(),
        "generated stage output typing violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__stages_output_typing_policy__registry_stages_declare_semantics_and_artifacts()
{
    let root = support::workspace_root();
    let registry = bijux_dna_runtime::manifests::load_manifests(
        &root.join("configs").join("tool_registry.toml"),
    )
    .expect("load tool registry manifests");

    let mut offenders = Vec::new();
    let stable_name = regex::Regex::new(r"^[a-z0-9_]+$").expect("compile stable name regex");
    let semver_re = regex::Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+$").expect("compile semver regex");
    for (stage_id, stage) in registry.stages() {
        if stage.produced_artifacts.is_empty() {
            offenders.push(format!("stage={stage_id}: missing produced_artifacts"));
        }
        for artifact in &stage.produced_artifacts {
            if !stable_name.is_match(artifact) {
                offenders.push(format!(
                    "stage={stage_id}: produced artifact `{artifact}` must be stable snake_case"
                ));
            }
        }
        if !semver_re.is_match(&stage.stage_semver) {
            offenders.push(format!(
                "stage={stage_id}: invalid stage_semver `{}`",
                stage.stage_semver
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "stage semantic declaration violations:\n{}",
        offenders.join("\n")
    );
}
