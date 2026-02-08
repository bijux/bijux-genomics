#![allow(non_snake_case)]
#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use toml::Value as TomlValue;
use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn rs_files_under(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(|entry| entry.into_path())
        .collect()
}

fn crate_dependencies(root: &Path, crate_name: &str) -> BTreeSet<String> {
    let manifest = root
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|_| panic!("read manifest {}", manifest.display()));
    let parsed: TomlValue = content
        .parse()
        .unwrap_or_else(|_| panic!("parse manifest {}", manifest.display()));
    let mut deps = BTreeSet::new();

    let mut collect_from = |table: Option<&TomlValue>| {
        if let Some(TomlValue::Table(entries)) = table {
            for (name, _) in entries.iter() {
                deps.insert(name.to_string());
            }
        }
    };

    collect_from(parsed.get("dependencies"));
    collect_from(parsed.get("dev-dependencies"));
    collect_from(parsed.get("build-dependencies"));

    if let Some(TomlValue::Table(targets)) = parsed.get("target") {
        for target in targets.values() {
            if let TomlValue::Table(cfg_table) = target {
                collect_from(cfg_table.get("dependencies"));
                collect_from(cfg_table.get("dev-dependencies"));
                collect_from(cfg_table.get("build-dependencies"));
            }
        }
    }

    deps
}

fn policy_test_prefix(path: &Path, root: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let mut parts = rel.iter().filter_map(|p| p.to_str()).collect::<Vec<_>>();
    if parts.len() >= 2 && parts[0] == "tests" {
        parts.remove(0);
    }
    let suite = if parts.len() > 1 { parts[0] } else { "root" };
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    format!("policy__{suite}__{stem}__")
}

#[test]
fn policy__contracts__policies__prelude_exports_only() {
    let root = workspace_root();
    let prelude_dir = root
        .join("crates")
        .join("bijux-core")
        .join("src")
        .join("prelude");
    for file in rs_files_under(&prelude_dir) {
        let content = std::fs::read_to_string(&file).expect("read prelude file");
        let has_fn = content.lines().any(|line| line.contains("fn "));
        let has_impl = content.lines().any(|line| line.contains("impl "));
        bijux_policies::policy_assert!(
            !(has_fn || has_impl),
            "prelude must be exports-only; found impl/fn in {}",
            file.display()
        );
    }
}

#[test]
fn policy__contracts__policies__error_category_is_core_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if entry
            .path()
            .ends_with("crates/bijux-policies/tests/policies.rs")
        {
            continue;
        }
        if entry
            .path()
            .ends_with("bijux-core/src/foundation/errors.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if content.contains(concat!("enum ", "ErrorCategory")) {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "ErrorCategory must be defined only in bijux-core: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__policies__engine_does_not_depend_on_runner_or_environment() {
    let root = workspace_root();
    let deps = crate_dependencies(&root, "bijux-engine");
    bijux_policies::policy_assert!(
        !deps.contains("bijux-runner"),
        "bijux-engine must not depend on bijux-runner"
    );
    bijux_policies::policy_assert!(
        !deps.contains("bijux-environment"),
        "bijux-engine must not depend on bijux-environment"
    );
}

#[test]
fn policy__contracts__policies__core_does_not_depend_on_runtime() {
    let root = workspace_root();
    let deps = crate_dependencies(&root, "bijux-core");
    bijux_policies::policy_assert!(
        !deps.contains("bijux-runtime"),
        "bijux-core must not depend on bijux-runtime"
    );
}

#[test]
fn policy__contracts__policies__domains_do_not_depend_on_stages_or_runner() {
    let root = workspace_root();
    let domains = ["bijux-domain-fastq", "bijux-domain-bam"];
    for domain in domains {
        let deps = crate_dependencies(&root, domain);
        let forbidden = ["bijux-stages-fastq", "bijux-stages-bam", "bijux-runner"];
        for banned in forbidden {
            bijux_policies::policy_assert!(
                !deps.contains(banned),
                "{domain} must not depend on {banned}"
            );
        }
    }
}

#[test]
fn policy__contracts__policies__public_modules_live_in_lib_rs() {
    let root = workspace_root();
    let crates = ["bijux-core", "bijux-engine", "bijux-runtime"];
    for krate in crates {
        let src = root.join("crates").join(krate).join("src");
        for file in rs_files_under(&src) {
            if file.ends_with("lib.rs") {
                continue;
            }
            if file.ends_with("mod.rs") {
                continue;
            }
            let content = std::fs::read_to_string(&file).expect("read source");
            if content
                .lines()
                .any(|line| line.trim_start().starts_with("pub mod "))
            {
                bijux_policies::policy_panic!(
                    "public modules must be declared in lib.rs for {}: {}",
                    krate,
                    file.display()
                );
            }
        }
    }
}

#[test]
fn policy__contracts__policies__policy_test_names_are_consistent() {
    let root = workspace_root();
    let tests_root = root.join("crates").join("bijux-policies").join("tests");
    let mut offenders = Vec::new();
    for file in rs_files_under(&tests_root) {
        if file.components().any(|c| c.as_os_str() == "support") {
            continue;
        }
        let content = std::fs::read_to_string(&file).expect("read test file");
        let expected_prefix = policy_test_prefix(&file, &tests_root);
        let mut awaiting_fn = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "#[test]" {
                awaiting_fn = true;
                continue;
            }
            if awaiting_fn {
                if let Some(rest) = trimmed.strip_prefix("fn ") {
                    if let Some(name) = rest.split(['(', ' ']).next() {
                        if !name.starts_with(&expected_prefix) {
                            offenders.push(format!(
                                "{}: expected prefix {} but found {}",
                                file.display(),
                                expected_prefix,
                                name
                            ));
                        }
                    }
                    awaiting_fn = false;
                }
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "policy test names must follow policy__<suite>__<file>__<rule>: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__policies__litmus_doc_exists_and_lists_rules() {
    let root = workspace_root();
    let path = root.join("docs/ARCHITECTURE_LITMUS.md");
    let content = std::fs::read_to_string(&path).expect("read ARCHITECTURE_LITMUS.md");
    let required = [
        "engine does not depend on runner or environment",
        "prelude is exports-only",
        "defaults live only in bijux-pipelines",
        "composition roots are only in API/CLI",
    ];
    for rule in required {
        bijux_policies::policy_assert!(
            content.contains(rule),
            "ARCHITECTURE_LITMUS.md missing rule: {rule}"
        );
    }
}

#[test]
fn policy__contracts__policies__planners_do_not_embed_defaults_ledgers() {
    let root = workspace_root();
    let planner_dirs = [
        root.join("crates").join("bijux-planner-fastq"),
        root.join("crates").join("bijux-planner-bam"),
    ];
    let mut offenders = Vec::new();
    for dir in planner_dirs {
        for file in rs_files_under(&dir.join("src")) {
            let content = std::fs::read_to_string(&file).expect("read planner file");
            if content.contains("DefaultsLedger") || content.contains("defaults_ledger") {
                offenders.push(file.display().to_string());
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "planners must reference defaults through bijux-pipelines only; found direct ledger use: {:?}",
        offenders
    );
}
