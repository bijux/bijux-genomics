use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;
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

#[test]
fn prelude_exports_only() {
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
        assert!(
            !(has_fn || has_impl),
            "prelude must be exports-only; found impl/fn in {}",
            file.display()
        );
    }
}

#[test]
fn error_category_is_core_only() {
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
        if content.contains("enum ErrorCategory") {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "ErrorCategory must be defined only in bijux-core: {:?}",
        offenders
    );
}

#[test]
fn engine_does_not_depend_on_runner_or_environment() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");

    let engine = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-engine")
        .expect("bijux-engine package");
    let deps: BTreeSet<_> = engine
        .dependencies
        .iter()
        .map(|dep| dep.name.as_str())
        .collect();
    assert!(
        !deps.contains("bijux-runner"),
        "bijux-engine must not depend on bijux-runner"
    );
    assert!(
        !deps.contains("bijux-environment"),
        "bijux-engine must not depend on bijux-environment"
    );
}

#[test]
fn core_does_not_depend_on_runtime() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");

    let core = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-core")
        .expect("bijux-core package");
    let deps: BTreeSet<_> = core
        .dependencies
        .iter()
        .map(|dep| dep.name.as_str())
        .collect();
    assert!(
        !deps.contains("bijux-runtime"),
        "bijux-core must not depend on bijux-runtime"
    );
}

#[test]
fn domains_do_not_depend_on_stages_or_runner() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");
    let domains = ["bijux-domain-fastq", "bijux-domain-bam"];
    for domain in domains {
        let pkg = metadata
            .packages
            .iter()
            .find(|pkg| pkg.name == domain)
            .unwrap_or_else(|| panic!("{domain} package"));
        let deps: BTreeSet<_> = pkg
            .dependencies
            .iter()
            .map(|dep| dep.name.as_str())
            .collect();
        let forbidden = ["bijux-stages-fastq", "bijux-stages-bam", "bijux-runner"];
        for banned in forbidden {
            assert!(
                !deps.contains(banned),
                "{domain} must not depend on {banned}"
            );
        }
    }
}

#[test]
fn public_modules_live_in_lib_rs() {
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
                panic!(
                    "public modules must be declared in lib.rs for {}: {}",
                    krate,
                    file.display()
                );
            }
        }
    }
}

#[test]
fn litmus_doc_exists_and_lists_rules() {
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
        assert!(
            content.contains(rule),
            "ARCHITECTURE_LITMUS.md missing rule: {rule}"
        );
    }
}

#[test]
fn planners_do_not_embed_defaults_ledgers() {
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
    assert!(
        offenders.is_empty(),
        "planners must reference defaults through bijux-pipelines only; found direct ledger use: {:?}",
        offenders
    );
}
