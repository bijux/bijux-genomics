#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

#[test]
fn policy__contracts__opentelemetry_version_policy__workspace_uses_single_otel_version() {
    let root = support::workspace_root();
    let lock = root.join("Cargo.lock");
    let raw = std::fs::read_to_string(&lock).unwrap_or_else(|_| panic!("read {}", lock.display()));

    let mut current_name: Option<String> = None;
    let mut otel_versions = BTreeSet::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("name = \"") {
            current_name = Some(name.trim_end_matches('"').to_string());
            continue;
        }
        if let Some(version) = trimmed.strip_prefix("version = \"") {
            if let Some(name) = current_name.as_deref() {
                if name == "opentelemetry" || name == "opentelemetry_sdk" {
                    otel_versions.insert(version.trim_end_matches('"').to_string());
                }
            }
        }
    }

    let otel_minor_series: BTreeSet<String> = otel_versions
        .iter()
        .filter_map(|version| {
            let mut parts = version.split('.');
            let major = parts.next()?;
            let minor = parts.next()?;
            Some(format!("{major}.{minor}"))
        })
        .collect();

    assert!(
        otel_minor_series.len() <= 1,
        "OpenTelemetry crates must stay on one major.minor series; found versions {otel_versions:?}"
    );
}
