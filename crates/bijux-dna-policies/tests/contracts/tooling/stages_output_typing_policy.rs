#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__stages_output_typing_policy__generated_stages_define_output_kinds() {
    let stages_path = support::workspace_root().join("configs/stages.toml");
    let raw = std::fs::read_to_string(&stages_path)
        .unwrap_or_else(|_| panic!("read {}", stages_path.display()));
    let parsed: toml::Value = raw.parse().expect("parse configs/stages.toml");
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
