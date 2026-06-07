use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use crate::commands::benchmark::local_all_domain_fake_runs::{
    fake_run_all_domain_benchmark_results, AllDomainFakeRunResultReport,
};
use crate::commands::benchmark::local_stage_commands::materialize_local_stage;
use crate::commands::benchmark::local_stage_result_manifest::load_validated_stage_result_manifest_path;
use crate::commands::benchmark::local_vcf_stats_smoke::run_local_vcf_stats_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH: &str =
    "benchmarks/readiness/parser-collector-all-domains.json";
const DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT: &str =
    "benchmarks/readiness/parser-collector-all-domains-fixture";
const ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_parser_collector.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AllDomainParserCollectorSourceKind {
    FakeRun,
    RealSmoke,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainParserCollectorRow {
    pub(crate) record_id: String,
    pub(crate) source_kind: AllDomainParserCollectorSourceKind,
    pub(crate) document_kind: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) result_id: Option<String>,
    pub(crate) parsed_path: String,
    pub(crate) parsed_schema_version: String,
    pub(crate) parsed_top_level_key_count: usize,
    pub(crate) parsed_top_level_keys: Vec<String>,
    pub(crate) manifest_path: Option<String>,
    pub(crate) manifest_status: Option<String>,
    pub(crate) manifest_exit_code: Option<i32>,
    pub(crate) declared_output_count: usize,
    pub(crate) normalized_snapshot: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainParserCollectorReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fixture_root: String,
    pub(crate) fake_run_root: String,
    pub(crate) row_count: usize,
    pub(crate) fake_run_row_count: usize,
    pub(crate) real_smoke_row_count: usize,
    pub(crate) source_kind_counts: BTreeMap<String, usize>,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) document_kind_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainParserCollectorRow>,
}

