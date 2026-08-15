use std::collections::BTreeMap;
use std::path::Path;

use bijux_dna_bench::{
    normalize_benchmark_config, parse_benchmark_config, validate_benchmark_config_contract,
    BenchmarkConfig, BenchmarkConfigValidationRequest, BenchmarkCorpusConfig,
    BenchmarkCorpusPublicationConfig, BenchmarkDepleteRrnaInputConfig, BenchmarkPublicationConfig,
    BenchmarkReferenceInputConfig, BenchmarkScreenTaxonomyInputConfig, BenchmarkStageInputConfig,
    BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout, BenchmarkWorkspaceLocal,
    BenchmarkWorkspaceRemote, BenchmarkWorkspaceStageRuns, CorpusBenchmarkContract,
};

fn write_file(path: &Path) {
    std::fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
    std::fs::write(path, b"fixture").expect("write fixture");
}

fn valid_config(root: &Path) -> BenchmarkConfig {
    for path in [
        "benchmarks/corpora/corpus-01.toml",
        "inputs/rrna/rrna.db",
        "inputs/reference/host/index",
        "inputs/reference/contaminants/index",
        "inputs/taxonomy/database/nodes.dmp",
    ] {
        write_file(&root.join(path));
    }

    BenchmarkConfig {
        workspace: BenchmarkWorkspaceConfig {
            local: Some(BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/results".to_string()),
                cache_mirror_root: Some("/bench/local/cache".to_string()),
                extra_data_root: Some("/bench/local/extra".to_string()),
                reference_root: None,
            }),
            remote: Some(BenchmarkWorkspaceRemote {
                ssh_host: Some("cluster".to_string()),
                repo_root: Some("/bench/remote/repo".to_string()),
                cache_root: Some("/bench/remote/cache".to_string()),
                corpus_root: Some("/bench/remote/corpora".to_string()),
                results_root: Some("/bench/remote/results".to_string()),
                extra_data_root: Some("/bench/remote/extra".to_string()),
                ..BenchmarkWorkspaceRemote::default()
            }),
            layout: Some(BenchmarkWorkspaceLayout {
                stage_runs: Some(BenchmarkWorkspaceStageRuns {
                    remote_results_template: Some("{corpus_id}/{stage_id}/cluster".to_string()),
                    local_cache_results_template: Some(
                        "results/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    local_archive_results_template: Some(
                        "{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                }),
            }),
            ..BenchmarkWorkspaceConfig::default()
        },
        publication: BenchmarkPublicationConfig {
            corpora: BTreeMap::from([(
                "corpus_01".to_string(),
                BenchmarkCorpusPublicationConfig {
                    contracts: vec![CorpusBenchmarkContract {
                        stage_id: "fastq.validate_reads".to_string(),
                        scenario_id: "validation_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        tools: vec!["fastqvalidator".to_string()],
                    }],
                    exclusions: Vec::new(),
                },
            )]),
        },
        corpora: BTreeMap::from([(
            "corpus-01".to_string(),
            BenchmarkCorpusConfig {
                spec_path: Some("benchmarks/corpora/corpus-01.toml".to_string()),
            },
        )]),
        stage_inputs: BenchmarkStageInputConfig {
            fastq_deplete_rrna: BenchmarkDepleteRrnaInputConfig {
                rrna_db: Some("inputs/rrna/rrna.db".to_string()),
                ..BenchmarkDepleteRrnaInputConfig::default()
            },
            fastq_deplete_host: BenchmarkReferenceInputConfig {
                reference_index: Some("inputs/reference/host/index".to_string()),
                ..BenchmarkReferenceInputConfig::default()
            },
            fastq_deplete_reference_contaminants: BenchmarkReferenceInputConfig {
                reference_index: Some("inputs/reference/contaminants/index".to_string()),
                ..BenchmarkReferenceInputConfig::default()
            },
            fastq_screen_taxonomy: BenchmarkScreenTaxonomyInputConfig {
                database_root: Some("inputs/taxonomy/database".to_string()),
                ..BenchmarkScreenTaxonomyInputConfig::default()
            },
        },
    }
}

fn validate(
    root: &Path,
    config: &BenchmarkConfig,
    check_paths: bool,
) -> anyhow::Result<bijux_dna_bench::BenchmarkConfigValidationSummary> {
    let config_path = root.join("configs/bench/benchmark.toml");
    validate_benchmark_config_contract(&BenchmarkConfigValidationRequest {
        cwd: root,
        config_path: &config_path,
        config,
        check_paths,
    })
}

#[test]
fn normalizes_optional_configuration_strings() {
    let mut config = BenchmarkConfig::default();
    config.workspace.local = Some(BenchmarkWorkspaceLocal {
        results_root: Some("  /bench/results  ".to_string()),
        cache_mirror_root: Some("  ".to_string()),
        ..BenchmarkWorkspaceLocal::default()
    });
    config.corpora.insert(
        "corpus-01".to_string(),
        BenchmarkCorpusConfig {
            spec_path: Some("  benchmarks/corpora/corpus-01.toml  ".to_string()),
        },
    );

    let normalized = normalize_benchmark_config(config);
    let local = normalized.workspace.local.expect("local workspace");
    assert_eq!(local.results_root.as_deref(), Some("/bench/results"));
    assert_eq!(local.cache_mirror_root, None);
    assert_eq!(
        normalized.corpora["corpus-01"].spec_path.as_deref(),
        Some("benchmarks/corpora/corpus-01.toml")
    );
}

#[test]
fn parses_and_normalizes_benchmark_configuration_text() {
    let config = parse_benchmark_config(
        r#"[workspace.local]
results_root = "  /bench/results  "
"#,
    )
    .expect("parse benchmark config");

    assert_eq!(
        config.workspace.local.and_then(|row| row.results_root),
        Some("/bench/results".to_string())
    );
}

#[test]
fn accepts_complete_configuration_and_existing_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let summary = validate(temp.path(), &valid_config(temp.path()), true).expect("valid config");
    assert_eq!(summary.corpus_count, 1);
    assert!(summary.paths_checked);
}

#[test]
fn requires_a_declared_corpus() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = valid_config(temp.path());
    config.corpora.clear();
    let error = validate(temp.path(), &config, false).expect_err("missing corpus must fail");
    assert!(error.to_string().contains("at least one corpus"));
}

