#[test]
fn policy__boundaries__workspace__workspace_no_ad_hoc_fs_write() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let legacy_allowlist: BTreeSet<&str> = BTreeSet::from([
        "crates/bijux-dna/src/commands/bench_suite/bench_suite_part1.rs",
        "crates/bijux-dna/src/commands/benchmark_config.rs",
        "crates/bijux-dna/src/commands/benchmark_corpus_fastq.rs",
        "crates/bijux-dna/src/commands/benchmark_repo_checks.rs",
        "crates/bijux-dna/src/commands/benchmark_taxonomy_database.rs",
        "crates/bijux-dna/src/commands/benchmark_workspace.rs",
        "crates/bijux-dna/src/commands/benchmark_publication/corpus_dossier.rs",
        "crates/bijux-dna/src/commands/benchmark_publication/docs_status.rs",
        "crates/bijux-dna/src/commands/benchmark_publication/dossier_index.rs",
        "crates/bijux-dna/src/commands/benchmark_publication/mod.rs",
        "crates/bijux-dna/src/commands/benchmark_publication/remediation.rs",
        "crates/bijux-dna/src/commands/benchmark_publication/results_status.rs",
        "crates/bijux-dna/src/commands/cli/env/env_registry_commands.rs",
        "crates/bijux-dna/src/commands/cli/env/env_runtime_support.rs",
        "crates/bijux-dna/src/commands/corpus.rs",
        "crates/bijux-dna/src/commands/ena/ena_impl.rs",
        "crates/bijux-dna/src/commands/example.rs",
        "crates/bijux-dna/src/commands/hpc/hpc_impl.rs",
        "crates/bijux-dna/src/commands/vcf/vcf_impl.rs",
        "crates/bijux-dna-api/src/internal/fastq/stages/merge_pairs.rs",
        "crates/bijux-dna-api/src/internal/fastq/stages/trim_reads.rs",
        "crates/bijux-dna-api/src/internal/fastq/stages/trim_terminal_damage.rs",
        "crates/bijux-dna-api/src/support/reference_resolution.rs",
        "crates/bijux-dna-db-ena/src/cli/manifest.rs",
        "crates/bijux-dna-db-ena/src/download/execute.rs",
        "crates/bijux-dna-db-ena/src/download.rs",
        "crates/bijux-dna-db-ena/src/main.rs",
        "crates/bijux-dna-core/src/foundation/input_assessment.rs",
        "crates/bijux-dna-dev/src/commands/ops/mod.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline/calling/mod.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline/imputation/postprocess.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline/imputation/workflow.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline/orchestration/mod.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline/population_panel/panel_output.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/imputation/imputation_core.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/imputation/impute_and_postprocess_workflow.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/execution/population_and_panel_prep_helpers.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/execution/call_filter_and_gl.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/execution/runtime_and_orchestration.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/postprocess/filter_and_stats_stages.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/postprocess/postprocess_output_normalization.rs",
        "crates/bijux-dna-stages-vcf/src/vcf_io.rs",
        "crates/bijux-dna-api/src/internal/fastq/stages/preprocess/stage_backend_policy.rs",
        "crates/bijux-dna-api/src/internal/handlers/cross/bam_exec_contracts.rs",
        "crates/bijux-dna-db-ena/src/manifest_store.rs",
        "crates/bijux-dna-db-ena/src/download/transfer.rs",
        "crates/bijux-dna/src/commands/benchmark/publication/results_status.rs",
        "crates/bijux-dna/src/commands/benchmark/publication/corpus_dossier.rs",
        "crates/bijux-dna/src/commands/benchmark/publication/docs_status.rs",
        "crates/bijux-dna/src/commands/benchmark/publication/mod.rs",
        "crates/bijux-dna/src/commands/benchmark/publication/remediation.rs",
        "crates/bijux-dna/src/commands/benchmark/publication/dossier_index.rs",
        "crates/bijux-dna/src/commands/benchmark/taxonomy_database.rs",
        "crates/bijux-dna/src/commands/benchmark/config.rs",
        "crates/bijux-dna/src/commands/benchmark/workspace/layout_normalization.rs",
        "crates/bijux-dna/src/commands/benchmark/workspace/layout_status.rs",
        "crates/bijux-dna/src/commands/benchmark/corpus_fastq/report_qc_support.rs",
        "crates/bijux-dna/src/commands/benchmark/corpus_fastq/mod.rs",
        "crates/bijux-dna/src/commands/benchmark/corpus_fastq/sortmerna_support.rs",
        "crates/bijux-dna/src/commands/benchmark/workspace.rs",
        "crates/bijux-dna/src/commands/benchmark/repo_checks.rs",
        "crates/bijux-dna/src/commands/corpus/mod.rs",
        "crates/bijux-dna-api/src/support/reference_resolution/local.rs",
    ]);
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
        if name == "bijux-dna-infra" {
            continue;
        }
        for entry in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        {
            let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            let rel_string = rel.display().to_string();
            if legacy_allowlist.contains(rel_string.as_str()) {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if needles.iter().any(|needle| content.contains(needle)) {
                offenders.push(rel_string);
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
    let allowlist = [
        "/crates/bijux-dna-core/src/public_api/metrics/mod.rs",
        "/crates/bijux-dna-core/src/public_api/identity/mod.rs",
        "/crates/bijux-dna-core/src/public_api/contracts/mod.rs",
        "/crates/bijux-dna-core/src/public_api/catalog/mod.rs",
        "/crates/bijux-dna-core/src/public_api/ergonomics/mod.rs",
    ];
    for path in crate_dirs() {
        for mod_path in walkdir::WalkDir::new(path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name() == "mod.rs")
        {
            let path_s = mod_path.path().to_string_lossy();
            if allowlist.iter().any(|allowed| path_s.ends_with(allowed)) {
                continue;
            }
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
        "pipeline_contract",
    ];
    let mut domain_sets = Vec::new();
    for name in domains {
        let crate_dir = crates
            .get(name)
            .unwrap_or_else(|| bijux_dna_policies::policy_panic!("missing crate dir for {name}"));
        let src = crate_dir.join("src");
        let mut present = BTreeSet::new();
        for item in required {
            let exists = src.join(item).exists() || src.join(format!("{item}.rs")).exists();
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
    let banned_tokens = ["normalize_tool", "normalize_stage", "tool_list"];
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
fn slow__policy__boundaries__workspace__workspace_bans_resource_fork_artifacts() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if is_excluded(entry.path()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.starts_with("._") {
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
        "resource fork artifacts (._*) are not allowed: {offenders:?}"
    );
}

#[test]
fn policy__boundaries__workspace__workspace_has_no_legacy_bijux_packages() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() == "Cargo.toml")
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("name") {
                continue;
            }
            if let Some((_, value)) = line.split_once('=') {
                let name = value.trim().trim_matches('"');
                if name.starts_with("bijux-") && !name.starts_with("bijux-dna") {
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
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "workspace Cargo.toml package names must use bijux-dna-* (no legacy bijux-*): {offenders:?}"
    );
}
const EXCLUDE_DIRS: &[&str] = &[".git", "target", "artifacts", "site", "node_modules"];

fn is_excluded(path: &std::path::Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| EXCLUDE_DIRS.contains(&name))
    })
}
