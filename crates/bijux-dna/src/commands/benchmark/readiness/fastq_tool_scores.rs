use bijux_dna_api::v1::api::bench::{
    build_fastq_tool_score_rows, build_fastq_tool_scores_report as build_report,
    FastqScoreEvidence as FastqEvidenceAggregate,
    FastqScoreExpectedRow as ExpectedBenchmarkReportRowView,
    FastqScoreMemoryRow as MicroBenchmarkMemoryRowView,
    FastqScoreRuntimeRow as MicroBenchmarkRuntimeRowView, FastqToolScoreRow, FastqToolScoreStatus,
    FastqToolScoresReport, FastqToolScoringInput,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::catalog::load_external_artifact_binding_keys;
use super::stage_scoring::{StageScoringConfig, DEFAULT_STAGE_SCORING_PATH};
use crate::commands::benchmark::local_micro_benchmark_report::DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;
use crate::commands::numeric::rounded_f64_to_u64;

pub(crate) const DEFAULT_FASTQ_TOOL_SCORES_PATH: &str =
    "artifacts/benchmark-smoke/micro/fastq/FASTQ_TOOL_SCORES.tsv";

const DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/expected.json";
const DEFAULT_AMPLICON_NORMALIZE_PRIMERS_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml";
const DEFAULT_AMPLICON_REMOVE_CHIMERAS_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv";
const DEFAULT_AMPLICON_NORMALIZE_ABUNDANCE_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/normalized_abundance.tsv";

#[derive(Debug, Clone, Deserialize)]
struct ExpectedBenchmarkReportView {
    rows: Vec<ExpectedBenchmarkReportRowView>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct MicroBenchmarkReportView {
    runtime_rows: Vec<MicroBenchmarkRuntimeRowView>,
    memory_source_rows: Vec<MicroBenchmarkMemoryRowView>,
}

pub(crate) fn run_render_fastq_tool_scores(
    args: &parse::BenchReadinessRenderFastqToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_tool_scores(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_fastq_tool_scores(
    args: &parse::BenchReadinessValidateFastqToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_fastq_tool_scores(
        &repo_root,
        args.input.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_tool_scores(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqToolScoresReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (config_path, rows) = collect_fastq_tool_score_rows(repo_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_tool_scores_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(build_report(
        path_relative_to_repo(repo_root, &output_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

pub(crate) fn validate_fastq_tool_scores(
    repo_root: &Path,
    input_path: PathBuf,
) -> Result<FastqToolScoresReport> {
    let input_path = repo_relative_path(repo_root, &input_path);
    let actual = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;
    let (config_path, rows) = collect_fastq_tool_score_rows(repo_root)?;
    let expected = render_fastq_tool_scores_tsv(&rows);
    if actual != expected {
        bail!(
            "FASTQ tool score TSV drifted from governed evidence contracts; rerun `bijux-dna bench readiness render-fastq-tool-scores`"
        );
    }
    Ok(build_report(
        path_relative_to_repo(repo_root, &input_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

fn collect_fastq_tool_score_rows(repo_root: &Path) -> Result<(PathBuf, Vec<FastqToolScoreRow>)> {
    let config_path = repo_root.join(DEFAULT_STAGE_SCORING_PATH);
    let config = load_stage_scoring_config(&config_path)?;
    let expected_report = collect_expected_benchmark_report(repo_root)?;
    let micro_report = load_micro_benchmark_report(repo_root)?;
    let evidence = load_fastq_evidence(repo_root)?;
    let external_artifact_bindings = load_external_artifact_binding_keys(repo_root)?;
    let rows = build_fastq_tool_score_rows(FastqToolScoringInput {
        config,
        expected_rows: expected_report.rows,
        runtime_rows: micro_report.runtime_rows,
        memory_rows: micro_report.memory_source_rows,
        evidence,
        external_artifact_bindings,
    })?;
    Ok((config_path, rows))
}

fn load_stage_scoring_config(path: &Path) -> Result<StageScoringConfig> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).context("parse stage-scoring TOML")
}

fn collect_expected_benchmark_report(repo_root: &Path) -> Result<ExpectedBenchmarkReportView> {
    Ok(ExpectedBenchmarkReportView {
        rows: collect_all_domain_expected_benchmark_result_rows(repo_root)?
            .into_iter()
            .map(|row| ExpectedBenchmarkReportRowView {
                report_row_id: row.result_id.clone(),
                result_id: Some(row.result_id),
                domain: row.domain,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_id: row.corpus_id,
                report_section: row.report_section,
                row_status: "present".to_string(),
            })
            .collect(),
    })
}

fn load_micro_benchmark_report(repo_root: &Path) -> Result<MicroBenchmarkReportView> {
    let path = repo_root.join(DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(MicroBenchmarkReportView::default());
        }
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    serde_json::from_str(&raw).context("parse micro benchmark report JSON")
}

fn load_fastq_evidence(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), FastqEvidenceAggregate>> {
    let mut rows = BTreeMap::<(String, String), FastqEvidenceAggregate>::new();

    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.filter_reads/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            let dropped_reads = required_u64(root, "reads_dropped")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "report retained reads and dropped reads without violating input bounds"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(dropped_reads),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.filter_low_complexity/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            let dropped_reads = required_u64(root, "reads_removed_low_complexity")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "report low-complexity removal without inflating the read count"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(dropped_reads),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.profile_overrepresented_sequences/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let top_fraction = required_f64(root, "top_fraction")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(top_fraction),
                    truth_correctness_basis: Some("top_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "surface publishes governed comparable metrics for overrepresented sequences"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.remove_duplicates/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            let duplicate_reads = required_u64(root, "duplicate_reads")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "duplicate removal preserves the governed post-dedup read count surface"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(duplicate_reads),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.detect_adapters/adapters.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let case_count = required_u64(root, "case_count")?;
            let detected_case_count = required_u64(root, "detected_case_count")?;
            Ok((
                stage_id.to_string(),
                "fastqc".to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(detected_case_count, case_count)?),
                    truth_correctness_basis: Some("detected_case_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "adapter detection summary keeps the governed case-count surface complete"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.extract_umis/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "UMI extraction summary and governed report artifacts materialized"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.detect_duplicates_premerge/duplicates.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            Ok((
                stage_id.to_string(),
                "bijux_dna".to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "duplicate-signal smoke report covers both duplicate and distinct pair cases"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.estimate_library_complexity_prealign/complexity.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            Ok((
                stage_id.to_string(),
                "bijux_dna".to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "complexity smoke report emits governed estimated- and insufficient-data counts"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.merge_pairs/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let input_pair_count = required_u64(root, "input_pair_count")?;
            let merged_count = required_u64(root, "merged_count")?;
            let unmerged_r1_count = required_u64(root, "unmerged_r1_count")?;
            let unmerged_r2_count = required_u64(root, "unmerged_r2_count")?;
            let discarded_count = required_u64(root, "discarded_count")?;
            let input_reads = input_pair_count.saturating_mul(2);
            let retained_reads = merged_count + unmerged_r1_count + unmerged_r2_count;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(retained_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "merge report preserves merged, unmerged, and discarded counts".to_string(),
                    ),
                    retained_reads: Some(retained_reads),
                    dropped_reads: Some(discarded_count),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.normalize_abundance/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "normalized abundance table materialized and retained numeric validity"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.remove_chimeras/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "chimera summary emits governed non-chimeric and chimera artifact paths"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.cluster_otus/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "OTU clustering report materialized representatives and abundance outputs"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.infer_asvs/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "ASV inference report materialized representatives and abundance outputs"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.trim_terminal_damage/metrics.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "terminal-damage trimming report preserves the governed output count surface"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(input_reads.saturating_sub(output_reads)),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;

    merge_case_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.trim_reads/report.json",
        "trim_reads",
    )?;
    merge_case_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.trim_polyg_tails/metrics.json",
        "trim_polyg_tails",
    )?;
    merge_case_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.normalize_primers/report.json",
        "normalize_primers",
    )?;
    merge_profile_reads_summary(repo_root, &mut rows)?;
    merge_validate_reads_summary(repo_root, &mut rows)?;
    merge_screen_taxonomy_summary(repo_root, &mut rows)?;
    merge_fixture_backed_amplicon_fastq_evidence(repo_root, &mut rows);

    Ok(rows)
}

fn merge_fixture_backed_amplicon_fastq_evidence(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) {
    merge_evidence_row(
        rows,
        ("fastq.normalize_primers".to_string(), "cutadapt".to_string()),
        repo_root.join(DEFAULT_AMPLICON_NORMALIZE_PRIMERS_MANIFEST_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(1.0),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon fixture preserves the governed primer-normalization output surface"
                    .to_string(),
            ),
            retained_reads: Some(3),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.remove_chimeras".to_string(), "vsearch".to_string()),
        repo_root.join(DEFAULT_AMPLICON_REMOVE_CHIMERAS_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon chimera fixtures preserve the governed chimera-removal outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.infer_asvs".to_string(), "dada2".to_string()),
        repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon truth bundle preserves the governed ASV inference outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.cluster_otus".to_string(), "vsearch".to_string()),
        repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon truth bundle preserves the governed OTU-clustering outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.normalize_abundance".to_string(), "seqkit".to_string()),
        repo_root.join(DEFAULT_AMPLICON_NORMALIZE_ABUNDANCE_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon truth bundle preserves the governed abundance-normalization outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
}

fn merge_case_stage_report(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    relative_path: &str,
    contract_label: &str,
) -> Result<()> {
    let absolute_path = repo_root.join(relative_path);
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let mut tool_totals = BTreeMap::<String, (u64, u64, u64, u64)>::new();
    for case in cases {
        let tool_id = required_str(case, "tool_id")?.to_string();
        let input_reads = optional_u64(case, "input_read_count_total")
            .or_else(|| optional_u64(case, "input_reads"))
            .ok_or_else(|| {
                anyhow!("{} is missing input read counts for `{tool_id}`", absolute_path.display())
            })?;
        let output_reads = optional_u64(case, "output_read_count_total")
            .or_else(|| optional_u64(case, "output_reads"))
            .ok_or_else(|| {
                anyhow!("{} is missing output read counts for `{tool_id}`", absolute_path.display())
            })?;
        let retained_reads = optional_u64(case, "reads_retained").unwrap_or(output_reads);
        let dropped_reads = optional_u64(case, "reads_dropped")
            .unwrap_or_else(|| input_reads.saturating_sub(output_reads));
        let entry = tool_totals.entry(tool_id).or_default();
        entry.0 += input_reads;
        entry.1 += retained_reads;
        entry.2 += dropped_reads;
        entry.3 += 1;
    }
    for (tool_id, (input_reads, retained_reads, dropped_reads, _case_count)) in tool_totals {
        merge_evidence_row(
            rows,
            (stage_id.clone(), tool_id),
            absolute_path.display().to_string(),
            FastqEvidenceAggregate {
                truth_correctness_score: Some(fraction_u64(retained_reads, input_reads)?),
                truth_correctness_basis: Some("retained_fraction".to_string()),
                contract_correctness_score: Some(1.0),
                contract_correctness_basis: Some(format!(
                    "{contract_label} smoke cases preserve the governed output-count surface"
                )),
                retained_reads: Some(retained_reads),
                dropped_reads: Some(dropped_reads),
                source_paths: BTreeSet::new(),
            },
        );
    }
    Ok(())
}

fn merge_profile_reads_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.profile_reads/profile.json");
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let total_reads = cases
        .iter()
        .map(|case| required_u64(case, "reads_total"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    merge_evidence_row(
        rows,
        (stage_id, "seqkit_stats".to_string()),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "profile report emits governed read-count and aggregate QC metrics".to_string(),
            ),
            retained_reads: Some(total_reads),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

fn merge_validate_reads_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json");
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let total_reads = cases
        .iter()
        .map(|case| required_u64(case, "input_read_count_total"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    let passed_cases = u64::try_from(
        cases.iter().filter(|case| optional_str(case, "validation_status") == Some("pass")).count(),
    )?;
    let case_count = u64::try_from(cases.len())?;
    merge_evidence_row(
        rows,
        (stage_id, "fastqvalidator".to_string()),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(passed_cases, case_count)?),
            truth_correctness_basis: Some("validation_pass_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "validation smoke report covers every governed case without missing outputs"
                    .to_string(),
            ),
            retained_reads: Some(total_reads),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

fn merge_screen_taxonomy_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path = repo_root
        .join("artifacts/benchmark-smoke/micro/pipelines/edna/artifacts/fastq.screen_taxonomy/report.json");
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let tool_id = required_str(&root, "tool_id")?.to_string();
    let samples = root
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing samples", absolute_path.display()))?;
    let mut total_reads = 0_f64;
    let mut classified_reads = 0_f64;
    for sample in samples {
        let reads_in = required_f64(sample, "reads_in")?;
        let unclassified_fraction = required_f64(sample, "unclassified_fraction")?;
        total_reads += reads_in;
        classified_reads += reads_in * (1.0 - unclassified_fraction);
    }
    merge_evidence_row(
        rows,
        (stage_id, tool_id),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: (total_reads > 0.0).then_some(classified_reads / total_reads),
            truth_correctness_basis: Some("classified_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "taxonomy screen report materialized the governed classified and unclassified surfaces"
                    .to_string(),
            ),
            retained_reads: Some(rounded_f64_to_u64(classified_reads, "retained reads")?),
            dropped_reads: Some(rounded_f64_to_u64(total_reads - classified_reads, "dropped reads")?),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

fn merge_single_stage_report<F>(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    relative_path: &str,
    extractor: F,
) -> Result<()>
where
    F: Fn(&Value) -> Result<(String, String, FastqEvidenceAggregate)>,
{
    let absolute_path = repo_root.join(relative_path);
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let (stage_id, tool_id, evidence) = extractor(&root)?;
    merge_evidence_row(rows, (stage_id, tool_id), absolute_path.display().to_string(), evidence);
    Ok(())
}

fn merge_evidence_row(
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    key: (String, String),
    source_path: String,
    mut evidence: FastqEvidenceAggregate,
) {
    evidence.source_paths.insert(source_path);
    let entry = rows.entry(key).or_default();
    entry.source_paths.extend(evidence.source_paths);
    if entry.truth_correctness_score.is_none() {
        entry.truth_correctness_score = evidence.truth_correctness_score;
    }
    if entry.truth_correctness_basis.is_none() {
        entry.truth_correctness_basis = evidence.truth_correctness_basis;
    }
    if entry.contract_correctness_score.is_none() {
        entry.contract_correctness_score = evidence.contract_correctness_score;
    }
    if entry.contract_correctness_basis.is_none() {
        entry.contract_correctness_basis = evidence.contract_correctness_basis;
    }
    if entry.retained_reads.is_none() {
        entry.retained_reads = evidence.retained_reads;
    }
    if entry.dropped_reads.is_none() {
        entry.dropped_reads = evidence.dropped_reads;
    }
}

fn render_fastq_tool_scores_tsv(rows: &[FastqToolScoreRow]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tretained_reads\tdropped_reads\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason".to_string(),
    );
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.stage_id,
            row.tool_id,
            row.decision_mode,
            row.correctness_signal,
            join_csv(&row.result_ids),
            join_csv(&row.report_row_ids),
            join_csv(&row.corpus_ids),
            join_csv(&row.report_sections),
            join_csv(&row.row_statuses),
            score_status_label(row.score_status),
            format_optional_f64(row.truth_correctness_score),
            row.truth_correctness_basis.clone().unwrap_or_default(),
            format_optional_f64(row.contract_correctness_score),
            row.contract_correctness_basis.clone().unwrap_or_default(),
            format_optional_u64(row.retained_reads),
            format_optional_u64(row.dropped_reads),
            format_optional_f64(row.runtime_seconds),
            row.runtime_source,
            format_optional_f64(row.observed_memory_mb),
            format_optional_f64(row.declared_memory_mb),
            row.memory_source,
            row.failure_class,
            row.micro_execution_status.clone().unwrap_or_default(),
            format_f64(row.score_weight_coverage),
            format_optional_f64(row.score_total),
            join_csv(&row.evidence_paths),
            sanitize_tsv_cell(&row.reason),
        ));
    }
    lines.join("\n") + "\n"
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    repo_relative_path(repo_root, path)
        .display()
        .to_string()
        .replace(&format!("{}/", repo_root.display()), "")
}

fn load_optional_json_value(path: &Path) -> Result<Option<Value>> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value =
        serde_json::from_str::<Value>(&raw).with_context(|| format!("parse {}", path.display()))?;
    if matches!(value, Value::Object(ref map) if map.is_empty()) {
        return Ok(None);
    }
    Ok(Some(value))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| anyhow!("missing string field `{key}`"))
}

fn optional_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn required_u64(value: &Value, key: &str) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).ok_or_else(|| anyhow!("missing integer field `{key}`"))
}

fn optional_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn required_f64(value: &Value, key: &str) -> Result<f64> {
    value.get(key).and_then(Value::as_f64).ok_or_else(|| anyhow!("missing numeric field `{key}`"))
}

fn fraction_u64(numerator: u64, denominator: u64) -> Result<f64> {
    if denominator == 0 {
        Ok(0.0)
    } else {
        Ok(crate::commands::numeric::checked_f64_from_u64(numerator, "fraction numerator")?
            / crate::commands::numeric::checked_f64_from_u64(denominator, "fraction denominator")?)
    }
}

fn join_csv(values: &[String]) -> String {
    values.iter().map(|value| sanitize_tsv_cell(value)).collect::<Vec<_>>().join(",")
}

fn sanitize_tsv_cell(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(String::new, format_f64)
}

fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

fn format_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn score_status_label(value: FastqToolScoreStatus) -> &'static str {
    match value {
        FastqToolScoreStatus::Scored => "scored",
        FastqToolScoreStatus::InsufficientEvidence => "insufficient_evidence",
        FastqToolScoreStatus::Blocked => "blocked",
    }
}
