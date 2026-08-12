use std::fs;

#[test]
fn source_modules_have_owner() {
    let src_dir = crate::support::crate_root("bijux-dna-bench")
        .unwrap_or_else(|err| panic!("resolve benchmark crate root: {err}"))
        .join("src");
    let mut modules = Vec::new();
    collect_rs_files(&src_dir, &mut modules);

    let mut offenders = Vec::new();
    for module in modules {
        let Ok(contents) = fs::read_to_string(&module) else {
            continue;
        };
        let mut has_owner = false;
        for line in contents.lines().take(8) {
            if line.trim().starts_with("//!") && line.contains("Owner:") {
                has_owner = true;
                break;
            }
            if !line.trim().is_empty() && !line.trim().starts_with("//!") {
                break;
            }
        }
        if !has_owner {
            offenders.push(module.display().to_string());
        }
    }

    assert!(offenders.is_empty(), "missing module owner doc comments: {offenders:?}");
}

#[test]
fn thin_cli_delegates_stage_result_admission_to_benchmark_owners() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_result_manifest.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(
        contents.contains("validate_benchmark_stage_result_path"),
        "thin CLI must delegate file-backed admission to bijux-dna-bench"
    );
    assert!(
        contents.contains("validate_benchmark_stage_result_manifest"),
        "thin CLI must delegate manifest validation to bijux-dna-bench-model through the API"
    );
    assert!(
        !contents.contains("struct BenchStageResultValidationReport"),
        "thin CLI must not own the persisted validation report"
    );
    assert!(
        !contents.contains("bijux.bench.stage_result_validation.v1"),
        "thin CLI must not own the validation schema identity"
    );
}

#[test]
fn thin_cli_uses_model_owned_local_runtime_metrics_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_runtime_metrics.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_runtime_metrics"));
    assert!(!contents.contains("struct BenchLocalStageRuntimeMetricEntry"));
    assert!(!contents.contains("struct BenchLocalStageRuntimeMetricsReport"));
    assert!(!contents.contains("bijux.bench.local_runtime_metrics.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_benchmark_summary_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_benchmark_summary.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_summary"));
    assert!(!contents.contains("struct BenchLocalBenchmarkSummaryStage"));
    assert!(!contents.contains("struct BenchLocalBenchmarkSummaryReport"));
    assert!(!contents.contains("bijux.bench.local_benchmark_summary.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_smoke_suite_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_vcf_smoke_suite_ready.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_vcf_smoke_suite"));
    assert!(!contents.contains("struct LocalVcfSmokeSuiteGoalCheck"));
    assert!(!contents.contains("struct LocalVcfSmokeSuiteReadyReport"));
    assert!(!contents.contains("bijux.bench.local_vcf_smoke_suite_ready.v1"));
}

#[test]
fn thin_cli_uses_model_owned_all_domain_parser_collector_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/readiness/all_domain_parser_collector.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_all_domain_parser_collector"));
    assert!(!contents.contains("struct AllDomainParserCollectorRow"));
    assert!(!contents.contains("struct AllDomainParserCollectorReport"));
    assert!(!contents.contains("bijux.bench.readiness.all_domain_parser_collector.v1"));
}

#[test]
fn thin_cli_uses_model_owned_essential_pipeline_readiness_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/readiness/essential_pipelines_ready.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_essential_pipeline_readiness"));
    assert!(!contents.contains("struct EssentialPipelinesReadyGoalCheck"));
    assert!(!contents.contains("struct EssentialPipelinesReadyReport"));
    assert!(!contents.contains("bijux.bench.readiness.essential_pipelines_ready.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_stage_completion_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    for (file_name, validator, forbidden) in [
        (
            "local_stage_manifest_completion.rs",
            "validate_benchmark_local_stage_manifest_completion",
            "struct BenchLocalStageManifestCompletionReport",
        ),
        (
            "local_stage_output_completion.rs",
            "validate_benchmark_local_stage_output_completion",
            "struct BenchLocalStageOutputCompletionReport",
        ),
    ] {
        let cli_source = repo_root.join("crates/bijux-dna/src/commands/benchmark").join(file_name);
        let contents = fs::read_to_string(&cli_source)
            .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

        assert!(contents.contains(validator));
        assert!(!contents.contains(forbidden));
        assert!(!contents.contains("bijux.bench.local_stage_manifest_completion.v1"));
        assert!(!contents.contains("bijux.bench.local_stage_output_completion.v1"));
    }
}

