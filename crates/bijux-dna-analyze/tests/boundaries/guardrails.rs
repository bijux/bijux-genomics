use std::fs;
use std::path::{Path, PathBuf};

#[path = "../../../bijux-dna-policies/tests/guardrails.rs"]
mod policies;

/// Centralized guardrails runner.
#[test]
fn guardrails() {
    policies::guardrails();
}

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn no_cross_layer_calls() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    for file in files {
        let path_str = file.to_string_lossy();
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        if path_str.contains("/pipeline/") {
            continue;
        }

        let depends_on = |needle: &str| contents.contains(needle);
        let allow_aggregate_for_load = path_str.ends_with("/load/sqlite/queries.rs")
            || path_str.contains("/load/sqlite/queries/")
            || path_str.contains("/load/sqlite/queries_")
            || path_str.ends_with("/load/sqlite/rows.rs");

        if path_str.contains("/decision/") {
            assert!(!depends_on("crate::load"), "decision must not depend on load: {path_str}");
            assert!(!depends_on("crate::report"), "decision must not depend on report: {path_str}");
            assert!(
                !depends_on("crate::pipeline"),
                "decision must not depend on pipeline: {path_str}"
            );
        }

        if path_str.contains("/report/") {
            assert!(!depends_on("crate::load"), "report must not depend on load: {path_str}");
            assert!(
                !depends_on("crate::pipeline"),
                "report must not depend on pipeline: {path_str}"
            );
        }

        if path_str.contains("/load/") {
            assert!(!depends_on("crate::decision"), "load must not depend on decision: {path_str}");
            assert!(!depends_on("crate::report"), "load must not depend on report: {path_str}");
            assert!(!depends_on("crate::pipeline"), "load must not depend on pipeline: {path_str}");
            if !allow_aggregate_for_load {
                assert!(
                    !depends_on("crate::aggregate"),
                    "load must not depend on aggregate: {path_str}"
                );
            }
        }

        if !path_str.contains("/pipeline/") {
            let uses_load = depends_on("crate::load");
            let uses_decision = depends_on("crate::decision");
            let uses_report = depends_on("crate::report");
            assert!(
                !(uses_load && (uses_decision || uses_report)),
                "only pipeline may import load + decision/report: {path_str}"
            );
            assert!(
                !(uses_decision && uses_report),
                "only pipeline may import decision + report: {path_str}"
            );
        }
    }
}

#[test]
fn public_api_is_small() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    let mut offenders = Vec::new();
    let mut skip_next_pub = false;
    let mut in_pub_block = 0usize;
    let allowed = [
        "pub mod aggregate;",
        "pub mod decision;",
        "pub mod exports;",
        "pub mod failure;",
        "pub mod load;",
        "pub mod model;",
        "pub mod public_api;",
        "pub mod report;",
        "pub use public_api::*;",
        "pub fn analyze_run(input: &AnalyzeInput) -> anyhow::Result<AnalyzeOutput> {",
    ];
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            skip_next_pub = true;
            continue;
        }
        if in_pub_block > 0 {
            if trimmed.contains('{') {
                in_pub_block += trimmed.matches('{').count();
            }
            if trimmed.contains('}') {
                let closes = trimmed.matches('}').count();
                in_pub_block = in_pub_block.saturating_sub(closes);
            }
            continue;
        }
        if skip_next_pub {
            if trimmed.starts_with("pub ") {
                skip_next_pub = false;
                continue;
            }
            if !trimmed.is_empty() {
                skip_next_pub = false;
            }
        }
        if trimmed.starts_with("pub ") {
            if (trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum "))
                && trimmed.contains('{')
            {
                in_pub_block = trimmed.matches('{').count();
            }
            if !allowed.iter().any(|allowed| trimmed.starts_with(allowed)) {
                offenders.push(trimmed.to_string());
            }
        }
    }
    assert!(offenders.is_empty(), "unexpected public items in lib.rs: {offenders:?}");
    Ok(())
}

#[test]
fn no_new_top_level_modules_without_owner() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut modules = Vec::new();
    let Ok(entries) = fs::read_dir(&src_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            modules.push(path);
        }
    }
    let owners_dir = manifest_dir.join("docs").join("owners");
    let mut offenders = Vec::new();
    for module in modules {
        let owner = module.join("OWNER.toml");
        let name = module.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        let docs_owner = owners_dir.join(format!("{name}.md"));
        if !owner.exists() && !docs_owner.exists() {
            offenders.push(module.display().to_string());
        }
    }
    assert!(offenders.is_empty(), "top-level modules require OWNER.toml: {offenders:?}");
}

#[test]
fn owner_files_document_responsibility_and_boundaries() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let required = ["owner", "responsibility", "may_depend_on", "must_not_depend_on"];
    let mut offenders = Vec::new();

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let owner_path = path.join("OWNER.toml");
        let raw = fs::read_to_string(&owner_path)?;
        let value: toml::Value = toml::from_str(&raw)?;
        let Some(table) = value.as_table() else {
            offenders.push(owner_path.display().to_string());
            continue;
        };

        for key in required {
            let valid = table.get(key).is_some_and(|value| match value {
                toml::Value::String(text) => !text.trim().is_empty(),
                toml::Value::Array(items) => !items.is_empty(),
                _ => false,
            });
            if !valid {
                offenders.push(format!("{} missing {key}", owner_path.display()));
            }
        }
    }

    assert!(offenders.is_empty(), "OWNER.toml files need durable metadata: {offenders:?}");
    Ok(())
}

#[test]
fn cargo_manifest_keeps_dependency_boundaries() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let raw = fs::read_to_string(cargo_toml)?;
    let value: toml::Value = toml::from_str(&raw)?;
    let dependencies = value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| anyhow::anyhow!("missing dependencies table"))?;
    let dev_dependencies = value
        .get("dev-dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| anyhow::anyhow!("missing dev-dependencies table"))?;

    for forbidden in ["bijux-dna-runner", "bijux-dna-engine", "bijux-dna-bench"] {
        assert!(
            !dependencies.contains_key(forbidden),
            "{forbidden} must not be a normal dependency of bijux-dna-analyze"
        );
    }

    let duplicates: Vec<_> =
        dependencies.keys().filter(|name| dev_dependencies.contains_key(*name)).collect();
    assert!(
        duplicates.is_empty(),
        "normal dependencies must not be duplicated as dev-dependencies: {duplicates:?}"
    );

    Ok(())
}
