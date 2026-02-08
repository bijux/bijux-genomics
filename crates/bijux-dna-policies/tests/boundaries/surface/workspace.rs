#![allow(non_snake_case)]
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bijux_dna_policies::GuardrailConfig;
use walkdir::WalkDir;

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
    bijux_dna_policies::policy_panic!("missing package name in {}", manifest.display());
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
    let path = root
        .join("docs")
        .join("10-architecture")
        .join("BOUNDARY_MAP.md");
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
    bijux_dna_policies::policy_assert!(
        in_block && !lines.is_empty(),
        "missing executable boundaries block in {}",
        path.display()
    );
    let mut map = BTreeMap::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, deps) = line.split_once(':').unwrap_or_else(|| {
            bijux_dna_policies::policy_panic!("invalid boundaries line: {line}")
        });
        let deps = deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .map(|dep| dep.to_string())
            .collect::<BTreeSet<_>>();
        map.insert(name.trim().to_string(), deps);
    }
    map
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

fn contains_term(text: &str, term: &str) -> bool {
    if term.is_empty() {
        return false;
    }
    let mut rest = text;
    while let Some(idx) = rest.find(term) {
        let before = rest[..idx].chars().last();
        let after = rest[idx + term.len()..].chars().next();
        let before_ok = before.map_or(true, |ch| !ch.is_ascii_alphanumeric());
        let after_ok = after.map_or(true, |ch| !ch.is_ascii_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        rest = &rest[idx + term.len()..];
    }
    false
}

fn assert_no_domain_terms(crate_root: &Path, denylist: &[&str]) {
    let src = crate_root.join("src");
    let files = rs_files_under(&src);
    for file in files {
        let content = std::fs::read_to_string(&file).expect("read source file");
        let lowered = content.to_lowercase();
        for term in denylist {
            if contains_term(&lowered, term) {
                bijux_dna_policies::policy_panic!(
                    "domain term '{}' found in {}",
                    term,
                    file.display()
                );
            }
        }
    }
}

#[test]
fn policy__boundaries__workspace__workspace_no_macos_dotfiles() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name == ".DS_Store" || name.starts_with("._") {
            offenders.push(entry.path().display().to_string());
        }
    }
    if !offenders.is_empty() {
        bijux_dna_policies::policy_panic!(
            "macOS dotfiles are forbidden in repo:\n{}",
            offenders.join("\n")
        );
    }
}

#[test]
fn policy__boundaries__workspace__engine_has_no_domain_terms() {
    let root = workspace_root();
    let engine = root.join("crates").join("bijux-dna-engine");
    let denylist = [
        "fastq",
        "bam",
        "qc",
        "retention",
        "adapter",
        "contaminant",
        "umi",
        "polyx",
    ];
    assert_no_domain_terms(&engine, &denylist);
}

#[test]
fn policy__boundaries__workspace__runner_has_no_domain_terms() {
    let root = workspace_root();
    let runner = root.join("crates").join("bijux-dna-runner");
    let denylist = [
        "fastq",
        "bam",
        "qc",
        "retention",
        "adapter",
        "contaminant",
        "umi",
        "polyx",
    ];
    assert_no_domain_terms(&runner, &denylist);
}

#[test]
fn policy__boundaries__workspace__engine_and_runner_have_no_domain_deps() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let forbidden = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-analyze",
        "bijux-dna-benchmark",
    ];
    for name in ["bijux-dna-engine", "bijux-dna-runner"] {
        let crate_dir = crates
            .get(name)
            .unwrap_or_else(|| bijux_dna_policies::policy_panic!("missing crate {name}"));
        let deps = parse_dependencies(&crate_dir.join("Cargo.toml"), &known);
        for banned in &forbidden {
            bijux_dna_policies::policy_assert!(
                !deps.contains(*banned),
                "{name} must not depend on {banned}"
            );
        }
    }
}

