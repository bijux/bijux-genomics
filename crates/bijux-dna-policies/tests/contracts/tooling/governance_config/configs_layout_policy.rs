#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

const REFERENCE_ROOT_OWNER: &str = "crates/bijux-dna-db-ref/src/runtime_config/paths.rs";
const REFERENCE_ROOT_MARKERS: &str = concat!(
    "root.join(\"Cargo.toml\").is_file()",
    " && root.join(\"Cargo.lock\").is_file()",
    " && root.join(\"crates/bijux-dna/Cargo.toml\").is_file()",
    " && root.join(\"configs\").is_dir()"
);

fn normalize_source_spacing(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_config_construction(source: &str) -> bool {
    source.contains("join(\"configs\")")
        || source.contains("Path::new(\"configs\")")
        || source.contains("cwd.join(\"configs/")
}

fn uses_configs_without_helper(relative_path: &str, source: &str) -> bool {
    if relative_path != REFERENCE_ROOT_OWNER {
        return contains_config_construction(source);
    }

    // The metadata resolver may observe the repository directory marker without depending on
    // infrastructure. Admit its complete pure predicate, never config acquisition or an owner-wide
    // exemption. Lexical scopes exclude matching text inside comments and literals.
    let marker =
        bijux_dna_testkit::workspace_support::rust_block_ranges(source).into_iter().find(|scope| {
            let signature = source[..scope.start].lines().last().unwrap_or_default();
            let body =
                source[scope.clone()].strip_prefix('{').and_then(|body| body.strip_suffix('}'));
            normalize_source_spacing(signature) == "fn is_repository_root(root: &Path) -> bool"
                && body.is_some_and(|body| normalize_source_spacing(body) == REFERENCE_ROOT_MARKERS)
        });

    if let Some(scope) = marker {
        contains_config_construction(&source[..scope.start])
            || contains_config_construction(&source[scope.end..])
    } else {
        contains_config_construction(source)
    }
}

#[test]
fn policy__contracts__configs_layout_policy__configs_root_contains_only_index_and_directories() {
    let root = workspace_root();
    let configs = root.join("configs");
    let mut offenders = Vec::new();

    for entry in std::fs::read_dir(&configs).expect("read configs/") {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(&path).display().to_string();
        if rel != "configs/index.md" && rel != "configs/OWNERS.toml" {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "configs root must not contain files other than configs/index.md:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__configs_layout_policy__rust_src_uses_configs_path_helper() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path).display().to_string();
        if rel == "crates/bijux-dna-infra/src/paths/config.rs" {
            continue;
        }
        if !rel.contains("/src/") {
            continue;
        }

        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if uses_configs_without_helper(&rel, &raw) {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Rust src must use bijux_dna_infra::configs_file/configs_dir helpers:\n{}",
        offenders.join("\n")
    );
}

fn reference_marker_source(predicate: &str) -> String {
    format!(
        "fn is_repository_root(root: &Path) -> bool {{\n\
         root.join(\"Cargo.toml\").is_file()\n\
         && root.join(\"Cargo.lock\").is_file()\n\
         && root.join(\"crates/bijux-dna/Cargo.toml\").is_file()\n\
         && {predicate}\n}}"
    )
}

#[test]
fn policy__contracts__configs_layout_policy__reference_marker_admits_only_complete_pure_owner() {
    let marker = reference_marker_source("root.join(\"configs\").is_dir()");
    assert!(!uses_configs_without_helper(REFERENCE_ROOT_OWNER, &marker));

    let root = workspace_root();
    let actual = std::fs::read_to_string(root.join(REFERENCE_ROOT_OWNER))
        .expect("read reference root owner");
    assert!(!uses_configs_without_helper(REFERENCE_ROOT_OWNER, &actual));

    for owner in [
        "crates/bijux-dna-db-ref/src/runtime_config/catalog.rs",
        "crates/bijux-dna-db-ref/src/runtime_config/paths.rs.extra",
        "crates/bijux-dna-api/src/runtime_config/paths.rs",
    ] {
        assert!(uses_configs_without_helper(owner, &marker), "foreign marker owner: {owner}");
    }

    for predicate in [
        "std::fs::read_dir(root.join(\"configs\")).is_ok()",
        "root.join(\"configs\").is_dir() && permits_config_reads(root)",
        "root.join(\"configs\").is_dir()\n&& permits_config_reads(root)",
        "root.join(\"configs\").is_dir() || root.exists()",
        "root.join(\"configs\").is_dir().then(|| acquire_config(root)).is_some()",
        "root.join(\"configs\").is_dir(",
        "ro ot.join(\"configs\").is_dir()",
        "root.join(\"configs\").is_ dir()",
        "root.join(\"configs\").is_dir(); true",
    ] {
        let source = reference_marker_source(predicate);
        assert!(
            uses_configs_without_helper(REFERENCE_ROOT_OWNER, &source),
            "extended or consuming predicate: {predicate}"
        );
    }

    let unclosed = marker.strip_suffix('}').expect("closed marker fixture");
    assert!(uses_configs_without_helper(REFERENCE_ROOT_OWNER, unclosed));
    let consuming_signature = marker.replace("-> bool", "-> ConfigCatalog");
    assert!(uses_configs_without_helper(REFERENCE_ROOT_OWNER, &consuming_signature));
}

#[test]
fn policy__contracts__configs_layout_policy__reference_marker_keeps_other_config_reads_visible() {
    let marker = reference_marker_source("root.join(\"configs\").is_dir()");
    for consumer in [
        "fn load(root: &Path) { std::fs::read(root.join(\"configs\").join(\"species.toml\")); }",
        "fn load() { std::fs::read(Path::new(\"configs\").join(\"species.toml\")); }",
        "fn load(cwd: &Path) { std::fs::read(cwd.join(\"configs/runtime/species.toml\")); }",
    ] {
        for source in [format!("{marker}\n{consumer}"), format!("{consumer}\n{marker}")] {
            assert!(
                uses_configs_without_helper(REFERENCE_ROOT_OWNER, &source),
                "config consumer hidden by marker: {consumer}"
            );
        }
    }

    for source in
        [format!("/*\n{marker}\n*/"), format!("const EXAMPLE: &str = r#\"\n{marker}\n\"#;")]
    {
        assert!(
            uses_configs_without_helper(REFERENCE_ROOT_OWNER, &source),
            "non-code marker must not admit a config construction"
        );
    }
}