#[test]
fn requires_publication_contracts_for_each_corpus() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = valid_config(temp.path());
    config.publication.corpora.clear();
    let error = validate(temp.path(), &config, false).expect_err("missing publication must fail");
    assert!(error.to_string().contains("publication.corpus_01.contracts is empty"));
}

#[test]
fn requires_a_corpus_spec_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = valid_config(temp.path());
    config.corpora.get_mut("corpus-01").expect("corpus").spec_path = None;
    let error = validate(temp.path(), &config, false).expect_err("missing spec must fail");
    assert!(error.to_string().contains("corpora.corpus-01.spec_path"));
}

#[test]
fn reports_all_missing_workspace_contracts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = valid_config(temp.path());
    config.workspace.local.as_mut().expect("local").cache_mirror_root = None;
    config.workspace.remote.as_mut().expect("remote").cache_root = None;
    config
        .workspace
        .layout
        .as_mut()
        .expect("layout")
        .stage_runs
        .as_mut()
        .expect("stage runs")
        .remote_results_template = None;
    let rendered = validate(temp.path(), &config, false)
        .expect_err("missing workspace fields must fail")
        .to_string();
    assert!(rendered.contains("workspace.local.cache_mirror_root"));
    assert!(rendered.contains("workspace.remote.cache_root"));
    assert!(rendered.contains("workspace.layout.stage_runs.remote_results_template"));
}

#[test]
fn rejects_publication_corpora_without_a_declaration() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = valid_config(temp.path());
    let publication = config.publication.corpora["corpus_01"].clone();
    config.publication.corpora.insert("study_42".to_string(), publication);
    let error = validate(temp.path(), &config, false).expect_err("undeclared corpus must fail");
    assert!(error.to_string().contains("publication.study_42 does not match"));
}

#[test]
fn rejects_missing_stage_input_paths_when_requested() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = valid_config(temp.path());
    config.stage_inputs.fastq_deplete_host.reference_index =
        Some("inputs/reference/missing/index".to_string());
    let error = validate(temp.path(), &config, true).expect_err("missing stage input must fail");
    assert!(error.to_string().contains("stage_inputs.fastq_deplete_host.reference_index"));
}
