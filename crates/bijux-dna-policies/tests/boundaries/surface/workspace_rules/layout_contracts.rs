const LAYOUT_EXCLUDE_DIRS: &[&str] = &[".git", "target", "artifacts", "site", "node_modules"];

fn layout_is_excluded(path: &std::path::Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| LAYOUT_EXCLUDE_DIRS.contains(&name))
    })
}

#[test]
fn slow__policy__boundaries__workspace__workspace_no_macos_dotfiles() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if layout_is_excluded(entry.path()) {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.starts_with("._") {
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
        "bijux-dna-bench",
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
        "bijux-dna-bench",
        "bijux-dna-bench-model",
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
    let allowed_files = BTreeSet::from(["errors.rs", "engine_driver.rs", "lib.rs"]);
    let allowed_dirs = BTreeSet::from([
        "control",
        "engine_config",
        "executor",
        "observability",
        "public_api",
    ]);
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&src).expect("read bijux-dna-engine/src") {
        let entry = entry.expect("engine src entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_file() && !allowed_files.contains(name.as_str()) {
            offenders.push(name.clone());
        }
        if entry.path().is_dir() && !allowed_dirs.contains(name.as_str()) {
            offenders.push(name);
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine/src contains unexpected entries: {offenders:?}"
    );
    let required_executor_entries = ["contracts", "graph", "mod.rs", "recording", "step_execution"];
    for entry in required_executor_entries {
        let helper_path = src.join("executor").join(entry);
        bijux_dna_policies::policy_assert!(
            helper_path.exists(),
            "bijux-dna-engine/src/executor missing {entry}"
        );
    }
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
    for file in ["id_catalog.rs", "qc_contract.rs", "domain_adapter.rs"] {
        let path = fastq.join("src").join(file);
        bijux_dna_policies::policy_assert!(
            path.exists(),
            "bijux-dna-domain-fastq missing src/{file}"
        );
    }
    bijux_dna_policies::policy_assert!(
        fastq.join("src").join("stages").exists(),
        "bijux-dna-domain-fastq missing src/stages"
    );
    bijux_dna_policies::policy_assert!(
        fastq.join("src").join("pipeline_contract").exists(),
        "bijux-dna-domain-fastq missing src/pipeline_contract"
    );
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
        let has_plugin = src.join("plugin.rs").exists() || src.join("plugin").join("mod.rs").exists();
        bijux_dna_policies::policy_assert!(
            has_plugin,
            "{name} missing src/plugin.rs or src/plugin/mod.rs"
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
        "bijux-dna-bench",
        "bijux-dna-domain-vcf",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-science",
        "bijux-dna-stages-vcf",
    ]);
    for (name, count) in dependents {
        let crate_dir = crates.get(&name).expect("crate dir");
        if count == 0 && !allowlist.contains(name.as_str()) && !is_bin_crate(crate_dir) {
            bijux_dna_policies::policy_panic!("orphan crate without allowlist: {name}");
        }
    }
}
