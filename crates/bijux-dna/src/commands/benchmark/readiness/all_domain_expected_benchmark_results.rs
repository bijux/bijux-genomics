use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    validate_all_domain_expected_benchmark_results, validate_all_domain_expected_result_rows,
    ReadinessStageToolBinding as BindingKey, ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    AllDomainExpectedBenchmarkResultRow, AllDomainExpectedBenchmarkResultsReport,
};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::stage_comparable_metric_fields_for_stage;
use bijux_dna_domain_fastq::stage_sanity_metrics_for_stage;

use super::bam_report_map::collect_bam_report_map_rows;
use super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use super::fastq_report_map::collect_fastq_report_stage_metadata;
use super::stage_tool_assets::{
    validate_stage_tool_assets_config, StageToolAssetRow, StageToolAssetsConfig,
    DEFAULT_STAGE_TOOL_ASSETS_PATH,
};
use super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/expected-benchmark-results-all-domains.tsv";

pub(crate) fn run_render_all_domain_expected_benchmark_results(
    args: &parse::BenchReadinessRenderAllDomainExpectedBenchmarkResultsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_expected_benchmark_results(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_expected_benchmark_results(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainExpectedBenchmarkResultsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_expected_benchmark_results_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let row_count = rows.len();
    let result_id_count =
        rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let corpus_count = rows.iter().map(|row| row.corpus_id.as_str()).collect::<BTreeSet<_>>().len();
    let asset_profile_count =
        rows.iter().map(|row| row.asset_profile_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
    }

    let report = AllDomainExpectedBenchmarkResultsReport {
        schema_version: ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1.to_string(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        corpus_count,
        asset_profile_count,
        domain_counts,
        report_section_counts,
        rows,
    };
    validate_all_domain_expected_benchmark_results(&report)?;
    Ok(report)
}

pub(crate) fn collect_all_domain_expected_benchmark_result_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainExpectedBenchmarkResultRow>> {
    let asset_roles_by_binding = load_committed_stage_tool_asset_rows(repo_root)?.into_iter().fold(
        BTreeMap::<BindingKey, Vec<String>>::new(),
        |mut acc, row| {
            acc.entry(binding_key(&row.domain, &row.stage_id, &row.tool_id))
                .or_default()
                .push(row.asset_role);
            acc
        },
    );

    let fastq_report_sections = collect_fastq_report_stage_metadata(repo_root)?
        .into_iter()
        .map(|(stage_id, row)| (stage_id, row.report_section_id))
        .collect::<BTreeMap<_, _>>();
    let bam_report_sections = collect_bam_report_map_rows(repo_root)?
        .into_iter()
        .map(|row| (row.stage_id, row.report_section_id))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();

    let expected_fastq_bam = collect_expected_benchmark_result_rows(repo_root)?;
    let expected_fastq_bam_count = expected_fastq_bam.len();
    for row in expected_fastq_bam {
        let key = binding_key(&row.domain, &row.stage_id, &row.tool_id);
        let asset_profile_id = fastq_bam_asset_profile_id(&key, &asset_roles_by_binding);
        let report_section = match row.domain.as_str() {
            "fastq" => fastq_report_sections.get(&row.stage_id).cloned().ok_or_else(|| {
                anyhow!(
                    "all-domain expected-result table is missing FASTQ report section for `{}`",
                    row.stage_id
                )
            })?,
            "bam" => bam_report_sections.get(&row.stage_id).cloned().ok_or_else(|| {
                anyhow!(
                    "all-domain expected-result table is missing BAM report section for `{}`",
                    row.stage_id
                )
            })?,
            other => {
                return Err(anyhow!(
                    "all-domain expected-result table does not support legacy domain `{other}`"
                ))
            }
        };
        let expected_metrics = match row.domain.as_str() {
            "fastq" => {
                expected_fastq_metrics(&row.stage_id, row.normalized_metrics_output_id.as_deref())
            }
            "bam" => {
                expected_bam_metrics(&row.stage_id, row.normalized_metrics_output_id.as_deref())
            }
            _ => Vec::new(),
        };

        rows.push(AllDomainExpectedBenchmarkResultRow {
            result_id: row.result_row_id,
            domain: row.domain,
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            corpus_id: row.fixture_id,
            asset_profile_id,
            expected_outputs: row.expected_output_artifact_ids,
            expected_metrics,
            report_section,
        });
    }

    let expected_vcf = collect_vcf_expected_benchmark_result_rows(repo_root)?;
    let expected_vcf_count = expected_vcf.len();
    for row in expected_vcf {
        rows.push(AllDomainExpectedBenchmarkResultRow {
            result_id: vcf_result_id(&row),
            domain: row.domain,
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            corpus_id: row.corpus_id,
            asset_profile_id: row.asset_profile_id,
            expected_outputs: row.expected_outputs,
            expected_metrics: row.expected_metrics,
            report_section: row.report_section,
        });
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    validate_all_domain_expected_result_rows(&rows, expected_fastq_bam_count, expected_vcf_count)?;
    Ok(rows)
}

fn expected_fastq_metrics(
    stage_id: &str,
    normalized_metrics_output_id: Option<&str>,
) -> Vec<String> {
    let stage_id = StageId::new(stage_id.to_string());
    let mut metrics = BTreeSet::<String>::new();
    if let Some(output_id) = normalized_metrics_output_id {
        metrics.insert(output_id.to_string());
    }
    for metric in stage_sanity_metrics_for_stage(&stage_id) {
        metrics.insert(metric);
    }
    metrics.into_iter().collect()
}

fn expected_bam_metrics(stage_id: &str, normalized_metrics_output_id: Option<&str>) -> Vec<String> {
    let stage_id = StageId::new(stage_id.to_string());
    let mut metrics = BTreeSet::<String>::new();
    if let Some(output_id) = normalized_metrics_output_id {
        metrics.insert(output_id.to_string());
    }
    for metric in stage_comparable_metric_fields_for_stage(&stage_id) {
        metrics.insert(metric);
    }
    metrics.into_iter().collect()
}

fn render_all_domain_expected_benchmark_results_tsv(
    rows: &[AllDomainExpectedBenchmarkResultRow],
) -> String {
    let mut rendered = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\texpected_outputs\texpected_metrics\treport_section\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.expected_outputs.join(",")),
            sanitize_tsv(&row.expected_metrics.join(",")),
            sanitize_tsv(&row.report_section),
        ));
    }
    rendered
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

fn fastq_bam_asset_profile_id(
    key: &BindingKey,
    asset_roles_by_binding: &BTreeMap<BindingKey, Vec<String>>,
) -> String {
    let Some(asset_roles) = asset_roles_by_binding.get(key) else {
        return "corpus_only".to_string();
    };
    let mut roles = asset_roles.clone();
    roles.sort();
    roles.dedup();
    if roles.is_empty() {
        "corpus_only".to_string()
    } else {
        roles.join("+")
    }
}

fn load_committed_stage_tool_asset_rows(repo_root: &Path) -> Result<Vec<StageToolAssetRow>> {
    let asset_path = repo_root.join(DEFAULT_STAGE_TOOL_ASSETS_PATH);
    let rendered = fs::read_to_string(&asset_path)
        .with_context(|| format!("read {}", asset_path.display()))?;
    let report: StageToolAssetsConfig =
        toml::from_str(&rendered).with_context(|| format!("parse {}", asset_path.display()))?;
    validate_stage_tool_assets_config(&report)?;
    Ok(report.rows)
}

fn vcf_result_id(
    row: &super::vcf_expected_benchmark_results::VcfExpectedBenchmarkResultRow,
) -> String {
    crate::commands::benchmark::benchmark_result_ids::build_asset_profile_benchmark_result_id(
        &row.domain,
        &row.corpus_id,
        &row.stage_id,
        &row.asset_profile_id,
        &row.tool_id,
    )
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(repo_root) {
        return relative.to_string_lossy().replace('\\', "/");
    }
    let components = path.components().collect::<Vec<_>>();
    if let Some(start) = components.iter().position(
        |component| matches!(component, std::path::Component::Normal(name) if *name == "artifacts"),
    ) {
        return components[start..]
            .iter()
            .collect::<PathBuf>()
            .to_string_lossy()
            .replace('\\', "/");
    }
    path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned()
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{
        path_relative_to_repo, render_all_domain_expected_benchmark_results,
        ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1,
        DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_all_domain_expected_benchmark_results_tracks_governed_rows() {
        let root = repo_root();
        let report = render_all_domain_expected_benchmark_results(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
        )
        .expect("render all-domain expected benchmark results");

        assert_eq!(report.schema_version, ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH);
        assert_eq!(report.result_id_count, report.row_count);
        assert_eq!(
            report.stage_count,
            report.rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len()
        );
        assert_eq!(
            report.tool_count,
            report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len()
        );
        assert_eq!(
            report.corpus_count,
            report.rows.iter().map(|row| row.corpus_id.as_str()).collect::<BTreeSet<_>>().len()
        );
        assert_eq!(
            report.asset_profile_count,
            report
                .rows
                .iter()
                .map(|row| row.asset_profile_id.as_str())
                .collect::<BTreeSet<_>>()
                .len()
        );
        for domain in ["fastq", "bam", "vcf"] {
            let observed = report.rows.iter().filter(|row| row.domain == domain).count();
            assert_eq!(report.domain_counts.get(domain), Some(&observed));
        }

        let taxonomy = report
            .rows
            .iter()
            .find(|row| {
                row.result_id
                    == "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2"
            })
            .expect("taxonomy result row");
        assert_eq!(taxonomy.asset_profile_id, "database_artifact_id+taxonomy_database_root");
        assert_eq!(taxonomy.report_section, "contamination_screening");
        assert!(taxonomy
            .expected_outputs
            .iter()
            .any(|value| value == "classification_report_json"));
        assert!(taxonomy
            .expected_metrics
            .iter()
            .any(|value| value == "classification_report_json"));

        let kinship = report
            .rows
            .iter()
            .find(|row| row.result_id == "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
            .expect("kinship result row");
        assert_eq!(kinship.asset_profile_id, "reference_fasta+reference_panel");
        assert_eq!(kinship.report_section, "sample_identity");
        assert!(kinship.expected_outputs.iter().any(|value| value == "kinship_report"));
        assert!(kinship.expected_metrics.iter().any(|value| value == "kinship_report"));

        let vcf_call = report
            .rows
            .iter()
            .find(|row| {
                row.result_id == "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools"
            })
            .expect("VCF call result row");
        assert_eq!(vcf_call.report_section, "variant_calling");
        assert!(vcf_call.expected_outputs.iter().any(|value| value == "called_vcf"));
        assert!(vcf_call.expected_metrics.iter().any(|value| value == "variant_count"));
    }

    #[test]
    fn report_path_never_persists_an_absolute_workspace_location() {
        let root = std::path::Path::new("/workspace");
        let external = std::path::Path::new("/outside-repository/expected-results.tsv");
        assert_eq!(path_relative_to_repo(root, external), "expected-results.tsv");
        let governed = root.join("artifacts/readiness/expected-results.tsv");
        assert_eq!(
            path_relative_to_repo(root, &governed),
            "artifacts/readiness/expected-results.tsv"
        );
    }
}
