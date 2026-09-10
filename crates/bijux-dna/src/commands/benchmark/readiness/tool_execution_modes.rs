use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::tool_serving_map::{
    collect_bam_tool_serving_map_rows, collect_fastq_tool_serving_map_rows, ToolServingMapRow,
};
use super::vcf_tool_serving_map::{collect_vcf_tool_serving_map_rows, VcfToolServingMapRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.tool_execution_modes_validation.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolExecutionModeRow {
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) expected_install_kind: String,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) required_runtime_fields: Vec<String>,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolExecutionModesValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) classification_scope: String,
    pub(crate) mode_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) multidomain_tool_count: usize,
    pub(crate) mode_counts: BTreeMap<String, usize>,
    pub(crate) valid: bool,
    pub(crate) rows: Vec<ToolExecutionModeRow>,
}

pub(crate) fn run_validate_tool_execution_modes(
    args: &parse::BenchReadinessValidateToolExecutionModesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let config_path = match &args.config {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
    };
    let report = validate_tool_execution_modes_path(&repo_root, &config_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn validate_tool_execution_modes_path(
    repo_root: &Path,
    config_path: &Path,
) -> Result<ToolExecutionModesValidationReport> {
    let config = load_tool_execution_modes_config(config_path)?;

    let (_, _, fastq_rows) = collect_fastq_tool_serving_map_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_tool_serving_map_rows(repo_root)?;
    let vcf_rows = collect_vcf_tool_serving_map_rows()?;
    let benchmark_tool_index = build_benchmark_tool_index(&fastq_rows, &bam_rows, &vcf_rows);
    let assignments = config.assignments()?;

    let configured_tool_ids = assignments.keys().cloned().collect::<BTreeSet<_>>();
    let benchmark_tool_ids = benchmark_tool_index.keys().cloned().collect::<BTreeSet<_>>();

    let missing_tool_ids =
        benchmark_tool_ids.difference(&configured_tool_ids).cloned().collect::<Vec<_>>();
    if !missing_tool_ids.is_empty() {
        return Err(anyhow!(
            "tool execution mode config is missing {} benchmark tool ids: {}",
            missing_tool_ids.len(),
            missing_tool_ids.join(", "),
        ));
    }

    let extra_tool_ids =
        configured_tool_ids.difference(&benchmark_tool_ids).cloned().collect::<Vec<_>>();
    if !extra_tool_ids.is_empty() {
        return Err(anyhow!(
            "tool execution mode config declares {} tool ids outside the governed benchmark scope: {}",
            extra_tool_ids.len(),
            extra_tool_ids.join(", "),
        ));
    }

    for (tool_id, assignment) in &assignments {
        let benchmark_entry = benchmark_tool_index
            .get(tool_id)
            .expect("benchmark tool coverage must exist after set validation");
        for domain in &benchmark_entry.domains {
            let probe = load_runtime_probe(repo_root, domain, tool_id)?;
            if probe.install_kind() != assignment.expected_install_kind {
                return Err(anyhow!(
                    "tool `{}` in domain `{}` declares install_kind `{}` but execution mode config expects `{}`",
                    tool_id,
                    domain,
                    probe.install_kind(),
                    assignment.expected_install_kind
                ));
            }
            for field in &assignment.required_runtime_fields {
                if !probe.has_required_field(field) {
                    return Err(anyhow!(
                        "tool `{}` in domain `{}` is missing required runtime field `{}` for execution mode `{}`",
                        tool_id,
                        domain,
                        field,
                        assignment.execution_mode
                    ));
                }
            }
        }
    }

    let mut mode_counts = BTreeMap::<String, usize>::new();
    let mut rows = Vec::with_capacity(benchmark_tool_index.len());
    for (tool_id, benchmark_entry) in benchmark_tool_index {
        let assignment =
            assignments.get(&tool_id).expect("tool execution mode assignment must exist");
        *mode_counts.entry(assignment.execution_mode.clone()).or_default() += 1;
        rows.push(ToolExecutionModeRow {
            tool_id,
            execution_mode: assignment.execution_mode.clone(),
            expected_install_kind: assignment.expected_install_kind.clone(),
            domains: benchmark_entry.domains,
            stage_ids: benchmark_entry.stage_ids,
            required_runtime_fields: assignment.required_runtime_fields.clone(),
            summary: assignment.summary.clone(),
        });
    }
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));

    let multidomain_tool_count = rows.iter().filter(|row| row.domains.len() > 1).count();

    Ok(ToolExecutionModesValidationReport {
        schema_version: TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, config_path),
        classification_scope: config.classification_scope().to_string(),
        mode_count: config.mode_count(),
        tool_count: rows.len(),
        multidomain_tool_count,
        mode_counts,
        valid: true,
        rows,
    })
}

pub(crate) use bijux_dna_api::v1::api::bench::{
    load_runtime_probe, load_runtime_probe_with_source, load_tool_execution_mode_assignments,
    load_tool_execution_modes_config, BenchmarkRuntimeProbe as RuntimeProbe,
    ToolExecutionModeAssignment, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};

