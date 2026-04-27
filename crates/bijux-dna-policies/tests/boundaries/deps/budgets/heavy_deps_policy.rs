#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn crate_dirs() -> Vec<PathBuf> {
    let root = workspace_root().join("crates");
    let mut crates = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return crates;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("Cargo.toml").exists() {
            crates.push(path);
        }
    }
    crates.sort();
    crates
}

fn has_feature_line(content: &str, feature: &str, dep: &str) -> bool {
    content.lines().any(|line| line.contains(feature) && line.contains(dep))
}

fn crate_name(manifest: &Path) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            let name = line.split_once('=').map(|(_, v)| v.trim())?;
            let name = name.trim_matches('"').to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
        if line.starts_with('[') {
            break;
        }
    }
    None
}

#[test]
fn policy__boundaries__heavy_deps_policy__heavy_dependencies_are_feature_gated() {
    let heavy = ["tracing-subscriber", "rusqlite", "sysinfo", "opentelemetry"];
    let mut offenders = Vec::new();
    for crate_dir in crate_dirs() {
        let manifest = crate_dir.join("Cargo.toml");
        let name = crate_name(&manifest).unwrap_or_default();
        if name == "bijux-dna" {
            continue;
        }
        let content = std::fs::read_to_string(&manifest).expect("read Cargo.toml");
        for dep in heavy {
            if content.contains(dep) {
                let is_optional = content
                    .lines()
                    .any(|line| line.contains(dep) && line.contains("optional = true"));
                let gated = content.contains("[features]")
                    && (has_feature_line(&content, "otel", dep)
                        || has_feature_line(&content, "sqlite", dep)
                        || has_feature_line(&content, "tracing", dep)
                        || has_feature_line(&content, "sysinfo", dep)
                        || has_feature_line(&content, "telemetry", dep)
                        || has_feature_line(&content, "bench", dep)
                        || has_feature_line(&content, "report-html", dep));
                if !is_optional && !gated {
                    offenders.push(format!(
                        "{} depends on {} without feature gating",
                        manifest.display(),
                        dep
                    ));
                }
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "heavy dependencies must be feature-gated:\n{}",
        offenders.join("\n")
    );
}
