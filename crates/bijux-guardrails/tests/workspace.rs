use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bijux_guardrails::GuardrailConfig;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn crate_dirs() -> Vec<PathBuf> {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(&crates_dir).expect("read crates dir") {
        let entry = entry.expect("crate entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join("Cargo.toml").exists() {
            continue;
        }
        dirs.push(path);
    }
    dirs
}

fn read_package_name(manifest: &Path) -> String {
    let content = std::fs::read_to_string(manifest).expect("read Cargo.toml");
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            let name = line
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"'))
                .unwrap_or("");
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    panic!("missing package name in {}", manifest.display());
}

fn is_bin_crate(crate_dir: &Path) -> bool {
    let src = crate_dir.join("src");
    src.join("main.rs").exists() && !src.join("lib.rs").exists()
}

fn collect_workspace_crates() -> BTreeMap<String, PathBuf> {
    let mut crates = BTreeMap::new();
    for dir in crate_dirs() {
        let manifest = dir.join("Cargo.toml");
        let name = read_package_name(&manifest);
        crates.insert(name, dir);
    }
    crates
}

fn parse_dependencies(manifest: &Path, known: &BTreeSet<String>) -> BTreeSet<String> {
    let content = std::fs::read_to_string(manifest).expect("read Cargo.toml");
    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = matches!(
                line,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _rest)) = line.split_once('=') {
            let name = name.trim().trim_matches('"');
            if !name.is_empty() && known.contains(name) {
                deps.insert(name.to_string());
            }
        }
    }
    deps
}

#[test]
fn workspace_has_guardrails_tests() {
    for path in crate_dirs() {
        let guardrails = path.join("tests").join("guardrails.rs");
        assert!(
            guardrails.exists(),
            "missing tests/guardrails.rs in {}",
            path.display()
        );
        let content = std::fs::read_to_string(&guardrails).expect("read guardrails test");
        assert!(
            content.contains("GuardrailConfig::for_crate"),
            "guardrails test must use GuardrailConfig::for_crate in {}",
            guardrails.display()
        );
    }
}

#[test]
fn workspace_guardrail_defaults_not_increased() {
    let defaults = GuardrailConfig::default();
    for path in crate_dirs() {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let config = GuardrailConfig::for_crate(name);
        let bad = config.max_loc > defaults.max_loc
            || config.max_depth > defaults.max_depth
            || config.max_modules_per_dir > defaults.max_modules_per_dir
            || config.max_rs_files_per_dir > defaults.max_rs_files_per_dir
            || config.max_pub_items_per_file > defaults.max_pub_items_per_file
            || config.max_pub_use_per_file > defaults.max_pub_use_per_file;
        assert!(
            !bad,
            "guardrails defaults increased for {}: {:?}",
            name, config
        );
    }
}

#[test]
fn workspace_constitution_contract() {
    let crates = collect_workspace_crates();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for name in crates.keys() {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    let required = [
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-pipelines",
        "bijux-api",
        "bijux-infra",
        "bijux-core",
        "bijux-engine",
        "bijux-analyze",
        "bijux-bench",
    ];
    for name in required {
        assert!(crates.contains_key(name), "missing required crate: {name}");
        assert_eq!(
            counts.get(name).copied().unwrap_or(0),
            1,
            "duplicate crate: {name}"
        );
    }
    let env_crates: Vec<_> = crates
        .keys()
        .filter(|name| name.starts_with("bijux-env-"))
        .collect();
    assert!(!env_crates.is_empty(), "missing bijux-env-* crates");
    assert!(
        !crates.contains_key("bijux-pipelines-bam"),
        "bijux-pipelines-bam is forbidden"
    );
    assert!(
        !crates.contains_key("bijux-testkit"),
        "shared testkit crate is not allowed"
    );
}

#[test]
fn workspace_bans_pipelines_bam_crate_name() {
    let crates = collect_workspace_crates();
    for name in crates.keys() {
        assert!(
            !name.contains("pipelines-bam"),
            "crate name contains forbidden substring: {name}"
        );
    }
}

#[test]
fn workspace_crate_layout_contract() {
    for crate_dir in crate_dirs() {
        let manifest = crate_dir.join("Cargo.toml");
        assert!(
            manifest.exists(),
            "missing Cargo.toml in {}",
            crate_dir.display()
        );
        let src_dir = crate_dir.join("src");
        assert!(src_dir.exists(), "missing src/ in {}", crate_dir.display());
        if is_bin_crate(&crate_dir) {
            continue;
        }
        let makefile = crate_dir.join("Makefile.toml");
        assert!(
            makefile.exists(),
            "missing Makefile.toml in {}",
            crate_dir.display()
        );
        let tests_dir = crate_dir.join("tests");
        assert!(
            tests_dir.exists(),
            "missing tests/ in {}",
            crate_dir.display()
        );
    }
}

#[test]
fn workspace_domain_layout_contract() {
    let crates = collect_workspace_crates();
    for name in ["bijux-domain-fastq", "bijux-domain-bam"] {
        let Some(path) = crates.get(name) else {
            panic!("missing crate {name}");
        };
        for dir in ["metrics", "params", "invariants", "stage_registry", "types"] {
            let path = path.join("src").join(dir);
            assert!(path.exists(), "{name} missing src/{dir}");
        }
        let lib = path.join("src").join("lib.rs");
        assert!(lib.exists(), "{name} missing src/lib.rs");
    }
}

#[test]
fn workspace_stages_layout_contract() {
    let crates = collect_workspace_crates();
    for name in ["bijux-stages-fastq", "bijux-stages-bam"] {
        let Some(path) = crates.get(name) else {
            panic!("missing crate {name}");
        };
        let src = path.join("src");
        assert!(src.join("plan.rs").exists(), "{name} missing src/plan.rs");
        assert!(src.join("tools").is_dir(), "{name} missing src/tools/");
        let has_registry = std::fs::read_dir(&src)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| name.ends_with("_tools_registry.rs"))
                    .unwrap_or(false)
            });
        assert!(has_registry, "{name} missing *_tools_registry.rs");
    }
}

