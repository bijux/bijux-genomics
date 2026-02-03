use std::path::Path;

use bijux_guardrails::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}

#[test]
fn no_ad_hoc_fs_writes_in_services() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let services = crate_root.join("src").join("services");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&services)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("std::fs::write(") {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "direct std::fs::write is forbidden in engine services: {offenders:?}"
    );
}