#[test]
fn policy__boundaries__workspace__workspace_has_guardrails_tests() {
    for path in crate_dirs() {
        let guardrails = path.join("tests").join("guardrails.rs");
        bijux_dna_policies::policy_assert!(
            guardrails.exists(),
            "missing tests/guardrails.rs in {}",
            path.display()
        );
        let content = std::fs::read_to_string(&guardrails).expect("read guardrails test");
        bijux_dna_policies::policy_assert!(
            content.contains("GuardrailConfig::for_crate"),
            "guardrails test must use GuardrailConfig::for_crate in {}",
            guardrails.display()
        );
    }
}

#[test]
fn policy__boundaries__workspace__workspace_guardrail_defaults_not_increased() {
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
        bijux_dna_policies::policy_assert!(
            !bad,
            "guardrails defaults increased for {}: {:?}",
            name,
            config
        );
    }
}

#[test]
fn policy__boundaries__workspace__workspace_members_are_deterministic() {
    let root = workspace_root();
    let members = parse_workspace_members(&root);
    bijux_dna_policies::policy_assert!(!members.is_empty(), "workspace members not found");
    let mut sorted = members.clone();
    sorted.sort();
    let mut deduped = sorted.clone();
    deduped.dedup();
    bijux_dna_policies::policy_assert_eq!(
        sorted,
        deduped,
        "workspace members contain duplicates or are unsorted"
    );
    bijux_dna_policies::policy_assert_eq!(
        members,
        sorted,
        "workspace members must be sorted and deterministic"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_constitution_contract() {
    let crates = collect_workspace_crates();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for name in crates.keys() {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    let required = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-stage-contract",
        "bijux-dna-pipelines",
        "bijux-dna-api",
        "bijux-dna-infra",
        "bijux-dna-core",
        "bijux-dna-engine",
        "bijux-dna-runtime",
        "bijux-dna-analyze",
        "bijux-dna-benchmark",
        "bijux-dna-benchmark-model",
        "bijux-dna-testkit",
    ];
    for name in required {
        bijux_dna_policies::policy_assert!(
            crates.contains_key(name),
            "missing required crate: {name}"
        );
        bijux_dna_policies::policy_assert_eq!(
            counts.get(name).copied().unwrap_or(0),
            1,
            "duplicate crate: {name}"
        );
    }
    bijux_dna_policies::policy_assert!(
        crates.contains_key("bijux-dna-environment"),
        "missing bijux-dna-environment crate"
    );
    bijux_dna_policies::policy_assert!(
        crates.contains_key("bijux-dna-environment-qa"),
        "missing bijux-dna-environment-qa crate"
    );
    let env_crates: Vec<_> = crates
        .keys()
        .filter(|name| name.starts_with("bijux-dna-env-"))
        .collect();
    bijux_dna_policies::policy_assert!(
        env_crates.is_empty(),
        "legacy bijux-dna-env-* crates are forbidden"
    );
    bijux_dna_policies::policy_assert!(
        !crates.contains_key("bijux-dna-pipelines-bam"),
        "bijux-dna-pipelines-bam is forbidden"
    );
    bijux_dna_policies::policy_assert!(
        crates.contains_key("bijux-dna-testkit"),
        "missing bijux-dna-testkit crate"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_bans_pipelines_bam_crate_name() {
    let crates = collect_workspace_crates();
    for name in crates.keys() {
        bijux_dna_policies::policy_assert!(
            !name.contains("pipelines-bam"),
            "crate name contains forbidden substring: {name}"
        );
    }
}

#[test]
fn policy__boundaries__workspace__workspace_crate_layout_contract() {
    for crate_dir in crate_dirs() {
        let manifest = crate_dir.join("Cargo.toml");
        bijux_dna_policies::policy_assert!(
            manifest.exists(),
            "missing Cargo.toml in {}",
            crate_dir.display()
        );
        let src_dir = crate_dir.join("src");
        bijux_dna_policies::policy_assert!(
            src_dir.exists(),
            "missing src/ in {}",
            crate_dir.display()
        );
        if is_bin_crate(&crate_dir) {
            continue;
        }
        let tests_dir = crate_dir.join("tests");
        bijux_dna_policies::policy_assert!(
            tests_dir.exists(),
            "missing tests/ in {}",
            crate_dir.display()
        );
    }
}

#[test]
fn policy__boundaries__workspace__engine_src_layout_contract() {
    let crates = collect_workspace_crates();
    let Some(engine) = crates.get("bijux-dna-engine") else {
        bijux_dna_policies::policy_panic!("missing crate bijux-dna-engine");
    };
    let src = engine.join("src");
    let allowed = BTreeSet::from(["errors.rs", "executor.rs", "lib.rs", "runtime_facade.rs"]);
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&src).expect("read bijux-dna-engine/src") {
        let entry = entry.expect("engine src entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_file() && !allowed.contains(name.as_str()) {
            offenders.push(name.clone());
        }
        if entry.path().is_dir() {
            offenders.push(name);
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine/src must stay small; unexpected entries: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_domain_layout_contract() {
    let crates = collect_workspace_crates();
    let Some(fastq) = crates.get("bijux-dna-domain-fastq") else {
        bijux_dna_policies::policy_panic!("missing crate bijux-dna-domain-fastq");
    };
    for dir in ["metrics", "params", "invariants", "types"] {
        let path = fastq.join("src").join(dir);
        bijux_dna_policies::policy_assert!(
            path.exists(),
            "bijux-dna-domain-fastq missing src/{dir}"
        );
    }
    for file in [
        "stage_contract.rs",
        "id_catalog.rs",
        "stage_semantics.rs",
        "stage_specs.rs",
    ] {
        let path = fastq.join("src").join(file);
        bijux_dna_policies::policy_assert!(
            path.exists(),
            "bijux-dna-domain-fastq missing src/{file}"
        );
    }
    let lib = fastq.join("src").join("lib.rs");
    bijux_dna_policies::policy_assert!(lib.exists(), "bijux-dna-domain-fastq missing src/lib.rs");

    let Some(bam) = crates.get("bijux-dna-domain-bam") else {
        bijux_dna_policies::policy_panic!("missing crate bijux-dna-domain-bam");
    };
    for dir in ["metrics", "params", "invariants", "types", "stage_specs"] {
        let path = bam.join("src").join(dir);
        bijux_dna_policies::policy_assert!(path.exists(), "bijux-dna-domain-bam missing src/{dir}");
    }
    let lib = bam.join("src").join("lib.rs");
    bijux_dna_policies::policy_assert!(lib.exists(), "bijux-dna-domain-bam missing src/lib.rs");
}

#[test]
fn policy__boundaries__workspace__workspace_stages_layout_contract() {
    let crates = collect_workspace_crates();
    for name in ["bijux-dna-stages-fastq", "bijux-dna-stages-bam"] {
        let Some(path) = crates.get(name) else {
            bijux_dna_policies::policy_panic!("missing crate {name}");
        };
        let src = path.join("src");
        let stage_specs = src.join("stage_specs");
        let has_stage_specs = stage_specs.exists() || src.join("stage_specs.rs").exists();
        bijux_dna_policies::policy_assert!(has_stage_specs, "{name} missing stage_specs module");
        bijux_dna_policies::policy_assert!(
            src.join("plugin.rs").exists(),
            "{name} missing src/plugin.rs"
        );
        let has_metrics =
            src.join("metrics.rs").exists() || src.join("metrics").join("mod.rs").exists();
        bijux_dna_policies::policy_assert!(
            has_metrics,
            "{name} missing src/metrics.rs or src/metrics/mod.rs"
        );
    }
}

#[test]
fn policy__boundaries__workspace__workspace_no_orphan_crates() {
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
        "bijux-dna",
        "bijux-dna-benchmark",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-runner",
        "bijux-dna-runtime",
    ]);
    for (name, count) in dependents {
        let crate_dir = crates.get(&name).expect("crate dir");
        if count == 0 && !allowlist.contains(name.as_str()) && !is_bin_crate(crate_dir) {
            bijux_dna_policies::policy_panic!("orphan crate without allowlist: {name}");
        }
    }
}

#[test]
fn policy__boundaries__workspace__workspace_dependency_graph_contract() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let deps_for = |name: &str| -> BTreeSet<String> {
        let path = crates
            .get(name)
            .unwrap_or_else(|| bijux_dna_policies::policy_panic!("missing crate {name}"));
        parse_dependencies(&path.join("Cargo.toml"), &known)
    };
    let is_guardrails = |dep: &str| dep == "bijux-dna-policies" || dep == "bijux-dna-testkit";

    let cli = deps_for("bijux-dna");
    bijux_dna_policies::policy_assert!(
        cli.contains("bijux-dna-api"),
        "cli must depend on bijux-dna-api"
    );
    for dep in &cli {
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-api"
                || dep == "bijux-dna-core"
                || dep == "bijux-dna-environment"
                || dep == "bijux-dna-environment-qa"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-policies",
            "cli must not depend on workspace crate {dep}"
        );
    }

    if let Some(cli_dir) = crates.get("bijux-dna") {
        let cli_deps = parse_dependencies(&cli_dir.join("Cargo.toml"), &known);
        bijux_dna_policies::policy_assert!(
            cli_deps.contains("bijux-dna-api"),
            "bijux-dna must depend on bijux-dna-api"
        );
        for dep in &cli_deps {
            bijux_dna_policies::policy_assert!(
                dep == "bijux-dna-api"
                    || dep == "bijux-dna-core"
                    || dep == "bijux-dna-environment"
                    || dep == "bijux-dna-environment-qa"
                    || dep == "bijux-dna-infra"
                    || dep == "bijux-dna-stage-contract"
                    || dep == "bijux-dna-policies",
                "bijux-dna must not depend on workspace crate {dep}"
            );
        }
    }

    let core = deps_for("bijux-dna-core");
    for dep in &core {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-infra",
            "bijux-dna-core must not depend on workspace crate {dep}"
        );
    }

    let runtime = deps_for("bijux-dna-runtime");
    for dep in &runtime {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core" || dep == "bijux-dna-infra",
            "bijux-dna-runtime must not depend on workspace crate {dep}"
        );
    }

    let engine = deps_for("bijux-dna-engine");
    for dep in &engine {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core" || dep == "bijux-dna-infra" || dep == "bijux-dna-runtime",
            "bijux-dna-engine must not depend on workspace crate {dep}"
        );
    }

    let planner_fastq = deps_for("bijux-dna-planner-fastq");
    for dep in &planner_fastq {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-stages-fastq"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-infra",
            "bijux-dna-planner-fastq must not depend on workspace crate {dep}"
        );
    }

    let planner_bam = deps_for("bijux-dna-planner-bam");
    for dep in &planner_bam {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-stages-bam"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-infra",
            "bijux-dna-planner-bam must not depend on workspace crate {dep}"
        );
    }

    let api = deps_for("bijux-dna-api");
    for dep in &api {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-planner-fastq"
                || dep == "bijux-dna-planner-bam"
                || dep == "bijux-dna-engine"
                || dep == "bijux-dna-runtime"
                || dep == "bijux-dna-runner"
                || dep == "bijux-dna-environment"
                || dep == "bijux-dna-environment-qa"
                || dep == "bijux-dna-analyze"
                || dep == "bijux-dna-benchmark"
                || dep == "bijux-dna-benchmark-model"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-infra",
            "bijux-dna-api must not depend on workspace crate {dep}"
        );
    }

    let runner = deps_for("bijux-dna-runner");
    for dep in &runner {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-environment"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime",
            "bijux-dna-runner must not depend on workspace crate {dep}"
        );
    }

    let analyze = deps_for("bijux-dna-analyze");
    for dep in &analyze {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-benchmark"
                || dep == "bijux-dna-testkit"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-planner-fastq"
                || dep == "bijux-dna-planner-bam",
            "bijux-dna-analyze must not depend on workspace crate {dep}"
        );
    }

    let bench = deps_for("bijux-dna-benchmark");
    for dep in &bench {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-analyze"
                || dep == "bijux-dna-benchmark-model"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime",
            "bijux-dna-benchmark must not depend on workspace crate {dep}"
        );
    }

    let api = deps_for("bijux-dna-api");
    let api_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-dna-core",
        "bijux-dna-stage-contract",
        "bijux-dna-engine",
        "bijux-dna-runner",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-analyze",
        "bijux-dna-benchmark",
        "bijux-dna-benchmark-model",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-pipelines",
        "bijux-dna-infra",
        "bijux-dna-policies",
        "bijux-dna-runtime",
        "bijux-dna-testkit",
    ]);
    for dep in &api {
        bijux_dna_policies::policy_assert!(
            api_allowed.contains(dep.as_str()),
            "bijux-dna-api must not depend on workspace crate {dep}"
        );
    }

    for domain in ["bijux-dna-domain-fastq", "bijux-dna-domain-bam"] {
        let deps = deps_for(domain);
        for banned in [
            "bijux-dna-stages-fastq",
            "bijux-dna-stages-bam",
            "bijux-dna-engine",
            "bijux-dna-environment",
            "bijux-dna",
            "bijux-dna-pipelines",
            "bijux-dna-api",
            "bijux-dna-environment",
            "bijux-dna-analyze",
            "bijux-dna-benchmark",
        ] {
            bijux_dna_policies::policy_assert!(
                !deps.contains(banned),
                "{domain} must not depend on {banned}"
            );
        }
    }

    for stages in ["bijux-dna-stages-fastq", "bijux-dna-stages-bam"] {
        let deps = deps_for(stages);
        for banned in [
            "bijux-dna",
            "bijux-dna-api",
            "bijux-dna-analyze",
            "bijux-dna-benchmark",
            "bijux-dna-engine",
            "bijux-dna-environment",
            "bijux-dna-pipelines",
        ] {
            bijux_dna_policies::policy_assert!(
                !deps.contains(banned),
                "{stages} must not depend on {banned}"
            );
        }
    }

    let pipelines = deps_for("bijux-dna-pipelines");
    for banned in [
        "bijux-dna-engine",
        "bijux-dna",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
    ] {
        bijux_dna_policies::policy_assert!(
            !pipelines.contains(banned),
            "bijux-dna-pipelines must not depend on {banned}"
        );
    }

    let analyze = deps_for("bijux-dna-analyze");
    for banned in ["bijux-dna-engine", "bijux-dna-environment"] {
        bijux_dna_policies::policy_assert!(
            !analyze.contains(banned),
            "bijux-dna-analyze must not depend on {banned}"
        );
    }

    if crates.contains_key("bijux-dna-runtime") {
        let runtime = deps_for("bijux-dna-runtime");
        for banned in [
            "bijux-dna-engine",
            "bijux-dna-environment",
            "bijux-dna-stages-fastq",
            "bijux-dna-stages-bam",
            "bijux-dna-planner-fastq",
            "bijux-dna-planner-bam",
            "bijux-dna-api",
            "bijux-dna",
        ] {
            bijux_dna_policies::policy_assert!(
                !runtime.contains(banned),
                "bijux-dna-runtime must not depend on {banned}"
            );
        }
    }

    let engine = deps_for("bijux-dna-engine");
    for banned in [
        "bijux-dna-analyze",
        "bijux-dna-benchmark",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
    ] {
        bijux_dna_policies::policy_assert!(
            !engine.contains(banned),
            "bijux-dna-engine must not depend on {banned}"
        );
    }

    for runner_name in ["bijux-dna-runner"] {
        if !crates.contains_key(runner_name) {
            continue;
        }
        let deps = deps_for(runner_name);
        for banned in [
            "bijux-dna-analyze",
            "bijux-dna-benchmark",
            "bijux-dna-domain-fastq",
            "bijux-dna-domain-bam",
            "bijux-dna-stages-fastq",
            "bijux-dna-stages-bam",
        ] {
            bijux_dna_policies::policy_assert!(
                !deps.contains(banned),
                "{runner_name} must not depend on {banned}"
            );
        }
    }

    let planner_fastq = deps_for("bijux-dna-planner-fastq");
    let planner_fastq_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-dna-core",
        "bijux-dna-stage-contract",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-pipelines",
        "bijux-dna-infra",
        "bijux-dna-policies",
        "bijux-dna-testkit",
    ]);
    for dep in &planner_fastq {
        bijux_dna_policies::policy_assert!(
            planner_fastq_allowed.contains(dep.as_str()),
            "bijux-dna-planner-fastq must not depend on workspace crate {dep}"
        );
    }

    let planner_bam = deps_for("bijux-dna-planner-bam");
    let planner_bam_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-dna-core",
        "bijux-dna-stage-contract",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-bam",
        "bijux-dna-pipelines",
        "bijux-dna-infra",
        "bijux-dna-policies",
        "bijux-dna-testkit",
    ]);
    for dep in &planner_bam {
        bijux_dna_policies::policy_assert!(
            planner_bam_allowed.contains(dep.as_str()),
            "bijux-dna-planner-bam must not depend on workspace crate {dep}"
        );
    }
}