#[test]
fn workspace_no_orphan_crates() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let mut dependents: BTreeMap<String, usize> =
        crates.keys().map(|name| (name.clone(), 0)).collect();
    for (name, path) in &crates {
        let deps = parse_dependencies(&path.join("Cargo.toml"), &known);
        for dep in deps {
            if let Some(count) = dependents.get_mut(&dep) {
                *count += 1;
            }
        }
        // Ensure we don't accidentally count self.
        if let Some(count) = dependents.get_mut(name) {
            if *count > 0 {
                *count -= 0;
            }
        }
    }
    let allowlist: BTreeSet<&str> = BTreeSet::from([
        "bijux",
        "bijux-cli",
        "bijux-bench",
        "bijux-env-builder",
        "bijux-env-runtime",
        "bijux-domain-vcf",
    ]);
    for (name, count) in dependents {
        let crate_dir = crates.get(&name).expect("crate dir");
        if count == 0 && !allowlist.contains(name.as_str()) && !is_bin_crate(crate_dir) {
            panic!("orphan crate without allowlist: {name}");
        }
    }
}

#[test]
fn workspace_dependency_graph_contract() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let deps_for = |name: &str| -> BTreeSet<String> {
        let path = crates
            .get(name)
            .unwrap_or_else(|| panic!("missing crate {name}"));
        parse_dependencies(&path.join("Cargo.toml"), &known)
    };

    let cli = deps_for("bijux");
    assert!(cli.contains("bijux-api"), "cli must depend on bijux-api");
    for dep in &cli {
        assert!(
            dep == "bijux-api" || dep == "bijux-guardrails",
            "cli must not depend on workspace crate {dep}"
        );
    }

    for domain in ["bijux-domain-fastq", "bijux-domain-bam", "bijux-domain-vcf"] {
        let deps = deps_for(domain);
        for banned in [
            "bijux-stages-fastq",
            "bijux-stages-bam",
            "bijux-engine",
            "bijux-api",
            "bijux",
            "bijux-pipelines",
        ] {
            assert!(
                !deps.contains(banned),
                "{domain} must not depend on {banned}"
            );
        }
    }

    for stages in ["bijux-stages-fastq", "bijux-stages-bam"] {
        let deps = deps_for(stages);
        for banned in [
            "bijux",
            "bijux-api",
            "bijux-analyze",
            "bijux-bench",
            "bijux-engine",
        ] {
            assert!(
                !deps.contains(banned),
                "{stages} must not depend on {banned}"
            );
        }
    }

    let pipelines = deps_for("bijux-pipelines");
    for banned in ["bijux-engine", "bijux"] {
        assert!(
            !pipelines.contains(banned),
            "bijux-pipelines must not depend on {banned}"
        );
    }
}

#[test]
fn workspace_bans_thin_mod_rs() {
    let mut offenders = Vec::new();
    for path in crate_dirs() {
        for mod_path in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name() == "mod.rs")
        {
            let content = std::fs::read_to_string(mod_path.path()).unwrap_or_default();
            let mut lines = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty()
                    || line.starts_with("//")
                    || line.starts_with("#[")
                    || line.starts_with("/*")
                {
                    continue;
                }
                lines.push(line.to_string());
            }
            if lines.len() == 1 && lines[0].starts_with("pub use ") {
                offenders.push(mod_path.path().display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "thin mod.rs files are not allowed: {offenders:?}"
    );
}
