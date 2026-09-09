use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn bench_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_docs_tree(root);
    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "bench/", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries(["artifacts/", "lib.rs", "public_api/", "repo/", "workflow/"]),
        "src tree must match the documented benchmark layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/artifacts")),
        entries(["admission/", "mod.rs", "source_assets.rs", "writer/"]),
        "artifacts tree must separate source inspection, output admission, and serialization"
    );
    assert_eq!(
        dir_entries(&root.join("src/artifacts/admission")),
        entries(["mod.rs", "model.rs", "validation.rs"]),
        "benchmark admission must separate typed evidence from output validation"
    );

    assert_eq!(
        dir_entries(&root.join("src/artifacts/writer")),
        entries([
            "effective_parameters.rs",
            "mod.rs",
            "observation_reader.rs",
            "observation_writer.rs",
            "structured_writer.rs",
        ]),
        "artifact writer tree must separate observation loading, observation persistence, and structured report writing"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["mod.rs", "stable_surface.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo")),
        entries([
            "case_data_authority.rs",
            "catalog_registry_authority.rs",
            "catalog_snapshot_authority.rs",
            "corpus/",
            "data_index_authority.rs",
            "mod.rs",
            "pipeline_reference.rs",
            "repo_root.rs",
            "repository.rs",
            "run_artifacts/",
            "run_metadata.rs",
            "sqlite/",
            "stage_inventory/",
            "workspace_paths.rs",
        ]),
        "repo tree must stay split between repository policy and persisted artifacts"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/corpus")),
        entries(["bam.rs", "fastq.rs", "mod.rs", "normalized_fastq/"]),
        "corpus tree must separate governed admission from normalized read metadata"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/corpus/normalized_fastq")),
        entries(["manifest.rs", "mod.rs", "tests.rs"]),
        "normalized corpus admission must separate manifest IO from sample metadata"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/run_artifacts")),
        entries(["manifest_loader.rs", "metrics_loader.rs", "mod.rs", "observations_loader.rs",]),
        "run artifact loaders must stay separated by persisted artifact kind"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/sqlite/queries")),
        entries(["mod.rs", "run_index/"]),
        "sqlite query tree must stay focused on explicit repository query families"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/sqlite/queries/run_index")),
        entries(["metadata_paths.rs", "mod.rs"]),
        "run index query tree must separate repository queries from metadata path policy"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow")),
        entries([
            "alias_resolution.rs",
            "appraisal.rs",
            "attempt_migration.rs",
            "bam_alignment/",
            "bam_campaign.rs",
            "bam_coverage/",
            "bam_damage/",
            "bam_diversity_catalog.rs",
            "bam_duplication_metrics/",
            "bam_endogenous_content/",
            "bam_genotyping/",
            "bam_mapping_summary/",
            "bam_observation/",
            "bam_picard_cohort/",
            "benchmark_config.rs",
            "benchmark_matrix.rs",
            "campaign_cases.rs",
            "campaign_lifecycle/",
            "campaign_engine/",
            "campaign_material/",
            "citation.rs",
            "corpus_acquisition.rs",
            "corpus_materialization/",
            "container_report_portfolio.rs",
            "cross_campaign.rs",
            "data_portfolio_audit.rs",
            "dossier_index.rs",
            "evaluation.rs",
            "evidence_book.rs",
            "fastq_campaign.rs",
            "fastq_detect_adapters/",
            "fastq_expectations/",
            "fastq_profile_read_lengths/",
            "fastq_profile_reads/",
            "fastq_validate_reads/",
            "full_benchmark_report.rs",
            "hardening_campaign.rs",
            "local_corpus_skip.rs",
            "mod.rs",
            "options.rs",
            "parameter_authority.rs",
            "pipeline_profiles/",
            "progress_ledger.rs",
            "protocol_portfolio.rs",
            "publication_receipt.rs",
            "publication_registry.rs",
            "publication_selection.rs",
            "reference_materialization.rs",
            "replay.rs",
            "report_load.rs",
            "run_index.rs",
            "run_registry.rs",
            "run_suite/",
            "runtime_selection.rs",
            "scientific_readiness.rs",
            "source_authority.rs",
            "stage_coverage.rs",
            "stage_families.rs",
            "stage_inventory.rs",
            "stage_universe.rs",
            "stage_report/",
            "stage_result.rs",
            "stage_tool_closure.rs",
            "summary/",
            "suite_load.rs",
            "summary_fairness.rs",
            "summary_scope.rs",
            "summary_statistics.rs",
            "vcf_campaign.rs",
            "vcf_stage_catalog.rs",
            "vcf_stage_matrix.rs",
            "workspace_convergence.rs",
            "workspace_layout.rs",
        ]),
        "workflow tree must stay partitioned by enduring benchmark concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/pipeline_profiles")),
        entries([
            "amplicon.rs",
            "ancient_dna.rs",
            "evidence.rs",
            "germline.rs",
            "imputation.rs",
            "mod.rs",
            "population.rs",
            "taxonomy.rs",
            "tests.rs",
        ]),
        "pipeline profiles must separate scientific rules from evidence acquisition"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/corpus_materialization")),
        entries(["catalog.rs", "fastq.rs", "mod.rs"]),
        "corpus materialization must separate transfer orchestration, FASTQ validation, and catalog projection"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_damage")),
        entries(["commands.rs", "mod.rs", "report/"]),
        "BAM damage qualification must separate execution from reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_damage/report")),
        entries(["mod.rs", "science.rs"]),
        "BAM damage reporting must keep stage science separate"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_duplication_metrics")),
        entries(["commands.rs", "mod.rs", "report/"]),
        "BAM duplication-metrics material must separate execution from reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_duplication_metrics/report")),
        entries(["mod.rs", "science.rs"]),
        "BAM duplication-metrics reporting must keep stage science separate"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_genotyping")),
        entries(["commands.rs", "mod.rs", "report/"]),
        "BAM genotyping material must separate execution from reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_genotyping/report")),
        entries(["evidence.rs", "mod.rs", "science.rs"]),
        "BAM genotyping reporting must separate evidence projection from scientific validation"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_mapping_summary")),
        entries(["commands.rs", "inputs.rs", "mod.rs", "parameters.rs", "report/"]),
        "BAM mapping-summary material must separate commands, inputs, parameter evidence, and reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/bam_mapping_summary/report")),
        entries(["evidence.rs", "mod.rs", "rendering.rs", "science.rs"]),
        "BAM report derivation must separate attempt evidence, science, and rendering"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/campaign_engine")),
        entries([
            "aggregation.rs",
            "mod.rs",
            "persistence.rs",
            "plugins.rs",
            "planning.rs",
            "reporting.rs",
        ]),
        "campaign engine must keep persisted models in bench-model and separate planning, persistence, aggregation, and reporting"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/campaign_lifecycle")),
        entries([
            "authorities.rs",
            "binding_case_material.rs",
            "execution.rs",
            "mod.rs",
            "normalization.rs",
            "operator.rs",
            "planning.rs",
            "preparation.rs",
            "storage.rs",
        ]),
        "campaign lifecycle tree must separate source authority, operator composition, preparation, planning, execution, and storage logic"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_detect_adapters")),
        entries(["commands.rs", "mod.rs", "parameters.rs", "report/"]),
        "FASTQ detect-adapters material must separate execution, parameters, and reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_detect_adapters/report")),
        entries(["evidence.rs", "mod.rs", "rendering.rs", "science.rs"]),
        "FASTQ detect-adapters reporting must separate evidence, science, and rendering"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_expectations")),
        entries(["mod.rs", "profile_reads.rs", "rendering.rs", "storage.rs"]),
        "FASTQ expectation workflow must separate derivation, rendering, and immutable storage"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_profile_read_lengths")),
        entries(["commands.rs", "inputs.rs", "mod.rs", "parameters.rs", "report/"]),
        "FASTQ campaign material must separate commands, inputs, parameter evidence, and reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_profile_read_lengths/report")),
        entries(["evidence.rs", "mod.rs", "rendering.rs", "science.rs"]),
        "FASTQ report derivation must separate attempt evidence, science, and rendering"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_profile_reads")),
        entries(["commands.rs", "inputs.rs", "mod.rs", "parameters.rs", "report/"]),
        "FASTQ profile campaign material must separate execution and reporting"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_profile_reads/report")),
        entries(["evidence.rs", "mod.rs", "rendering.rs", "science.rs"]),
        "FASTQ profile reporting must separate evidence, science, and rendering"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_validate_reads")),
        entries(["commands.rs", "mod.rs", "parameters.rs", "planner_contract.rs", "report/"]),
        "FASTQ validation material must separate planner projection, execution, and parameters"
    );
    assert_eq!(
        dir_entries(&root.join("src/workflow/fastq_validate_reads/report")),
        entries(["evidence.rs", "mod.rs", "rendering.rs", "science.rs"]),
        "FASTQ validation reporting must separate evidence, science, and rendering"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/run_suite")),
        entries(["mod.rs", "persistence.rs"]),
        "suite run tree must separate orchestration from artifact persistence"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/stage_report")),
        entries([
            "evidence.rs",
            "freshness.rs",
            "metrics.rs",
            "mod.rs",
            "publication.rs",
            "rendering.rs",
        ]),
        "stage report authority must separate admission, metrics, rendering, and publication"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/summary")),
        entries(["grouping.rs", "mod.rs", "row_metrics.rs", "strata.rs"]),
        "summary tree must separate grouping, row metrics, and stratum aggregation"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "benchmark_config.rs",
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "semantics/",
            "semantics.rs",
            "snapshots/",
            "vcf_local_planning.rs",
            "workspace_paths.rs",
        ]),
        "test tree must stay organized by enduring intent"
    );
}

