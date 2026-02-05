use std::path::Path;

use bijux_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}

#[test]
fn api_has_no_planning_policy_keywords() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let denylist = [
        "smart_pipeline",
        "normalize_",
        "tool_list",
        "stage ordering",
        "bijux_stages_",
        "bijux_domain_",
        "bijux_exec",
    ];
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for needle in &denylist {
            if content.contains(needle) {
                offenders.push(format!("{}::{needle}", entry.path().display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "API must not embed planning policy: {offenders:?}"
    );
}
