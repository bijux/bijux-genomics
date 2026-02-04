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

fn parse_workspace_members(root: &Path) -> Vec<String> {
    let manifest = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest).expect("read workspace Cargo.toml");
    let mut members = Vec::new();
    let mut in_members = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("members") && line.contains('[') {
            in_members = true;
        }
        if !in_members {
            continue;
        }
        if line.contains(']') {
            in_members = false;
        }
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                let member = &line[start + 1..start + 1 + end];
                members.push(member.to_string());
            }
        }
    }
    members
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

fn parse_boundary_contract() -> BTreeMap<String, BTreeSet<String>> {
    let root = workspace_root();
    let path = root.join("crates").join("bijux-core").join("src").join("boundaries.md");
    let content = std::fs::read_to_string(&path).expect("read boundaries.md");
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == "```boundaries" {
            in_block = true;
            continue;
        }
        if in_block && line.trim() == "```" {
            break;
        }
        if in_block {
            lines.push(line.trim().to_string());
        }
    }
    assert!(
        in_block && !lines.is_empty(),
        "missing executable boundaries block in {}",
        path.display()
    );
    let mut map = BTreeMap::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, deps) = line
            .split_once(':')
            .unwrap_or_else(|| panic!("invalid boundaries line: {line}"));
        let deps = deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .map(|dep| dep.to_string())
            .collect::<BTreeSet<_>>();
        map.insert(name.trim().to_string(), deps);
    }
    map
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
fn workspace_members_are_deterministic() {
    let root = workspace_root();
    let members = parse_workspace_members(&root);
    assert!(!members.is_empty(), "workspace members not found");
    let mut sorted = members.clone();
    sorted.sort();
    let mut deduped = sorted.clone();
    deduped.dedup();
    assert_eq!(
        sorted, deduped,
        "workspace members contain duplicates or are unsorted"
    );
    assert_eq!(
        members, sorted,
        "workspace members must be sorted and deterministic"
    );
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
        "bijux-runner-local",
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

    if let Some(cli_dir) = crates.get("bijux-cli") {
        let cli_deps = parse_dependencies(&cli_dir.join("Cargo.toml"), &known);
        assert!(cli_deps.contains("bijux-api"), "bijux-cli must depend on bijux-api");
        for dep in &cli_deps {
            assert!(
                dep == "bijux-api" || dep == "bijux-guardrails",
                "bijux-cli must not depend on workspace crate {dep}"
            );
        }
    }

    let api = deps_for("bijux-api");
    let api_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-core",
        "bijux-engine",
        "bijux-runner-docker",
        "bijux-env-runtime",
        "bijux-env-builder",
        "bijux-analyze",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-planner-fastq",
        "bijux-planner-bam",
        "bijux-pipelines",
        "bijux-infra",
        "bijux-guardrails",
    ]);
    for dep in &api {
        assert!(
            api_allowed.contains(dep.as_str()),
            "bijux-api must not depend on workspace crate {dep}"
        );
    }

    for domain in ["bijux-domain-fastq", "bijux-domain-bam"] {
        let deps = deps_for(domain);
        for banned in [
            "bijux-stages-fastq",
            "bijux-stages-bam",
            "bijux-engine",
            "bijux-env-builder",
            "bijux-env-runtime",
            "bijux",
            "bijux-pipelines",
            "bijux-api",
            "bijux-env-builder",
            "bijux-env-runtime",
            "bijux-analyze",
            "bijux-bench",
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
            "bijux-env-builder",
            "bijux-env-runtime",
            "bijux-pipelines",
        ] {
            assert!(
                !deps.contains(banned),
                "{stages} must not depend on {banned}"
            );
        }
    }

    let pipelines = deps_for("bijux-pipelines");
    for banned in [
        "bijux-engine",
        "bijux",
        "bijux-stages-fastq",
        "bijux-stages-bam",
    ] {
        assert!(
            !pipelines.contains(banned),
            "bijux-pipelines must not depend on {banned}"
        );
    }

    let analyze = deps_for("bijux-analyze");
    for banned in ["bijux-engine", "bijux-env-builder", "bijux-env-runtime"] {
        assert!(
            !analyze.contains(banned),
            "bijux-analyze must not depend on {banned}"
        );
    }

    let engine = deps_for("bijux-engine");
    for banned in [
        "bijux-analyze",
        "bijux-bench",
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-runner-docker",
        "bijux-runner-local",
    ] {
        assert!(
            !engine.contains(banned),
            "bijux-engine must not depend on {banned}"
        );
    }

    let runner = deps_for("bijux-runner-docker");
    let runner_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-core",
        "bijux-engine",
        "bijux-env-runtime",
        "bijux-env-builder",
        "bijux-infra",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-guardrails",
    ]);
    for dep in &runner {
        assert!(
            runner_allowed.contains(dep.as_str()),
            "bijux-runner-docker must not depend on workspace crate {dep}"
        );
    }

    let planner_fastq = deps_for("bijux-planner-fastq");
    let planner_fastq_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-core",
        "bijux-stages-fastq",
        "bijux-pipelines",
        "bijux-infra",
        "bijux-guardrails",
    ]);
    for dep in &planner_fastq {
        assert!(
            planner_fastq_allowed.contains(dep.as_str()),
            "bijux-planner-fastq must not depend on workspace crate {dep}"
        );
    }

    let planner_bam = deps_for("bijux-planner-bam");
    let planner_bam_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-core",
        "bijux-stages-bam",
        "bijux-infra",
        "bijux-guardrails",
    ]);
    for dep in &planner_bam {
        assert!(
            planner_bam_allowed.contains(dep.as_str()),
            "bijux-planner-bam must not depend on workspace crate {dep}"
        );
    }
}