#[test]
fn thin_cli_uses_model_owned_local_stage_inventory_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_inventory.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_stage_inventory"));
    assert!(contents.contains("validate_benchmark_local_all_domain_stage_inventory"));
    assert!(!contents.contains("struct BenchLocalStageInventory"));
    assert!(!contents.contains("struct BenchLocalAllDomainStageInventory"));
    assert!(!contents.contains("bijux.bench.local_stage_inventory.v1"));
    assert!(!contents.contains("bijux.bench.local_all_domain_stage_inventory.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_job_graph_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_job_graph.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_job_graph"));
    assert!(!contents.contains("struct LocalHpcJobGraphNode"));
    assert!(!contents.contains("struct LocalHpcJobGraph"));
    assert!(!contents.contains("bijux.bench.local_hpc_job_graph.v1"));
}

#[test]
fn thin_cli_uses_model_owned_fastq_corpus_run_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/corpus_fastq");
    let command = fs::read_to_string(cli_root.join("mod.rs")).expect("read corpus command");
    let local_models = fs::read_to_string(cli_root.join("models.rs")).expect("read CLI models");

    assert!(command.contains("validate_benchmark_local_corpus_run"));
    assert!(command.contains("BenchmarkCorpusRunManifest"));
    assert!(!local_models.contains("struct CorpusRunManifest"));
    assert!(!local_models.contains("struct SampleRunRecord"));
    assert!(!local_models.contains("bijux.fastq.corpus_benchmark_run.v1"));
}

#[test]
fn thin_cli_uses_model_owned_published_results_status_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command = fs::read_to_string(cli_root.join("results_status.rs"))
        .expect("read published-results status command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("validate_benchmark_published_results_status"));
    assert!(command.contains("PublishedResultsStatusReport"));
    assert!(!local_models.contains("struct PublishedResultsStatusReport"));
    assert!(!local_models.contains("struct PublishedResultsStageReport"));
    assert!(!local_models.contains("struct StageResultIssue"));
}

#[test]
fn thin_cli_uses_model_owned_dossier_index_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command =
        fs::read_to_string(cli_root.join("dossier_index.rs")).expect("read dossier-index command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("validate_benchmark_dossier_index"));
    assert!(command.contains("BenchmarkDossierIndex"));
    assert!(!local_models.contains("struct DossierIndex"));
    assert!(!local_models.contains("struct DossierStageEntry"));
}

#[test]
fn thin_cli_uses_model_owned_publication_status_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command = fs::read_to_string(cli_root.join("docs_status.rs"))
        .expect("read publication-status command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("BenchmarkPublicationStatusReport"));
    assert!(command.contains("validate_benchmark_publication_status"));
    assert!(!local_models.contains("struct BenchmarkPublicationStatusReport"));
    assert!(!local_models.contains("struct PublicationStageReport"));
    assert!(!local_models.contains("struct StageAuditIssue"));
}

#[test]
fn thin_cli_uses_model_owned_remediation_queue_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command =
        fs::read_to_string(cli_root.join("remediation.rs")).expect("read remediation command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("RemediationQueue"));
    assert!(command.contains("validate_benchmark_remediation_queue"));
    assert!(!local_models.contains("struct RemediationQueue"));
    assert!(!local_models.contains("struct RemediationStageEntry"));
    assert!(!local_models.contains("struct RemediationIssueGroup"));
    assert!(!local_models.contains("struct RemediationIssue"));
}

#[test]
fn thin_cli_uses_model_owned_publication_findings_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command =
        fs::read_to_string(cli_root.join("docs_status.rs")).expect("read findings command");

    assert!(command.contains("BenchmarkPublicationFindings"));
    assert!(command.contains("validate_benchmark_publication_findings"));
    assert!(!command.contains("payload.get(\"findings\")"));
}

#[test]
fn thin_cli_uses_model_owned_fastq_corpus_source_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let corpus_metadata = fs::read_to_string(
        repo_root.join("crates/bijux-dna/src/commands/benchmark/corpus_metadata.rs"),
    )
    .expect("read corpus metadata command");
    let publication_models = fs::read_to_string(
        repo_root.join("crates/bijux-dna/src/commands/benchmark/publication/models.rs"),
    )
    .expect("read publication CLI models");

    assert!(corpus_metadata.contains("load_fastq_corpus_source_spec"));
    assert!(corpus_metadata.contains("FastqCorpusSourceSpec as CorpusSpec"));
    assert!(!corpus_metadata.contains("struct CorpusSpec"));
    assert!(!corpus_metadata.contains("struct CorpusSpecSample"));
    assert!(!publication_models.contains("struct PublicationCorpusSpec"));
}