pub(crate) fn run_render_all_domain_parser_collector(
    args: &parse::BenchReadinessRenderAllDomainParserCollectorArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_parser_collector(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_parser_collector(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainParserCollectorReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let fixture_root =
        repo_relative_path(repo_root, Path::new(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT));
    if fixture_root.exists() {
        fs::remove_dir_all(&fixture_root)
            .with_context(|| format!("remove {}", fixture_root.display()))?;
    }
    let fake_run_root = fixture_root.join("fake-runs");
    let fake_runs = fake_run_all_domain_benchmark_results(repo_root, fake_run_root.clone())
        .with_context(|| {
            format!("materialize all-domain fake runs under {}", fake_run_root.display())
        })?;

    let mut rows = collect_fake_run_rows(repo_root, &fake_runs.results)?;
    rows.extend(collect_real_smoke_rows(repo_root)?);
    rows.sort_by(|left, right| {
        source_kind_label(left.source_kind)
            .cmp(source_kind_label(right.source_kind))
            .then_with(|| left.domain.cmp(&right.domain))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.record_id.cmp(&right.record_id))
    });

    let fake_run_row_count = rows
        .iter()
        .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
        .count();
    let real_smoke_row_count = rows.len().saturating_sub(fake_run_row_count);
    let mut source_kind_counts = BTreeMap::<String, usize>::new();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut document_kind_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *source_kind_counts.entry(source_kind_label(row.source_kind).to_string()).or_default() += 1;
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *document_kind_counts.entry(row.document_kind.clone()).or_default() += 1;
    }

    let report = AllDomainParserCollectorReport {
        schema_version: ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        fixture_root: path_relative_to_repo(repo_root, &fixture_root),
        fake_run_root: path_relative_to_repo(repo_root, &fake_run_root),
        row_count: rows.len(),
        fake_run_row_count,
        real_smoke_row_count,
        source_kind_counts,
        domain_counts,
        document_kind_counts,
        rows,
    };
    ensure_all_domain_parser_collector_contract(repo_root, &report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn collect_fake_run_rows(
    repo_root: &Path,
    results: &[AllDomainFakeRunResultReport],
) -> Result<Vec<AllDomainParserCollectorRow>> {
    let mut rows = Vec::with_capacity(results.len());
    for result in results {
        let metrics_path = repo_root.join(&result.metrics_path);
        let metrics = read_json_document(&metrics_path)?;
        let metrics_schema_version = json_string_field(&metrics, "schema_version")?;
        let stage_result_path = repo_root.join(&result.stage_result_path);
        let stage_result = load_validated_stage_result_manifest_path(&stage_result_path)
            .with_context(|| format!("load {}", stage_result_path.display()))?;
        rows.push(AllDomainParserCollectorRow {
            record_id: format!("fake-run:{}", result.result_id),
            source_kind: AllDomainParserCollectorSourceKind::FakeRun,
            document_kind: "all_domain_fake_run_metrics".to_string(),
            domain: result.domain.clone(),
            stage_id: result.stage_id.clone(),
            tool_id: result.tool_id.clone(),
            corpus_id: result.corpus_id.clone(),
            asset_profile_id: result.asset_profile_id.clone(),
            result_id: Some(result.result_id.clone()),
            parsed_path: result.metrics_path.clone(),
            parsed_schema_version: metrics_schema_version,
            parsed_top_level_key_count: top_level_keys(&metrics).len(),
            parsed_top_level_keys: top_level_keys(&metrics),
            manifest_path: Some(result.stage_result_path.clone()),
            manifest_status: Some(manifest_status_label(&stage_result)),
            manifest_exit_code: Some(stage_result.runtime.exit_code),
            declared_output_count: result.declared_output_count,
            normalized_snapshot: BTreeMap::from([
                (
                    "command_step_count".to_string(),
                    Value::from(json_u64_field(&metrics, "command_step_count")?),
                ),
                (
                    "declared_output_count".to_string(),
                    Value::from(json_u64_field(&metrics, "declared_output_count")?),
                ),
                (
                    "expected_metric_count".to_string(),
                    Value::from(json_u64_field(&metrics, "expected_metric_count")?),
                ),
                (
                    "simulated_elapsed_seconds".to_string(),
                    Value::from(json_f64_field(&metrics, "simulated_elapsed_seconds")?),
                ),
            ]),
        });
    }
    Ok(rows)
}

fn collect_real_smoke_rows(repo_root: &Path) -> Result<Vec<AllDomainParserCollectorRow>> {
    let fastq_validate_path = materialize_local_stage(repo_root, "fastq.validate_reads")
        .context("materialize fastq.validate_reads real-smoke report")?;
    let bam_validate_path = materialize_local_stage(repo_root, "bam.validate")
        .context("materialize bam.validate real-smoke report")?;
    let vcf_stats_report = run_local_vcf_stats_smoke(repo_root, "bcftools")
        .context("materialize vcf.stats real-smoke report")?;

    Ok(vec![
        collect_fastq_validate_smoke_row(repo_root, &fastq_validate_path)?,
        collect_bam_validate_smoke_row(repo_root, &bam_validate_path)?,
        collect_vcf_stats_smoke_row(
            repo_root,
            &vcf_stats_report.metrics_path,
            &vcf_stats_report.stage_result_manifest_path,
        )?,
    ])
}

fn collect_fastq_validate_smoke_row(
    repo_root: &Path,
    report_path: &Path,
) -> Result<AllDomainParserCollectorRow> {
    let report = read_json_document(report_path)?;
    let case_count = json_u64_field(&report, "case_count")?;
    let all_cases_passed = json_bool_field(&report, "all_cases_passed")?;
    let missing_output_marker_present = json_bool_field(&report, "missing_output_marker_present")?;
    let parsed_top_level_keys = top_level_keys(&report);

    Ok(AllDomainParserCollectorRow {
        record_id: "real-smoke:fastq.validate_reads".to_string(),
        source_kind: AllDomainParserCollectorSourceKind::RealSmoke,
        document_kind: "fastq_local_smoke_report".to_string(),
        domain: "fastq".to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: "fastqc".to_string(),
        corpus_id: "local_smoke".to_string(),
        asset_profile_id: "sample_set".to_string(),
        result_id: None,
        parsed_path: path_relative_to_repo(repo_root, report_path),
        parsed_schema_version: json_string_field(&report, "schema_version")?,
        parsed_top_level_key_count: parsed_top_level_keys.len(),
        parsed_top_level_keys,
        manifest_path: None,
        manifest_status: None,
        manifest_exit_code: None,
        declared_output_count: case_artifact_count(
            &report,
            &["validation_report", "validated_reads_manifest"],
        ),
        normalized_snapshot: BTreeMap::from([
            ("case_count".to_string(), Value::from(case_count)),
            ("all_cases_passed".to_string(), Value::from(all_cases_passed)),
            (
                "missing_output_marker_present".to_string(),
                Value::from(missing_output_marker_present),
            ),
        ]),
    })
}

fn collect_bam_validate_smoke_row(
    repo_root: &Path,
    report_path: &Path,
) -> Result<AllDomainParserCollectorRow> {
    let report = read_json_document(report_path)?;
    let cases = report
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing `cases`", report_path.display()))?;
    let pass_case_count = cases
        .iter()
        .filter(|case| case.get("validation_status").and_then(Value::as_str) == Some("pass"))
        .count();
    let refusal_case_count = cases
        .iter()
        .filter(|case| case.get("validation_status").and_then(Value::as_str) == Some("refusal"))
        .count();
    let parsed_top_level_keys = top_level_keys(&report);

    Ok(AllDomainParserCollectorRow {
        record_id: "real-smoke:bam.validate".to_string(),
        source_kind: AllDomainParserCollectorSourceKind::RealSmoke,
        document_kind: "bam_local_smoke_report".to_string(),
        domain: "bam".to_string(),
        stage_id: "bam.validate".to_string(),
        tool_id: "samtools".to_string(),
        corpus_id: "local_smoke".to_string(),
        asset_profile_id: "sample_set".to_string(),
        result_id: None,
        parsed_path: path_relative_to_repo(repo_root, report_path),
        parsed_schema_version: json_string_field(&report, "schema_version")?,
        parsed_top_level_key_count: parsed_top_level_keys.len(),
        parsed_top_level_keys,
        manifest_path: None,
        manifest_status: None,
        manifest_exit_code: None,
        declared_output_count: case_artifact_count(
            &report,
            &["validation_report", "flagstat", "stage_metrics"],
        ),
        normalized_snapshot: BTreeMap::from([
            ("case_count".to_string(), Value::from(json_u64_field(&report, "case_count")?)),
            (
                "all_cases_matched".to_string(),
                Value::from(json_bool_field(&report, "all_cases_matched")?),
            ),
            ("pass_case_count".to_string(), Value::from(pass_case_count as u64)),
            ("refusal_case_count".to_string(), Value::from(refusal_case_count as u64)),
        ]),
    })
}

fn collect_vcf_stats_smoke_row(
    repo_root: &Path,
    metrics_relative_path: &str,
    stage_result_relative_path: &str,
) -> Result<AllDomainParserCollectorRow> {
    let metrics_path = repo_root.join(metrics_relative_path);
    let metrics = read_json_document(&metrics_path)?;
    let parsed_top_level_keys = top_level_keys(&metrics);
    let stage_result_path = repo_root.join(stage_result_relative_path);
    let stage_result = load_validated_stage_result_manifest_path(&stage_result_path)
        .with_context(|| format!("load {}", stage_result_path.display()))?;

    Ok(AllDomainParserCollectorRow {
        record_id: "real-smoke:vcf.stats".to_string(),
        source_kind: AllDomainParserCollectorSourceKind::RealSmoke,
        document_kind: "vcf_local_smoke_metrics".to_string(),
        domain: "vcf".to_string(),
        stage_id: "vcf.stats".to_string(),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        asset_profile_id: "vcf_cohort".to_string(),
        result_id: None,
        parsed_path: metrics_relative_path.to_string(),
        parsed_schema_version: json_string_field(&metrics, "schema_version")?,
        parsed_top_level_key_count: parsed_top_level_keys.len(),
        parsed_top_level_keys,
        manifest_path: Some(stage_result_relative_path.to_string()),
        manifest_status: Some(manifest_status_label(&stage_result)),
        manifest_exit_code: Some(stage_result.runtime.exit_code),
        declared_output_count: stage_result.outputs.len(),
        normalized_snapshot: BTreeMap::from([
            ("variant_count".to_string(), Value::from(json_u64_field(&metrics, "variant_count")?)),
            ("snp_count".to_string(), Value::from(json_u64_field(&metrics, "snp_count")?)),
            ("indel_count".to_string(), Value::from(json_u64_field(&metrics, "indel_count")?)),
            (
                "transition_count".to_string(),
                Value::from(json_u64_field(&metrics, "transition_count")?),
            ),
            (
                "transversion_count".to_string(),
                Value::from(json_u64_field(&metrics, "transversion_count")?),
            ),
            ("ti_tv".to_string(), Value::from(json_f64_field(&metrics, "ti_tv")?)),
            ("sample_count".to_string(), Value::from(json_u64_field(&metrics, "sample_count")?)),
        ]),
    })
}

fn ensure_all_domain_parser_collector_contract(
    repo_root: &Path,
    report: &AllDomainParserCollectorReport,
) -> Result<()> {
    if report.row_count == 0 {
        return Err(anyhow!("all-domain parser collector must produce at least one row"));
    }
    if report.fake_run_row_count == 0 || report.real_smoke_row_count == 0 {
        return Err(anyhow!(
            "all-domain parser collector must retain both fake_run and real_smoke evidence"
        ));
    }

    let expected_result_count = collect_all_domain_expected_benchmark_result_rows(repo_root)?.len();
    if report.fake_run_row_count != expected_result_count {
        return Err(anyhow!(
            "all-domain parser collector fake_run rows drifted from the canonical expected-result slice: expected {}, found {}",
            expected_result_count,
            report.fake_run_row_count
        ));
    }

    let fake_run_result_ids = report
        .rows
        .iter()
        .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
        .filter_map(|row| row.result_id.as_deref())
        .collect::<BTreeSet<_>>();
    if fake_run_result_ids.len() != expected_result_count {
        return Err(anyhow!(
            "all-domain parser collector must keep one fake_run row per canonical result_id"
        ));
    }

    let real_smoke_domains = report
        .rows
        .iter()
        .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::RealSmoke)
        .map(|row| row.domain.as_str())
        .collect::<BTreeSet<_>>();
    if real_smoke_domains != BTreeSet::from(["fastq", "bam", "vcf"]) {
        return Err(anyhow!(
            "all-domain parser collector must keep real_smoke evidence for fastq, bam, and vcf"
        ));
    }

    let mut seen_record_ids = BTreeSet::<&str>::new();
    for row in &report.rows {
        if row.record_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.parsed_path.trim().is_empty()
            || row.parsed_schema_version.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain parser collector row `{}` contains a blank required field",
                row.record_id
            ));
        }
        if !seen_record_ids.insert(row.record_id.as_str()) {
            return Err(anyhow!(
                "all-domain parser collector contains duplicate record_id `{}`",
                row.record_id
            ));
        }
        if row.parsed_top_level_key_count != row.parsed_top_level_keys.len() {
            return Err(anyhow!(
                "all-domain parser collector row `{}` drifted in top-level key accounting",
                row.record_id
            ));
        }
        if row.parsed_top_level_keys.is_empty() {
            return Err(anyhow!(
                "all-domain parser collector row `{}` must retain parsed top-level keys",
                row.record_id
            ));
        }
        if row.normalized_snapshot.is_empty() {
            return Err(anyhow!(
                "all-domain parser collector row `{}` must retain normalized snapshot fields",
                row.record_id
            ));
        }
    }

    Ok(())
}

