use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use super::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
};
use crate::commands::benchmark::local_all_domain_fake_runs::{
    declared_output_ids, fake_run_all_domain_benchmark_results, AllDomainFakeRunResultReport,
};
use crate::commands::benchmark::local_stage_fake_runs::path_relative_to_repo;
use crate::commands::benchmark::local_stage_result_manifest::load_validated_stage_result_manifest_path;
use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH: &str =
    "benchmarks/readiness/missing-result-test-all-domains.json";
const DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT: &str =
    "runs/bench/readiness-probes/all-domains/missing-result-test";
const ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_missing_result_test.v1";

const FASTQ_REMOVED_RESULT_ID: &str =
    "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2";
const BAM_REMOVED_RESULT_ID: &str = "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools";
const VCF_REMOVED_RESULT_ID: &str = "vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AllDomainMissingResultStatus {
    Present,
    MissingResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainMissingResultRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) expected_manifest_path: String,
    pub(crate) audit_manifest_path: String,
    pub(crate) result_status: AllDomainMissingResultStatus,
    pub(crate) expected_output_artifact_ids: Vec<String>,
    pub(crate) observed_output_artifact_ids: Vec<String>,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) report_section: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainMissingResultTestReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fake_result_root: String,
    pub(crate) expected_row_count: usize,
    pub(crate) present_result_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) removed_result_ids: Vec<String>,
    pub(crate) removed_manifest_paths: Vec<String>,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainMissingResultRow>,
}

#[derive(Debug, Clone)]
struct AllDomainMissingResultFixture {
    fake_result_root: PathBuf,
    removed_result_ids: Vec<String>,
    removed_manifest_paths: Vec<PathBuf>,
}