#[test]
fn thin_cli_uses_model_owned_corpus_dossier_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let dossier =
        fs::read_to_string(cli_root.join("corpus_dossier.rs")).expect("read corpus dossier");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(dossier.contains("validate_benchmark_corpus_dossier_summary"));
    assert!(dossier.contains("BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1"));
    for forbidden in [
        "struct CorpusSummary",
        "struct CorpusToolSummary",
        "struct CorpusHeadline",
        "struct CorpusSampleResultRow",
    ] {
        assert!(!local_models.contains(forbidden));
    }
    assert!(!dossier.contains("bijux.corpus_benchmark.summary.v1"));
}

#[test]
fn thin_cli_uses_model_owned_comparable_metric_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness");
    for (file_name, validator, forbidden_types, forbidden_schema) in [
        (
            "fastq_comparable_metrics.rs",
            "validate_fastq_comparable_metrics_report",
            ["struct FastqComparableMetricsRow", "struct FastqComparableMetricsReport"],
            "bijux.bench.readiness.fastq_comparable_metrics.v1",
        ),
        (
            "bam_comparable_metrics.rs",
            "validate_bam_comparable_metrics_report",
            ["struct BamComparableMetricsRow", "struct BamComparableMetricsReport"],
            "bijux.bench.readiness.bam_comparable_metrics.v1",
        ),
        (
            "vcf_comparable_metrics.rs",
            "validate_vcf_comparable_metrics_report",
            ["struct VcfComparableMetricsRow", "struct VcfComparableMetricsReport"],
            "bijux.bench.readiness.vcf_comparable_metrics.v1",
        ),
    ] {
        let source = fs::read_to_string(cli_root.join(file_name)).expect("read readiness command");
        assert!(source.contains(validator));
        for forbidden_type in forbidden_types {
            assert!(!source.contains(forbidden_type));
        }
        assert!(!source.contains(forbidden_schema));
        if file_name == "all_domain_active_stage_tool_matrix.rs" {
            for licensed_tool in ["\"angsd\"", "\"fastuniq\"", "\"impute5\"", "\"ngsbriggs\""] {
                assert!(!source.contains(licensed_tool));
            }
        }
    }
}

#[test]
fn thin_cli_uses_model_owned_readiness_view_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness");
    for (file_name, validator, forbidden_types, forbidden_schema) in [
        (
            "stage_centric_report.rs",
            "validate_stage_centric_report",
            [
                "struct StageCentricToolRow",
                "struct StageCentricStageReport",
                "struct StageCentricReport",
            ],
            "bijux.bench.readiness.stage_centric_report.v1",
        ),
        (
            "tool_centric_report.rs",
            "validate_tool_centric_report",
            [
                "struct ToolCentricStageRow",
                "struct ToolCentricToolReport",
                "struct ToolCentricReport",
            ],
            "bijux.bench.readiness.tool_centric_report.v1",
        ),
        (
            "corpus_centric_report.rs",
            "validate_corpus_centric_report",
            [
                "struct CorpusCentricStageRow",
                "struct CorpusCentricCorpusReport",
                "struct CorpusCentricReport",
            ],
            "bijux.bench.readiness.corpus_centric_report.v1",
        ),
    ] {
        let source = fs::read_to_string(cli_root.join(file_name)).expect("read readiness command");
        assert!(source.contains(validator));
        for forbidden_type in forbidden_types {
            assert!(!source.contains(forbidden_type));
        }
        assert!(!source.contains(forbidden_schema));
    }
}

#[test]
fn thin_cli_uses_model_owned_scope_view_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness");
    for (file_name, validator, forbidden_types, forbidden_schema) in [
        (
            "all_domain_active_stage_tool_matrix.rs",
            "validate_all_domain_active_stage_tool_matrix",
            [
                "struct AllDomainActiveStageToolMatrixRow",
                "struct AllDomainActiveStageToolMatrixReport",
            ],
            "bijux.bench.readiness.all_domain_active_stage_tool_matrix.v1",
        ),
        (
            "all_domain_retained_tools.rs",
            "validate_all_domain_retained_tools",
            ["struct AllDomainRetainedToolRow", "struct AllDomainRetainedToolsReport"],
            "bijux.bench.readiness.retained_tool_inventory.v1",
        ),
        (
            "stage_registry_extra_pairs.rs",
            "validate_stage_registry_extra_pairs",
            ["struct StageRegistryExtraPairRow", "struct StageRegistryExtraPairsReport"],
            "bijux.bench.readiness.stage_registry_extra_pairs.v1",
        ),
    ] {
        let source = fs::read_to_string(cli_root.join(file_name)).expect("read scope-view command");
        assert!(source.contains(validator));
        for forbidden_type in forbidden_types {
            assert!(!source.contains(forbidden_type));
        }
        assert!(!source.contains(forbidden_schema));
    }
}

fn collect_rs_files(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