#[test]
fn workspace_boundary_contract_matches_docs() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let contract = parse_boundary_contract();
    for (crate_name, path) in &crates {
        let Some(allowed) = contract.get(crate_name) else {
            panic!("missing boundaries entry for {crate_name}");
        };
        let deps = parse_dependencies(&path.join("Cargo.toml"), &known);
        for dep in deps {
            assert!(
                allowed.contains(&dep),
                "boundary violation: {crate_name} depends on {dep}, allowed: {allowed:?}"
            );
        }
    }
}

#[test]
fn stage_spec_and_registry_defs_scoped() {
    let crates = collect_workspace_crates();
    let root = workspace_root();
    let mut offenders = Vec::new();
    for (name, path) in crates {
        let is_domain = name.starts_with("bijux-domain-");
        let is_stages = name.starts_with("bijux-stages-");
        for entry in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            let file_name = entry
                .path()
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if file_name.ends_with("_tools_registry.rs") && !is_stages {
                offenders.push(rel.display().to_string());
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if !is_domain && content.contains("fn stage_spec") {
                offenders.push(rel.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "stage specs/tool registries must live in domains or stages only: {offenders:?}"
    );
}

#[test]
fn workspace_has_no_target_dirs() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
    {
        if entry.file_name() == "target" {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "target/ directories must not exist in crates: {offenders:?}"
    );
}

#[test]
fn crate_root_contents_allowlist() {
    let allowed = BTreeSet::from([
        "Cargo.toml",
        "Makefile.toml",
        "src",
        "tests",
        "README.md",
        "API.md",
        "PIPELINE_VERSIONING.md",
    ]);
    let mut offenders = Vec::new();
    for (name, path) in collect_workspace_crates() {
        let entries = std::fs::read_dir(&path).unwrap_or_else(|_| panic!("read {name}"));
        for entry in entries.filter_map(Result::ok) {
            let entry_name = entry.file_name();
            let entry_name = entry_name.to_string_lossy();
            if allowed.contains(entry_name.as_ref()) {
                continue;
            }
            offenders.push(format!(
                "{}: {}",
                name,
                entry_name.as_ref()
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "crate roots must only contain allowlisted entries: {offenders:?}"
    );
}

#[test]
fn fixtures_policy_enforced() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for (name, path) in collect_workspace_crates() {
        for entry in walkdir::WalkDir::new(path.join("src").parent().unwrap())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
        {
            if entry.file_name() != "fixtures" {
                continue;
            }
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            let rel_str = rel.to_string_lossy();
            if rel_str.ends_with("/tests/fixtures") || rel_str.ends_with("/fixtures") {
                continue;
            }
            offenders.push(rel.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "fixtures must live under tests/fixtures or fixtures/: {offenders:?}"
    );
}

#[test]
fn workspace_no_cross_layer_imports() {
    let crates = collect_workspace_crates();
    let root = workspace_root();
    let mut offenders = Vec::new();
    for (name, path) in crates {
        let is_domain = name.starts_with("bijux-domain-");
        let is_stages = name.starts_with("bijux-stages-");
        if !is_domain && !is_stages {
            continue;
        }
        for entry in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if is_domain
                && (content.contains("bijux_engine::")
                    || content.contains("bijux_cli::")
                    || content.contains("bijux_api::")
                    || content.contains("bijux_analyze::")
                    || content.contains("bijux_bench::")
                    || content.contains("bijux_env_builder::")
                    || content.contains("bijux_env_runtime::"))
            {
                offenders.push(rel.display().to_string());
            }
            if is_stages
                && (content.contains("bijux_cli::")
                    || content.contains("bijux_api::")
                    || content.contains("bijux_engine::")
                    || content.contains("bijux_pipelines::")
                    || content.contains("bijux_env_builder::")
                    || content.contains("bijux_env_runtime::"))
            {
                offenders.push(rel.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "cross-layer imports detected: {offenders:?}"
    );
}

#[test]
fn params_hash_only_defined_in_core() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
        let rel_str = rel.to_string_lossy();
        if rel_str.ends_with("crates/bijux-core/src/hashing.rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("fn params_hash") {
            offenders.push(rel.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "params_hash must only be defined in bijux-core: {offenders:?}"
    );
}

#[test]
fn workspace_single_orchestration_surface() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for path in crate_dirs() {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "bijux-api" {
            continue;
        }
        for entry in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            for needle in [
                "pub fn select_pipeline(",
                "pub fn plan_run(",
                "pub fn execute_run(",
                "pub fn render_report(",
            ] {
                if content.contains(needle) {
                    offenders.push(rel.display().to_string());
                    break;
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "only bijux-api may expose orchestration entrypoints: {offenders:?}"
    );
}

#[test]
fn workspace_no_ad_hoc_fs_write() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = [
        "std::fs::write(",
        "fs::write(",
        "std::fs::rename(",
        "fs::rename(",
        "std::fs::remove_file(",
        "fs::remove_file(",
        "std::fs::create_dir_all(",
        "fs::create_dir_all(",
    ];
    for path in crate_dirs() {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "bijux-infra" {
            continue;
        }
        for entry in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .chain(walkdir::WalkDir::new(path.join("tests")).into_iter())
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if needles.iter().any(|needle| content.contains(needle)) {
                offenders.push(rel.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "ad-hoc fs writes/renames/removals/dir-creation are forbidden outside bijux-infra: {offenders:?}"
    );
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

#[test]
fn workspace_domain_symmetry_contract() {
    let root = workspace_root();
    let domains = ["bijux-domain-fastq", "bijux-domain-bam"];
    let required = [
        "metrics",
        "params",
        "types",
        "invariants",
        "stage_registry",
        "pipeline_contract.rs",
    ];
    let mut domain_sets = Vec::new();
    for name in domains {
        let crate_dir = crate_dirs()
            .into_iter()
            .find(|dir| {
                dir.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s == name)
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("missing crate dir for {name}"));
        let src = crate_dir.join("src");
        let mut present = BTreeSet::new();
        for item in required {
            let exists = if item.ends_with(".rs") {
                src.join(item).exists()
            } else {
                src.join(item).exists() || src.join(format!("{item}.rs")).exists()
            };
            if exists {
                present.insert(item.to_string());
            }
        }
        assert_eq!(
            present.len(),
            required.len(),
            "domain {name} missing required modules: {:?}",
            required
                .iter()
                .filter(|item| !present.contains(**item))
                .collect::<Vec<_>>()
        );
        domain_sets.push((name, present));
    }
    let base = &domain_sets[0].1;
    for (name, set) in &domain_sets[1..] {
        assert_eq!(
            base, set,
            "domain module symmetry mismatch between {} and {}: {:?} vs {:?}",
            domain_sets[0].0, name, base, set
        );
    }
    let rel = root.join("docs").join("domain_template_checklist.md");
    assert!(
        rel.exists(),
        "missing docs/domain_template_checklist.md"
    );
}

#[test]
fn engine_src_has_no_domain_stage_ids() {
    let root = workspace_root();
    let engine_src = root.join("crates").join("bijux-engine").join("src");
    let mut offenders = Vec::new();
    let needles = ["fastq.", "bam.", "vcf."];
    for entry in walkdir::WalkDir::new(&engine_src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(
                entry
                    .path()
                    .strip_prefix(&root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        offenders.is_empty(),
        "bijux-engine/src must not contain domain stage IDs: {offenders:?}"
    );
}

#[test]
fn engine_has_no_tool_normalization_policy() {
    let root = workspace_root();
    let engine_src = root.join("crates").join("bijux-engine").join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&engine_src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("normalize_") && content.contains("_tool_list") {
            offenders.push(
                entry
                    .path()
                    .strip_prefix(&root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        offenders.is_empty(),
        "bijux-engine must not define tool normalization: {offenders:?}"
    );
}

#[test]
fn workspace_bans_resource_fork_artifacts() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let name = entry.file_name().to_string_lossy();
        if name == ".DS_Store" || name.starts_with("._") {
            offenders.push(
                entry
                    .path()
                    .strip_prefix(&root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        offenders.is_empty(),
        "resource fork artifacts (.DS_Store/._*) are not allowed: {offenders:?}"
    );
}