pub(crate) fn run_render_all_domain_missing_result_test(
    args: &parse::BenchReadinessRenderAllDomainMissingResultTestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let report = render_all_domain_missing_result_test(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_readiness_root().join("missing-result-test-all-domains.json")
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_missing_result_test(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainMissingResultTestReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let fixture_root =
        benchmark_paths.benchmark_readiness_probe_root().join("all-domains/missing-result-test");
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &fixture_root,
        "all-domain missing-result fixture root",
    )?;
    if fixture_root.exists() {
        fs::remove_dir_all(&fixture_root)
            .with_context(|| format!("remove {}", fixture_root.display()))?;
    }

    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let fake_runs = fake_run_all_domain_benchmark_results(repo_root, fixture_root.clone())
        .with_context(|| {
            format!(
                "materialize all-domain missing-result fixture under {}",
                fixture_root.display()
            )
        })?;
    let output_rows = collect_all_domain_output_declaration_rows(repo_root)?;
    let fixture =
        seed_all_domain_missing_result_fixture(repo_root, &fake_runs.results, &expected_rows)?;
    let rows = collect_all_domain_missing_result_rows(
        repo_root,
        &fake_runs.results,
        &expected_rows,
        &output_rows,
    )?;

    let expected_row_count = rows.len();
    let present_result_row_count = rows
        .iter()
        .filter(|row| row.result_status == AllDomainMissingResultStatus::Present)
        .count();
    let missing_result_row_count = rows
        .iter()
        .filter(|row| row.result_status == AllDomainMissingResultStatus::MissingResult)
        .count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    let report = AllDomainMissingResultTestReport {
        schema_version: ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        fake_result_root: path_relative_to_repo(repo_root, &fixture.fake_result_root),
        expected_row_count,
        present_result_row_count,
        missing_result_row_count,
        passes_behavior_test: false,
        removed_result_ids: fixture.removed_result_ids.clone(),
        removed_manifest_paths: fixture
            .removed_manifest_paths
            .iter()
            .map(|path| path_relative_to_repo(repo_root, path))
            .collect(),
        domain_counts,
        rows,
    };
    let report = ensure_all_domain_missing_result_contract(report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn seed_all_domain_missing_result_fixture(
    repo_root: &Path,
    fake_runs: &[AllDomainFakeRunResultReport],
    expected_rows: &[AllDomainExpectedBenchmarkResultRow],
) -> Result<AllDomainMissingResultFixture> {
    let fake_runs_by_id =
        fake_runs.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let expected_by_id =
        expected_rows.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();

    let mut removed_manifest_paths = Vec::new();
    let mut removed_result_ids = Vec::new();
    for result_id in [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID] {
        let fake_run = fake_runs_by_id.get(result_id).copied().ok_or_else(|| {
            anyhow!(
                "all-domain missing-result fixture is missing fake-run coverage for `{result_id}`"
            )
        })?;
        let expected = expected_by_id.get(result_id).copied().ok_or_else(|| {
            anyhow!("all-domain missing-result fixture is missing expected-result coverage for `{result_id}`")
        })?;
        if fake_run.domain != expected.domain
            || fake_run.stage_id != expected.stage_id
            || fake_run.tool_id != expected.tool_id
            || fake_run.corpus_id != expected.corpus_id
            || fake_run.asset_profile_id != expected.asset_profile_id
        {
            return Err(anyhow!(
                "all-domain missing-result fixture drifted for `{result_id}` between expected rows and fake runs"
            ));
        }
        let manifest_path = repo_root.join(&fake_run.stage_result_path);
        fs::remove_file(&manifest_path)
            .with_context(|| format!("remove {}", manifest_path.display()))?;
        removed_manifest_paths.push(manifest_path);
        removed_result_ids.push(result_id.to_string());
    }

    Ok(AllDomainMissingResultFixture {
        fake_result_root: BenchmarkPathResolver::new(repo_root, None)
            .benchmark_readiness_probe_root()
            .join("all-domains/missing-result-test"),
        removed_result_ids,
        removed_manifest_paths,
    })
}

fn collect_all_domain_missing_result_rows(
    repo_root: &Path,
    fake_runs: &[AllDomainFakeRunResultReport],
    expected_rows: &[AllDomainExpectedBenchmarkResultRow],
    output_rows: &[AllDomainOutputDeclarationRow],
) -> Result<Vec<AllDomainMissingResultRow>> {
    let fake_runs_by_id =
        fake_runs.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let output_rows_by_id =
        output_rows.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();

    if fake_runs_by_id.len() != expected_rows.len() || output_rows_by_id.len() != expected_rows.len()
    {
        return Err(anyhow!(
            "all-domain missing-result test requires exact row-count alignment between expected results, output declarations, and fake runs"
        ));
    }

    let expected_ids =
        expected_rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    let fake_run_ids = fake_runs_by_id.keys().copied().collect::<BTreeSet<_>>();
    let output_row_ids = output_rows_by_id.keys().copied().collect::<BTreeSet<_>>();
    if expected_ids != fake_run_ids || expected_ids != output_row_ids {
        return Err(anyhow!(
            "all-domain missing-result test requires exact result_id alignment between expected rows, output declarations, and fake runs"
        ));
    }

    let mut rows = Vec::with_capacity(expected_rows.len());
    for expected in expected_rows {
        let fake_run =
            fake_runs_by_id.get(expected.result_id.as_str()).copied().ok_or_else(|| {
                anyhow!(
                    "all-domain missing-result test is missing fake-run coverage for `{}`",
                    expected.result_id
                )
            })?;
        let output_row = output_rows_by_id
            .get(expected.result_id.as_str())
            .copied()
            .ok_or_else(|| {
                anyhow!(
                    "all-domain missing-result test is missing output-declaration coverage for `{}`",
                    expected.result_id
                )
            })?;
        let expected_output_artifact_ids = declared_output_ids(output_row);
        let manifest_path = repo_root.join(&fake_run.stage_result_path);
        let manifest_label = fake_run.stage_result_path.clone();
        if !manifest_path.is_file() {
            rows.push(AllDomainMissingResultRow {
                result_id: expected.result_id.clone(),
                domain: expected.domain.clone(),
                stage_id: expected.stage_id.clone(),
                tool_id: expected.tool_id.clone(),
                corpus_id: expected.corpus_id.clone(),
                asset_profile_id: expected.asset_profile_id.clone(),
                expected_manifest_path: manifest_label.clone(),
                audit_manifest_path: manifest_label,
                result_status: AllDomainMissingResultStatus::MissingResult,
                expected_output_artifact_ids,
                observed_output_artifact_ids: Vec::new(),
                expected_metrics: expected.expected_metrics.clone(),
                report_section: expected.report_section.clone(),
                reason: format!(
                    "expected all-domain benchmark result `{}` remains visible even though its fake-run manifest is missing",
                    expected.result_id
                ),
            });
            continue;
        }

        let manifest = load_validated_stage_result_manifest_path(&manifest_path)
            .with_context(|| format!("load {}", manifest_path.display()))?;
        if manifest.stage_id != expected.stage_id || manifest.tool.id != expected.tool_id {
            return Err(anyhow!(
                "all-domain fake-run manifest `{}` drifted from `{}` / `{}`",
                manifest_path.display(),
                expected.stage_id,
                expected.tool_id
            ));
        }
        let observed_output_artifact_ids = manifest
            .outputs
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect::<Vec<_>>();
        let observed_output_set =
            observed_output_artifact_ids.iter().cloned().collect::<BTreeSet<_>>();
        let expected_output_set =
            expected_output_artifact_ids.iter().cloned().collect::<BTreeSet<_>>();
        if observed_output_set != expected_output_set {
            return Err(anyhow!(
                "all-domain fake-run manifest `{}` drifted from expected output ids for `{}`",
                manifest_path.display(),
                expected.result_id
            ));
        }

        rows.push(AllDomainMissingResultRow {
            result_id: expected.result_id.clone(),
            domain: expected.domain.clone(),
            stage_id: expected.stage_id.clone(),
            tool_id: expected.tool_id.clone(),
            corpus_id: expected.corpus_id.clone(),
            asset_profile_id: expected.asset_profile_id.clone(),
            expected_manifest_path: manifest_label.clone(),
            audit_manifest_path: manifest_label,
            result_status: AllDomainMissingResultStatus::Present,
            expected_output_artifact_ids,
            observed_output_artifact_ids,
            expected_metrics: expected.expected_metrics.clone(),
            report_section: expected.report_section.clone(),
            reason: format!(
                "expected all-domain benchmark result `{}` remains present with a validated fake-run manifest",
                expected.result_id
            ),
        });
    }

    Ok(rows)
}

fn ensure_all_domain_missing_result_contract(
    mut report: AllDomainMissingResultTestReport,
) -> Result<AllDomainMissingResultTestReport> {
    if report.rows.len() != report.expected_row_count {
        return Err(anyhow!(
            "all-domain missing-result rows must stay aligned with expected rows (rows={}, expected={})",
            report.rows.len(),
            report.expected_row_count
        ));
    }
    let expected_missing_row_count = report.removed_result_ids.len();
    let expected_present_row_count =
        report.expected_row_count.saturating_sub(expected_missing_row_count);
    if report.present_result_row_count != expected_present_row_count {
        return Err(anyhow!(
            "all-domain missing-result present rows must equal expected rows minus removed results (present={}, expected_present={})",
            report.present_result_row_count,
            expected_present_row_count
        ));
    }
    if report.missing_result_row_count != expected_missing_row_count {
        return Err(anyhow!(
            "all-domain missing-result rows must equal the removed-result count (missing={}, removed={})",
            report.missing_result_row_count,
            expected_missing_row_count
        ));
    }
    if report.removed_manifest_paths.len() != expected_missing_row_count {
        return Err(anyhow!(
            "all-domain missing-result test must record one removed manifest path per removed result id"
        ));
    }

    let missing_rows = report
        .rows
        .iter()
        .filter(|row| row.result_status == AllDomainMissingResultStatus::MissingResult)
        .collect::<Vec<_>>();
    let missing_ids =
        missing_rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    let expected_missing_ids =
        [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID]
            .into_iter()
            .collect::<BTreeSet<_>>();
    if missing_ids != expected_missing_ids {
        return Err(anyhow!(
            "all-domain missing-result test must keep the governed FASTQ, BAM, and VCF rows visible as missing_result"
        ));
    }

    let missing_domain_counts =
        missing_rows.iter().fold(BTreeMap::<&str, usize>::new(), |mut acc, row| {
            *acc.entry(row.domain.as_str()).or_default() += 1;
            acc
        });
    for domain in ["fastq", "bam", "vcf"] {
        if missing_domain_counts.get(domain).copied() != Some(1) {
            return Err(anyhow!(
                "all-domain missing-result test must emit exactly one missing_result row for `{domain}`"
            ));
        }
    }

    if report.rows.iter().any(|row| {
        row.result_status == AllDomainMissingResultStatus::Present
            && row.observed_output_artifact_ids.is_empty()
    }) {
        return Err(anyhow!(
            "all-domain missing-result test must not hide or strip observed outputs from present rows"
        ));
    }

    report.passes_behavior_test = true;
    Ok(report)
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::path::PathBuf;

    use super::{
        render_all_domain_missing_result_test, AllDomainMissingResultStatus,
        ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION, BAM_REMOVED_RESULT_ID,
        DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH, FASTQ_REMOVED_RESULT_ID,
        VCF_REMOVED_RESULT_ID,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn change_to(path: &std::path::Path) -> Self {
            let previous = env::current_dir().expect("current dir");
            env::set_current_dir(path).expect("set current dir");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous).expect("restore current dir");
        }
    }

    #[test]
    fn all_domain_missing_result_test_tracks_removed_rows_per_domain() {
        let root = repo_root();
        let _cwd_guard = CurrentDirGuard::change_to(&root);
        let report = render_all_domain_missing_result_test(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH),
        )
        .expect("render all-domain missing-result test");

        assert_eq!(report.schema_version, ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH);
        assert_eq!(
            report.fake_result_root,
            "runs/bench/readiness-probes/all-domains/missing-result-test"
        );
        assert_eq!(
            report.present_result_row_count + report.missing_result_row_count,
            report.expected_row_count
        );
        assert_eq!(report.missing_result_row_count, 3);
        assert!(report.passes_behavior_test);
        assert_eq!(report.domain_counts.get("fastq").copied(), Some(63));
        assert_eq!(report.domain_counts.get("bam").copied(), Some(49));
        assert_eq!(report.domain_counts.get("vcf").copied(), Some(16));

        let removed_ids =
            report.removed_result_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
        assert_eq!(
            removed_ids,
            [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID]
                .into_iter()
                .collect()
        );

        let missing_rows = report
            .rows
            .iter()
            .filter(|row| row.result_status == AllDomainMissingResultStatus::MissingResult)
            .collect::<Vec<_>>();
        assert_eq!(missing_rows.len(), 3);
        assert!(missing_rows.iter().all(|row| row.observed_output_artifact_ids.is_empty()));
        assert!(report.rows.iter().all(|row| {
            row.result_status == AllDomainMissingResultStatus::MissingResult
                || !row.observed_output_artifact_ids.is_empty()
        }));
    }
}
