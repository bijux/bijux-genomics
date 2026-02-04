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
        if content.contains("std::fs::write(")
            || content.contains("fs::write(")
            || content.contains("std::fs::rename(")
            || content.contains("fs::rename(")
            || content.contains("std::fs::remove_file(")
            || content.contains("fs::remove_file(")
            || content.contains("std::fs::create_dir_all(")
            || content.contains("fs::create_dir_all(")
        {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "direct std::fs writes/renames/removals/dir-creation are forbidden in engine services: {offenders:?}"
    );
}
