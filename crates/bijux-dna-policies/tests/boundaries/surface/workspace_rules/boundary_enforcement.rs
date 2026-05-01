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
            if dep == "bijux-dna-policies" || dep == "bijux-dna-testkit" {
                continue;
            }
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
            if !is_domain
                && content.contains("fn stage_spec")
                && !rel
                    .to_string_lossy()
                    .ends_with("crates/bijux-dna-stage-contract/src/plan_run/mod.rs")
            {
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
        "BOUNDARY.md",
        "PUBLIC_API.md",
        "clippy.toml",
        "src",
        "tests",
        "docs",
        "bench",
        "examples",
        "artifacts",
    ]);
    let mut offenders = Vec::new();
    for (name, path) in collect_workspace_crates() {
        let entries = std::fs::read_dir(&path)
            .unwrap_or_else(|_| bijux_dna_policies::policy_panic!("read {name}"));
        for entry in entries.filter_map(Result::ok) {
            let entry_name = entry.file_name();
            let entry_name = entry_name.to_string_lossy();
            if entry_name.as_ref() == ".DS_Store" {
                continue;
            }
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
                    || content.contains("bijux_dna_bench::")
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
fn slow__policy__boundaries__workspace__retention_reports_require_context() {
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
        let value: serde_json::Value = if let Ok(value) = serde_json::from_str(&raw) {
            value
        } else {
            offenders.push(format!("{} (invalid json)", entry.path().display()));
            continue;
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
            || rel_str.ends_with(
                "crates/bijux-dna-policies/tests/boundaries/surface/workspace_rules/boundary_enforcement.rs",
            )
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
        if name == "bijux-dna-api" {
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