fn read_json_document(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn top_level_keys(value: &Value) -> Vec<String> {
    let mut keys = value
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();
    keys
}

fn json_string_field(value: &Value, key: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("JSON document is missing string field `{key}`"))
}

fn json_u64_field(value: &Value, key: &str) -> Result<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("JSON document is missing u64 field `{key}`"))
}

fn json_f64_field(value: &Value, key: &str) -> Result<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("JSON document is missing f64 field `{key}`"))
}

fn json_bool_field(value: &Value, key: &str) -> Result<bool> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("JSON document is missing bool field `{key}`"))
}

fn case_artifact_count(report: &Value, artifact_keys: &[&str]) -> usize {
    report
        .get("cases")
        .and_then(Value::as_array)
        .map(|cases| {
            cases
                .iter()
                .map(|case| {
                    artifact_keys
                        .iter()
                        .filter(|artifact_key| {
                            case.get(**artifact_key).and_then(Value::as_str).is_some()
                        })
                        .count()
                })
                .sum()
        })
        .unwrap_or(0)
}

fn manifest_status_label(
    manifest: &crate::commands::benchmark::local_stage_result_manifest::BenchStageResultManifestV1,
) -> String {
    match manifest.runtime.status {
        crate::commands::benchmark::local_stage_result_manifest::BenchStageResultStatus::Succeeded => {
            "succeeded".to_string()
        }
        crate::commands::benchmark::local_stage_result_manifest::BenchStageResultStatus::Failed => {
            "failed".to_string()
        }
    }
}