#[test]
fn manifest_dependency_graph_matches_boundary_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let parsed: toml::Value = toml::from_str(&content)
        .unwrap_or_else(|err| panic!("parse {}: {err}", manifest.display()));

    let dependencies = dependency_names(&parsed, "dependencies");
    let dev_dependencies = dependency_names(&parsed, "dev-dependencies");
    let duplicates = dependencies.intersection(&dev_dependencies).cloned().collect::<Vec<_>>();

    assert!(
        duplicates.is_empty(),
        "normal and dev dependencies must not duplicate edges: {duplicates:?}",
    );
    for forbidden in [
        "bijux-dna-api",
        "bijux-dna-analyze",
        "bijux-dna-domain-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-runner",
        "fastrand",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "bijux-dna-bench must not carry normal dependency `{forbidden}`",
        );
    }
}

#[test]
fn benchmark_library_does_not_mutate_process_environment() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().is_none_or(|extension| extension != "rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
        for (index, line) in content.lines().enumerate() {
            if line.contains("std::env::set_var(") || line.contains("std::env::remove_var(") {
                offenders.push(format!("{}:{}", entry.path().display(), index + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "benchmark code and in-module tests must not mutate process-global environment: {offenders:?}",
    );
}

#[test]
fn commands_doc_lists_managed_benchmark_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs/COMMANDS.md");
    let content = std::fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    for command in [
        "load-suite",
        "summarize",
        "compare",
        "gate",
        "bench-data-dir",
        "bench-suites-dir",
        "plan-attempt-migration",
        "apply-attempt-migration",
        "verify-attempt-migration",
        "rollback-attempt-migration",
        "plan-stage-campaign-attempts",
        "verify-campaign-plan-source",
        "ensure-stage-campaign-attempts",
        "derive-stage-coverage",
        "publish-checked-in-stage-coverage",
        "publish-checked-in-stage-tool-closure",
        "derive-stage-universe",
        "publish-checked-in-stage-universe",
    ] {
        assert!(
            content.contains(command),
            "docs/COMMANDS.md must list `{command}` as a managed benchmark operation",
        );
    }

    for artifact in ["observations.jsonl", "summary.json", "decision.json", "decisions.json"] {
        assert!(
            content.contains(artifact),
            "docs/COMMANDS.md must list managed benchmark artifact `{artifact}`",
        );
    }
}

fn assert_docs_tree(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("docs")),
        entries([
            "ARCHITECTURE.md",
            "BENCH_CONTRACT.md",
            "BENCH_FORMAT.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "PUBLIC_API.md",
            "REPRODUCIBILITY.md",
            "SUITE_DESIGN.md",
            "TESTS.md",
        ]),
        "bench docs must stay at the 10-document allowance and live under docs/"
    );

    let misplaced_docs = markdown_files_outside_docs(root);
    assert!(
        misplaced_docs.is_empty(),
        "crate markdown outside docs/ must be limited to root README.md: {misplaced_docs:?}",
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}

fn dependency_names(parsed: &toml::Value, table_name: &str) -> BTreeSet<String> {
    parsed
        .get(table_name)
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

fn markdown_files_outside_docs(root: &Path) -> Vec<String> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        .filter(|entry| {
            let path = entry.path();
            path != root.join("README.md") && !path.starts_with(root.join("docs"))
        })
        .map(|entry| entry.path().display().to_string())
        .collect()
}