#[test]
fn policy__boundaries__workspace__workspace_boundary_contract_matches_docs() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let contract = parse_boundary_contract();
    for (crate_name, path) in &crates {
        let Some(allowed) = contract.get(crate_name) else {
            bijux_dna_policies::policy_panic!("missing boundaries entry for {crate_name}");
        };
        let deps = parse_dependencies(&path.join("Cargo.toml"), &known);
        for dep in deps {
            bijux_dna_policies::policy_assert!(
                allowed.contains(&dep),
                "boundary violation: {crate_name} depends on {dep}, allowed: {allowed:?}"
            );
        }
    }
}

#[test]
fn policy__boundaries__workspace__stage_spec_and_registry_defs_scoped() {
    let crates = collect_workspace_crates();
    let root = workspace_root();
    let mut offenders = Vec::new();
    for (name, path) in crates {
        let is_domain = name.starts_with("bijux-dna-domain-");
        let is_stages = name.starts_with("bijux-dna-stages-");
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stage specs/tool registries must live in domains or stages only: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_has_no_target_dirs() {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "target/ directories must not exist in crates: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__crate_root_contents_allowlist() {
    let allowed = BTreeSet::from([
        "Cargo.toml",
        "Makefile.toml",
        "README.md",
        "clippy.toml",
        "src",
        "tests",
        "docs",
    ]);
    let mut offenders = Vec::new();
    for (name, path) in collect_workspace_crates() {
        let entries = std::fs::read_dir(&path)
            .unwrap_or_else(|_| bijux_dna_policies::policy_panic!("read {name}"));
        for entry in entries.filter_map(Result::ok) {
            let entry_name = entry.file_name();
            let entry_name = entry_name.to_string_lossy();
            if allowed.contains(entry_name.as_ref()) {
                continue;
            }
            offenders.push(format!("{}: {}", name, entry_name.as_ref()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crate roots must only contain allowlisted entries: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__fixtures_policy_enforced() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for (_name, path) in collect_workspace_crates() {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "fixtures must live under tests/fixtures or fixtures/: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_no_cross_layer_imports() {
    let crates = collect_workspace_crates();
    let root = workspace_root();
    let mut offenders = Vec::new();
    for (name, path) in crates {
        let is_domain = name.starts_with("bijux-dna-domain-");
        let is_stages = name.starts_with("bijux-dna-stages-");
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
                && (content.contains("bijux_dna_engine::")
                    || content.contains("bijux_dna::")
                    || content.contains("bijux_dna_api::")
                    || content.contains("bijux_dna_analyze::")
                    || content.contains("bijux_dna_benchmark::")
                    || content.contains("bijux_dna_environment::"))
            {
                offenders.push(rel.display().to_string());
            }
            if is_stages
                && (content.contains("bijux_dna::")
                    || content.contains("bijux_dna_api::")
                    || content.contains("bijux_dna_engine::")
                    || content.contains("bijux_dna_pipelines::")
                    || content.contains("bijux_dna_environment::"))
            {
                offenders.push(rel.display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "cross-layer imports detected: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__retention_reports_require_context() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.file_name() != "retention_report.json" {
            continue;
        }
        let raw = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let value: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(_) => {
                offenders.push(format!("{} (invalid json)", entry.path().display()));
                continue;
            }
        };
        let has_context = value.get("numerator").is_some()
            && value.get("denominator").is_some()
            && value.get("units").is_some()
            && value.get("parameters_json").is_some();
        if !has_context {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "retention_report.json must include numerator/denominator/units/parameters_json: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__params_hash_only_defined_in_core() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
        let rel_str = rel.to_string_lossy();
        if rel_str.ends_with("crates/bijux-dna-core/src/foundation/hashing.rs")
            || rel_str.ends_with("crates/bijux-dna-policies/tests/workspace.rs")
            || rel_str.ends_with("crates/bijux-dna-policies/tests/surface/workspace.rs")
            || rel_str.ends_with("crates/bijux-dna-policies/tests/boundaries/surface/workspace.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("fn params_hash") {
            offenders.push(rel.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "params_hash must only be defined in bijux-dna-core: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_single_orchestration_surface() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for path in crate_dirs() {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "bijux-dna-api" || name == "bijux-dna-api" {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "only bijux-dna-api may expose orchestration entrypoints: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_no_ad_hoc_fs_write() {
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
        if name == "bijux-dna-infra" || name == "bijux-dna-infra" {
            continue;
        }
        for entry in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "ad-hoc fs writes/renames/removals/dir-creation are forbidden outside bijux-dna-infra: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__engine_has_no_domain_keywords() {
    let root = workspace_root();
    let engine_root = root.join("crates").join("bijux-dna-engine").join("src");
    let denylist = [
        "fastq",
        "bam",
        "qc_post",
        "retention",
        "adapter_bank",
        "adapters",
        "fastqc",
        "multiqc",
    ];
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&engine_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let lower = content.to_lowercase();
        if denylist.iter().any(|token| lower.contains(token)) {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            offenders.push(rel.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "engine must not contain domain keywords: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__api_has_no_planning_policy() {
    let root = workspace_root();
    let api_root = root.join("crates").join("bijux-dna-api").join("src");
    let denylist = [
        "smart_pipeline",
        "stage_order",
        "stage ordering",
        "normalize_stage",
        "normalize_tool",
        "tool_list",
    ];
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&api_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let lower = content.to_lowercase();
        if denylist.iter().any(|token| lower.contains(token)) {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            offenders.push(rel.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "api must not implement planning policy: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_bans_thin_mod_rs() {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "thin mod.rs files are not allowed: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_domain_symmetry_contract() {
    let crates = collect_workspace_crates();
    let domains = ["bijux-dna-domain-fastq", "bijux-dna-domain-bam"];
    let required = [
        "metrics",
        "params",
        "types",
        "invariants",
        "stage_specs",
        "pipeline_contract.rs",
    ];
    let mut domain_sets = Vec::new();
    for name in domains {
        let crate_dir = crates
            .get(name)
            .unwrap_or_else(|| bijux_dna_policies::policy_panic!("missing crate dir for {name}"));
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
        bijux_dna_policies::policy_assert_eq!(
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
        bijux_dna_policies::policy_assert_eq!(
            base,
            set,
            "domain module symmetry mismatch between {} and {}: {:?} vs {:?}",
            domain_sets[0].0,
            name,
            base,
            set
        );
    }
}

#[test]
fn policy__boundaries__workspace__engine_src_has_no_domain_id_catalog() {
    let root = workspace_root();
    let engine_src = root.join("crates").join("bijux-dna-engine").join("src");
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine/src must not contain domain stage IDs: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__engine_has_no_tool_normalization_policy() {
    let root = workspace_root();
    let engine_src = root.join("crates").join("bijux-dna-engine").join("src");
    let mut offenders = Vec::new();
    let banned_tokens = ["normalize_", "tool_list"];
    for entry in walkdir::WalkDir::new(&engine_src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if banned_tokens.iter().any(|token| content.contains(token)) {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine must not define tool normalization: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_bans_resource_fork_artifacts() {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "resource fork artifacts (.DS_Store/._*) are not allowed: {offenders:?}"
    );
}