fn source_kind_label(value: AllDomainParserCollectorSourceKind) -> &'static str {
    match value {
        AllDomainParserCollectorSourceKind::FakeRun => "fake_run",
        AllDomainParserCollectorSourceKind::RealSmoke => "real_smoke",
    }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};

    use serde_json::Value;

    use super::{
        render_all_domain_parser_collector, ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_VERSION,
        DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
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
        fn change_to(path: &Path) -> Self {
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
    fn render_all_domain_parser_collector_reports_fake_and_real_smoke_rows() {
        let root = repo_root();
        let _cwd_guard = CurrentDirGuard::change_to(&root);
        let report = render_all_domain_parser_collector(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
        )
        .expect("render all-domain parser collector");

        assert_eq!(report.schema_version, ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH);
        assert_eq!(
            report.fixture_root,
            "benchmarks/readiness/parser-collector-all-domains-fixture"
        );
        assert_eq!(
            report.fake_run_root,
            "benchmarks/readiness/parser-collector-all-domains-fixture/fake-runs"
        );
        assert_eq!(report.row_count, 123);
        assert_eq!(report.fake_run_row_count, 120);
        assert_eq!(report.real_smoke_row_count, 3);
        assert_eq!(report.source_kind_counts.get("fake_run"), Some(&120));
        assert_eq!(report.source_kind_counts.get("real_smoke"), Some(&3));
        assert_eq!(report.domain_counts.get("fastq"), Some(&64));
        assert_eq!(report.domain_counts.get("bam"), Some(&50));
        assert_eq!(report.domain_counts.get("vcf"), Some(&9));

        let fastq_smoke = report
            .rows
            .iter()
            .find(|row| row.record_id == "real-smoke:fastq.validate_reads")
            .expect("fastq smoke row");
        assert_eq!(fastq_smoke.document_kind, "fastq_local_smoke_report");
        assert_eq!(fastq_smoke.parsed_schema_version, "bijux.fastq.validate.local_smoke.report.v1");
        assert_eq!(fastq_smoke.normalized_snapshot.get("case_count"), Some(&Value::from(2_u64)));
        assert_eq!(
            fastq_smoke.normalized_snapshot.get("all_cases_passed"),
            Some(&Value::from(true))
        );

        let bam_smoke = report
            .rows
            .iter()
            .find(|row| row.record_id == "real-smoke:bam.validate")
            .expect("bam smoke row");
        assert_eq!(bam_smoke.document_kind, "bam_local_smoke_report");
        assert_eq!(bam_smoke.parsed_schema_version, "bijux.bam.validate.local_smoke.report.v1");
        assert_eq!(bam_smoke.normalized_snapshot.get("pass_case_count"), Some(&Value::from(1_u64)));
        assert_eq!(
            bam_smoke.normalized_snapshot.get("refusal_case_count"),
            Some(&Value::from(1_u64))
        );

        let vcf_smoke = report
            .rows
            .iter()
            .find(|row| row.record_id == "real-smoke:vcf.stats")
            .expect("vcf smoke row");
        assert_eq!(vcf_smoke.document_kind, "vcf_local_smoke_metrics");
        assert_eq!(vcf_smoke.parsed_schema_version, "bijux.bench.local_vcf_stats_smoke.metrics.v1");
        assert_eq!(vcf_smoke.normalized_snapshot.get("variant_count"), Some(&Value::from(4_u64)));
        assert_eq!(vcf_smoke.normalized_snapshot.get("ti_tv"), Some(&Value::from(2.0_f64)));
        assert_eq!(vcf_smoke.manifest_status.as_deref(), Some("succeeded"));
        assert_eq!(vcf_smoke.manifest_exit_code, Some(0));

        let fake_vcf = report
            .rows
            .iter()
            .find(|row| {
                row.result_id.as_deref()
                    == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
            })
            .expect("fake vcf row");
        assert_eq!(fake_vcf.document_kind, "all_domain_fake_run_metrics");
        assert_eq!(
            fake_vcf.parsed_schema_version,
            "bijux.bench.local_all_domain_fake_run_metrics.v1"
        );
        assert_eq!(fake_vcf.manifest_status.as_deref(), Some("succeeded"));
        assert_eq!(
            fake_vcf.normalized_snapshot.get("declared_output_count"),
            Some(&Value::from(2_u64))
        );
    }
}
