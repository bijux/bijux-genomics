use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::{
    validate_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus, AllDomainOutputDeclarationsReport,
};
use bijux_dna_api::v1::api::bench::{
    validate_all_domain_output_declarations_report, ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1,
};

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::bam_adapter_output_contract::{
    collect_bam_adapter_output_contract_rows, BamAdapterOutputContractStatus,
};
use super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use super::fastq_adapter_output_contract::{
    collect_fastq_adapter_output_contract_rows, FastqAdapterOutputContractStatus,
};
use super::vcf_adapter_output_coverage::{
    collect_vcf_adapter_output_coverage_rows, VcfAdapterOutputCoverageStatus,
};
use super::vcf_expected_benchmark_results::{
    collect_vcf_expected_benchmark_result_rows, VcfExpectedBenchmarkResultRow,
};
use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/output-declarations-all-domains.tsv";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

pub(crate) fn run_render_all_domain_output_declarations(
    args: &parse::BenchReadinessRenderAllDomainOutputDeclarationsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_output_declarations(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_output_declarations(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainOutputDeclarationsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_all_domain_output_declaration_rows(repo_root)?;

    let row_count = rows.len();
    let result_id_count =
        rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    let complete_row_count =
        rows.iter().filter(|row| row.status == AllDomainOutputDeclarationStatus::Complete).count();
    let incomplete_row_count = row_count.saturating_sub(complete_row_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *status_counts.entry(row.status.as_str().to_string()).or_default() += 1;
    }

    let report = AllDomainOutputDeclarationsReport {
        schema_version: ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1.to_string(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        result_id_count,
        complete_row_count,
        incomplete_row_count,
        domain_counts,
        status_counts,
        rows,
    };
    validate_all_domain_output_declarations_report(&report)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_output_declarations_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

pub(crate) fn collect_all_domain_output_declaration_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainOutputDeclarationRow>> {
    let canonical_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let fastq_bam_expected_by_result_id = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_row_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let fastq_contract_by_binding = collect_fastq_adapter_output_contract_rows(repo_root)?
        .into_iter()
        .map(|row| (binding_key("fastq", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let bam_contract_by_binding = collect_bam_adapter_output_contract_rows(repo_root)?
        .into_iter()
        .map(|row| (binding_key("bam", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let vcf_output_by_binding = collect_vcf_adapter_output_coverage_rows(repo_root)?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (binding_key("vcf", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let vcf_expected_by_result_id = collect_vcf_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| (vcf_result_id(&row), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(canonical_rows.len());
    for canonical in canonical_rows {
        match canonical.domain.as_str() {
            "fastq" => {
                let expected = fastq_bam_expected_by_result_id.get(&canonical.result_id).ok_or_else(
                    || {
                        anyhow!(
                            "all-domain output declarations are missing FASTQ expected-result coverage for `{}`",
                            canonical.result_id
                        )
                    },
                )?;
                let contract = fastq_contract_by_binding
                    .get(&binding_key("fastq", &canonical.stage_id, &canonical.tool_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "all-domain output declarations are missing FASTQ adapter output coverage for `{}` / `{}`",
                            canonical.stage_id,
                            canonical.tool_id
                        )
                    })?;
                rows.push(AllDomainOutputDeclarationRow {
                    result_id: canonical.result_id,
                    domain: canonical.domain,
                    stage_id: canonical.stage_id,
                    tool_id: canonical.tool_id,
                    corpus_id: canonical.corpus_id,
                    asset_profile_id: canonical.asset_profile_id,
                    raw_outputs: expected.raw_output_artifact_ids.clone(),
                    normalized_metrics: expected
                        .normalized_metrics_output_id
                        .clone()
                        .into_iter()
                        .collect(),
                    logs: vec![
                        format!("stdout={}", expected.stdout_path),
                        format!("stderr={}", expected.stderr_path),
                    ],
                    manifest: expected.stage_result_manifest_path.clone(),
                    index_outputs: Vec::new(),
                    status: fastq_status(contract.output_contract_status),
                });
            }
            "bam" => {
                let expected = fastq_bam_expected_by_result_id.get(&canonical.result_id).ok_or_else(
                    || {
                        anyhow!(
                            "all-domain output declarations are missing BAM expected-result coverage for `{}`",
                            canonical.result_id
                        )
                    },
                )?;
                let contract = bam_contract_by_binding
                    .get(&binding_key("bam", &canonical.stage_id, &canonical.tool_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "all-domain output declarations are missing BAM adapter output coverage for `{}` / `{}`",
                            canonical.stage_id,
                            canonical.tool_id
                        )
                    })?;
                rows.push(AllDomainOutputDeclarationRow {
                    result_id: canonical.result_id,
                    domain: canonical.domain,
                    stage_id: canonical.stage_id,
                    tool_id: canonical.tool_id,
                    corpus_id: canonical.corpus_id,
                    asset_profile_id: canonical.asset_profile_id,
                    raw_outputs: expected.raw_output_artifact_ids.clone(),
                    normalized_metrics: expected
                        .normalized_metrics_output_id
                        .clone()
                        .into_iter()
                        .collect(),
                    logs: vec![
                        format!("stdout={}", expected.stdout_path),
                        format!("stderr={}", expected.stderr_path),
                    ],
                    manifest: expected.stage_result_manifest_path.clone(),
                    index_outputs: Vec::new(),
                    status: bam_status(contract.output_contract_status),
                });
            }
            "vcf" => {
                let expected = vcf_expected_by_result_id.get(&canonical.result_id).ok_or_else(|| {
                    anyhow!(
                        "all-domain output declarations are missing VCF expected-result coverage for `{}`",
                        canonical.result_id
                    )
                })?;
                let coverage = vcf_output_by_binding
                    .get(&binding_key("vcf", &canonical.stage_id, &canonical.tool_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "all-domain output declarations are missing VCF output coverage for `{}` / `{}`",
                            canonical.stage_id,
                            canonical.tool_id
                        )
                    })?;
                let result_root = vcf_result_root(expected);
                rows.push(AllDomainOutputDeclarationRow {
                    result_id: canonical.result_id,
                    domain: canonical.domain,
                    stage_id: canonical.stage_id,
                    tool_id: canonical.tool_id,
                    corpus_id: canonical.corpus_id,
                    asset_profile_id: canonical.asset_profile_id,
                    raw_outputs: coverage
                        .raw_outputs
                        .iter()
                        .map(|entry| artifact_id(entry))
                        .collect(),
                    normalized_metrics: coverage
                        .normalized_metrics
                        .iter()
                        .map(|entry| artifact_id(entry))
                        .collect(),
                    logs: vec![
                        format!("stdout={result_root}/stdout.log"),
                        format!("stderr={result_root}/stderr.log"),
                    ],
                    manifest: format!("{result_root}/stage-result.json"),
                    index_outputs: coverage
                        .index_outputs
                        .iter()
                        .map(|entry| artifact_id(entry))
                        .collect(),
                    status: vcf_status(coverage.status),
                });
            }
            other => {
                return Err(anyhow!(
                    "all-domain output declarations do not support legacy domain `{other}`"
                ));
            }
        }
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    validate_all_domain_output_declaration_rows(&rows)?;
    Ok(rows)
}

fn render_all_domain_output_declarations_tsv(rows: &[AllDomainOutputDeclarationRow]) -> String {
    let mut rendered = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\traw_outputs\tnormalized_metrics\tlogs\tmanifest\tindex_outputs\tstatus\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.raw_outputs.join(",")),
            sanitize_tsv(&row.normalized_metrics.join(",")),
            sanitize_tsv(&row.logs.join(",")),
            sanitize_tsv(&row.manifest),
            sanitize_tsv(&row.index_outputs.join(",")),
            sanitize_tsv(row.status.as_str()),
        ));
    }
    rendered
}

fn fastq_status(status: FastqAdapterOutputContractStatus) -> AllDomainOutputDeclarationStatus {
    match status {
        FastqAdapterOutputContractStatus::Complete => AllDomainOutputDeclarationStatus::Complete,
        FastqAdapterOutputContractStatus::Incomplete => {
            AllDomainOutputDeclarationStatus::Incomplete
        }
        FastqAdapterOutputContractStatus::MissingAdapter => {
            AllDomainOutputDeclarationStatus::MissingAdapter
        }
    }
}

fn bam_status(status: BamAdapterOutputContractStatus) -> AllDomainOutputDeclarationStatus {
    match status {
        BamAdapterOutputContractStatus::Complete => AllDomainOutputDeclarationStatus::Complete,
        BamAdapterOutputContractStatus::Incomplete => AllDomainOutputDeclarationStatus::Incomplete,
        BamAdapterOutputContractStatus::MissingAdapter => {
            AllDomainOutputDeclarationStatus::MissingAdapter
        }
    }
}

fn vcf_status(status: VcfAdapterOutputCoverageStatus) -> AllDomainOutputDeclarationStatus {
    match status {
        VcfAdapterOutputCoverageStatus::Complete => AllDomainOutputDeclarationStatus::Complete,
        VcfAdapterOutputCoverageStatus::Incomplete => AllDomainOutputDeclarationStatus::Incomplete,
    }
}

fn artifact_id(entry: &str) -> String {
    entry
        .split_once('=')
        .map_or_else(|| entry.to_string(), |(artifact_id, _)| artifact_id.to_string())
}

fn vcf_result_root(row: &VcfExpectedBenchmarkResultRow) -> String {
    format!(
        "artifacts/runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        LOCAL_SLURM_DRY_RUN_RUN_ID, row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id
    )
}

fn vcf_result_id(row: &VcfExpectedBenchmarkResultRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.domain, row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id
    )
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