#[derive(Debug, Clone)]
struct BenchmarkToolEntry {
    domains: Vec<String>,
    stage_ids: Vec<String>,
}

fn build_benchmark_tool_index(
    fastq_rows: &[ToolServingMapRow],
    bam_rows: &[ToolServingMapRow],
    vcf_rows: &[VcfToolServingMapRow],
) -> BTreeMap<String, BenchmarkToolEntry> {
    let mut domains_by_tool = BTreeMap::<String, BTreeSet<String>>::new();
    let mut stages_by_tool = BTreeMap::<String, BTreeSet<String>>::new();

    for row in fastq_rows {
        domains_by_tool.entry(row.tool_id.clone()).or_default().insert("fastq".to_string());
        stages_by_tool.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }
    for row in bam_rows {
        domains_by_tool.entry(row.tool_id.clone()).or_default().insert("bam".to_string());
        stages_by_tool.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }
    for row in vcf_rows {
        domains_by_tool.entry(row.tool_id.clone()).or_default().insert("vcf".to_string());
        stages_by_tool.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }

    domains_by_tool
        .into_iter()
        .map(|(tool_id, domains)| {
            let stage_ids =
                stages_by_tool.remove(&tool_id).unwrap_or_default().into_iter().collect::<Vec<_>>();
            (
                tool_id,
                BenchmarkToolEntry { domains: domains.into_iter().collect::<Vec<_>>(), stage_ids },
            )
        })
        .collect()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
        TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn tool_execution_mode_config_covers_governed_benchmark_tools() {
        let root = repo_root();
        let report = validate_tool_execution_modes_path(
            &root,
            &root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
        )
        .expect("validate tool execution modes");

        assert_eq!(report.schema_version, TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.classification_scope, "primary_operator_runtime");
        assert_eq!(report.mode_count, 7);
        assert_eq!(report.tool_count, report.rows.len());
        assert!(report.tool_count > 0);
        assert!(report.valid, "tool execution mode config must validate cleanly");

        let bijux_dna =
            report.rows.iter().find(|row| row.tool_id == "bijux_dna").expect("bijux_dna row");
        assert_eq!(bijux_dna.execution_mode, "internal");
        assert_eq!(bijux_dna.expected_install_kind, "workspace_binary");

        let cutadapt =
            report.rows.iter().find(|row| row.tool_id == "cutadapt").expect("cutadapt row");
        assert_eq!(cutadapt.execution_mode, "python");

        let gatk = report.rows.iter().find(|row| row.tool_id == "gatk").expect("gatk row");
        assert_eq!(gatk.execution_mode, "java");
    }

    #[test]
    fn tool_execution_mode_rows_preserve_governed_cross_domain_scope() {
        let root = repo_root();
        let report = validate_tool_execution_modes_path(
            &root,
            &root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
        )
        .expect("validate tool execution modes");

        let bowtie2 = report.rows.iter().find(|row| row.tool_id == "bowtie2").expect("bowtie2 row");
        assert_eq!(bowtie2.execution_mode, "containerized");
        assert_eq!(bowtie2.domains, vec!["bam".to_string(), "fastq".to_string()]);
        assert!(
            bowtie2.stage_ids.contains(&"bam.align".to_string())
                && bowtie2.stage_ids.contains(&"fastq.deplete_host".to_string()),
            "cross-domain execution-mode rows must preserve their governed benchmark stage scope"
        );

        let trim_galore =
            report.rows.iter().find(|row| row.tool_id == "trim_galore").expect("trim_galore row");
        assert_eq!(trim_galore.execution_mode, "mixed");

        for external_tool_id in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
            let row = report
                .rows
                .iter()
                .find(|row| row.tool_id == external_tool_id)
                .expect("external artifact must remain in benchmark integration scope");
            assert_eq!(row.execution_mode, "external");
            assert_eq!(row.expected_install_kind, "external");
        }

        let germline =
            report.rows.iter().find(|row| row.tool_id == "germline").expect("germline row");
        assert_eq!(germline.execution_mode, "containerized");
        assert_eq!(germline.domains, vec!["vcf".to_string()]);

        for tool_id in ["eagle", "glimpse", "ibdhap", "minimac4", "shapeit"] {
            let row = report
                .rows
                .iter()
                .find(|row| row.tool_id == tool_id)
                .unwrap_or_else(|| panic!("supported VCF tool `{tool_id}` execution mode"));
            assert_eq!(row.execution_mode, "containerized");
            assert_eq!(row.expected_install_kind, "container");
            assert_eq!(row.domains, vec!["vcf".to_string()]);
        }

        let yleaf = report.rows.iter().find(|row| row.tool_id == "yleaf").expect("yleaf row");
        assert_eq!(yleaf.execution_mode, "containerized");
        assert_eq!(yleaf.domains, vec!["bam".to_string()]);
        assert!(
            yleaf.stage_ids.contains(&"bam.haplogroups".to_string())
                && yleaf.stage_ids.contains(&"bam.sex".to_string()),
            "shared BAM execution-mode rows must preserve the governed haplogroups and sex stage scope"
        );
    }
}
