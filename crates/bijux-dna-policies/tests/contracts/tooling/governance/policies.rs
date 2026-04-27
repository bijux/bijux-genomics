#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use toml::Value as TomlValue;
use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn rs_files_under(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(walkdir::DirEntry::into_path)
        .collect()
}

fn crate_dependencies(root: &Path, crate_name: &str) -> BTreeSet<String> {
    let manifest = root.join("crates").join(crate_name).join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|_| panic!("read manifest {}", manifest.display()));
    let parsed: TomlValue =
        content.parse().unwrap_or_else(|_| panic!("parse manifest {}", manifest.display()));
    let mut deps = BTreeSet::new();

    let mut collect_from = |table: Option<&TomlValue>| {
        if let Some(TomlValue::Table(entries)) = table {
            for (name, _) in entries {
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
    if parts.len() >= 4
        && parts[0] == "boundaries"
        && parts[1] == "surface"
        && parts[2] == "workspace_rules"
    {
        return "policy__boundaries__workspace__".to_string();
    }
    let suite = if parts.len() > 1 { parts[0] } else { "root" };
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    format!("policy__{suite}__{stem}__")
}

fn configured_domains(root: &Path) -> Vec<String> {
    let path = root.join("configs").join("ci").join("registry").join("domains.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("read domains config {}", path.display()));
    let parsed: TomlValue =
        raw.parse().unwrap_or_else(|_| panic!("parse domains config {}", path.display()));
    parsed
        .get("domains")
        .and_then(TomlValue::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("id").and_then(TomlValue::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__policies__prelude_exports_only() {
    let root = workspace_root();
    let prelude_dir = root.join("crates").join("bijux-dna-core").join("src").join("prelude");
    for file in rs_files_under(&prelude_dir) {
        let content = std::fs::read_to_string(&file).expect("read prelude file");
        let has_fn = content.lines().any(|line| line.contains("fn "));
        let has_impl = content.lines().any(|line| line.contains("impl "));
        bijux_dna_policies::policy_assert!(
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
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if entry.path().ends_with("../../../tests/policies.rs") {
            continue;
        }
        if entry.path().ends_with("bijux-dna-core/src/foundation/errors.rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if content.contains(concat!("enum ", "ErrorCategory")) {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "ErrorCategory must be defined only in bijux-dna-core: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__policies__engine_does_not_depend_on_runner_or_environment() {
    let root = workspace_root();
    let deps = crate_dependencies(&root, "bijux-dna-engine");
    bijux_dna_policies::policy_assert!(
        !deps.contains("bijux-dna-runner"),
        "bijux-dna-engine must not depend on bijux-dna-runner"
    );
    bijux_dna_policies::policy_assert!(
        !deps.contains("bijux-dna-environment"),
        "bijux-dna-engine must not depend on bijux-dna-environment"
    );
}

#[test]
fn policy__contracts__policies__core_does_not_depend_on_runtime() {
    let root = workspace_root();
    let deps = crate_dependencies(&root, "bijux-dna-core");
    bijux_dna_policies::policy_assert!(
        !deps.contains("bijux-dna-runtime"),
        "bijux-dna-core must not depend on bijux-dna-runtime"
    );
}

#[test]
fn policy__contracts__policies__domains_do_not_depend_on_stages_or_runner() {
    let root = workspace_root();
    let domains = configured_domains(&root);
    for domain_id in domains {
        let domain = format!("bijux-dna-domain-{domain_id}");
        let deps = crate_dependencies(&root, &domain);
        let forbidden = [format!("bijux-dna-stages-{domain_id}"), "bijux-dna-runner".to_string()];
        for banned in forbidden {
            bijux_dna_policies::policy_assert!(
                !deps.contains(&banned),
                "{domain} must not depend on {banned}"
            );
        }
    }
}

#[test]
fn policy__contracts__policies__public_modules_live_in_lib_rs() {
    let root = workspace_root();
    let crates = ["bijux-dna-core", "bijux-dna-engine", "bijux-dna-runtime"];
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
            if content.lines().any(|line| line.trim_start().starts_with("pub mod ")) {
                bijux_dna_policies::policy_panic!(
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
    let tests_root = root.join("crates").join("bijux-dna-policies").join("tests");
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
                        let normalized_name = name.strip_prefix("slow__").unwrap_or(name);
                        if !normalized_name.starts_with(&expected_prefix) {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "policy test names must follow policy__<suite>__<file>__<rule>: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__policies__litmus_doc_exists_and_lists_rules() {
    let root = workspace_root();
    let path = root.join("docs/10-architecture/ARCHITECTURE_LITMUS.md");
    let content = std::fs::read_to_string(&path).expect("read ARCHITECTURE_LITMUS.md");
    let required = [
        "engine does not depend on runner or environment",
        "prelude is exports-only",
        "defaults live only in bijux-dna-pipelines",
        "composition roots are only in API/CLI",
        "Domain is authored SSOT; configs are generated; code consumes generated configs; makes call CLI only.",
    ];
    for rule in required {
        bijux_dna_policies::policy_assert!(
            content.contains(rule),
            "ARCHITECTURE_LITMUS.md missing rule: {rule}"
        );
    }
    let architecture_path = root.join("docs/10-architecture/ARCHITECTURE.md");
    let architecture = std::fs::read_to_string(&architecture_path).expect("read ARCHITECTURE.md");
    bijux_dna_policies::policy_assert!(
        architecture.contains(
            "Domain is the authored SSOT; configs are generated; code consumes generated configs; makes call CLI only."
        ),
        "ARCHITECTURE.md must define the SSOT rule"
    );
    let authority_map_path = root.join("docs/10-architecture/CRATE_AUTHORITY_MAP.md");
    bijux_dna_policies::policy_assert!(
        authority_map_path.exists(),
        "CRATE_AUTHORITY_MAP.md must exist"
    );
    let authority_map =
        std::fs::read_to_string(&authority_map_path).expect("read CRATE_AUTHORITY_MAP.md");
    bijux_dna_policies::policy_assert!(
        authority_map.contains("bijux-dna-engine")
            && authority_map.contains("bijux-dna-runner")
            && authority_map.contains("bijux-dna-environment")
            && authority_map.contains("bijux-dna-planner-fastq")
            && authority_map.contains("bijux-dna-planner-bam")
            && authority_map.contains("bijux-dna-stages-fastq")
            && authority_map.contains("bijux-dna-stages-bam"),
        "CRATE_AUTHORITY_MAP.md must define planner/stage/engine/runner/environment authority boundaries"
    );
}

#[test]
fn policy__contracts__policies__planners_do_not_embed_defaults_ledgers() {
    let root = workspace_root();
    let planner_dirs = [
        root.join("crates").join("bijux-dna-planner-fastq"),
        root.join("crates").join("bijux-dna-planner-bam"),
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "planners must reference defaults through bijux-dna-pipelines only; found direct ledger use: {:?}",
        offenders
    );
}
