use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::{
    validate_stage_tool_containers_config, StageToolContainerRow, StageToolContainersConfig,
    StageToolContainersReport, LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1,
};
use bijux_dna_api::v1::api::bench::{
    validate_stage_tool_containers_report, STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN,
    STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1, STAGE_TOOL_CONTAINERS_SCOPE,
};

use super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows;
use super::tool_execution_modes::{
    load_runtime_probe, validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_TOOL_CONTAINERS_PATH: &str =
    "configs/bench/local/stage-tool-containers.toml";

pub(crate) fn run_render_stage_tool_containers(
    args: &parse::BenchReadinessRenderStageToolContainersArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_tool_containers(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_TOOL_CONTAINERS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_stage_tool_containers(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageToolContainersReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_stage_tool_container_rows(repo_root)?;

    let config = StageToolContainersConfig {
        schema_version: LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1.to_string(),
        classification_scope: STAGE_TOOL_CONTAINERS_SCOPE.to_string(),
        rows: rows.clone(),
    };
    validate_stage_tool_containers_config(&config)?;
    let rendered = render_stage_tool_containers_toml(&config)?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut execution_mode_counts = BTreeMap::<String, usize>::new();
    let mut external_row_count = 0_usize;
    let mut container_declared_row_count = 0_usize;
    let mut command_entrypoint_row_count = 0_usize;
    let mut host_binary_row_count = 0_usize;
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *execution_mode_counts.entry(row.execution_mode.clone()).or_default() += 1;
        if row.install_kind != "workspace_binary" {
            external_row_count += 1;
        }
        if row.container_id.is_some() {
            container_declared_row_count += 1;
        }
        if row.command_entrypoint.is_some() {
            command_entrypoint_row_count += 1;
        }
        if row.host_binary_mode.is_some() {
            host_binary_row_count += 1;
        }
    }

    let report = StageToolContainersReport {
        schema_version: STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1.to_string(),
        config_path: path_relative_to_repo(repo_root, &output_path),
        classification_scope: STAGE_TOOL_CONTAINERS_SCOPE.to_string(),
        row_count: rows.len(),
        benchmark_ready_row_count: rows.len(),
        external_row_count,
        container_declared_row_count,
        command_entrypoint_row_count,
        host_binary_row_count,
        domain_counts,
        execution_mode_counts,
        rows,
    };
    validate_stage_tool_containers_report(&report)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;
    Ok(report)
}

fn collect_stage_tool_container_rows(repo_root: &Path) -> Result<Vec<StageToolContainerRow>> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let execution_mode_report = validate_tool_execution_modes_path(
        repo_root,
        &repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
    )?;
    let execution_modes = execution_mode_report
        .rows
        .into_iter()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for row in active_rows {
        rows.push(render_stage_tool_container_row(
            repo_root,
            &row.domain,
            &row.stage_id,
            &row.tool_id,
            &execution_modes,
        )?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_stage_tool_rows(&rows)?;
    Ok(rows)
}

fn render_stage_tool_container_row(
    repo_root: &Path,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    execution_modes: &BTreeMap<String, super::tool_execution_modes::ToolExecutionModeRow>,
) -> Result<StageToolContainerRow> {
    let execution_mode = execution_modes.get(tool_id).ok_or_else(|| {
        anyhow!("missing execution mode declaration for benchmark-ready tool `{tool_id}`")
    })?;
    let probe = load_runtime_probe(repo_root, domain, tool_id).with_context(|| {
        format!("load runtime probe for benchmark-ready row `{stage_id}` / `{tool_id}`")
    })?;

    let install_kind = probe.install_kind().to_string();
    if install_kind != execution_mode.expected_install_kind {
        return Err(anyhow!(
            "runtime install_kind `{}` for benchmark-ready row `{}` / `{}` does not match execution-mode expectation `{}`",
            install_kind,
            stage_id,
            tool_id,
            execution_mode.expected_install_kind
        ));
    }

    let host_binary_mode = match install_kind.as_str() {
        "workspace_binary" | "host_binary" => Some(install_kind.clone()),
        _ => None,
    };
    let container_id = probe.container_id();
    let command_entrypoint = probe.command_entrypoint();

    if install_kind == "container" && container_id.is_none() && command_entrypoint.is_none() {
        return Err(anyhow!(
            "benchmark-ready external row `{stage_id}` / `{tool_id}` must declare a container image or a command entrypoint"
        ));
    }
    if install_kind == "workspace_binary" && host_binary_mode.is_none() {
        return Err(anyhow!(
            "benchmark-ready workspace row `{stage_id}` / `{tool_id}` must declare an explicit host-binary mode"
        ));
    }
    if container_id.is_none() && command_entrypoint.is_none() && host_binary_mode.is_none() {
        return Err(anyhow!(
            "benchmark-ready row `{stage_id}` / `{tool_id}` must declare runtime location details"
        ));
    }

    Ok(StageToolContainerRow {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.execution_mode.clone(),
        install_kind,
        container_id,
        command_entrypoint,
        host_binary_mode,
        declaration_origin: STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN.to_string(),
    })
}

fn ensure_unique_stage_tool_rows(rows: &[StageToolContainerRow]) -> Result<()> {
    let mut seen = BTreeSet::<(String, String)>::new();
    for row in rows {
        let pair = (row.stage_id.clone(), row.tool_id.clone());
        if !seen.insert(pair.clone()) {
            return Err(anyhow!(
                "stage-tool container rows repeat benchmark-ready pair `{}` / `{}`",
                pair.0,
                pair.1
            ));
        }
    }
    Ok(())
}

fn render_stage_tool_containers_toml(config: &StageToolContainersConfig) -> Result<String> {
    let body = toml::to_string_pretty(config).context("serialize stage-tool containers config")?;
    Ok(format!(
        "# schema_version = 1\n\
         # owner = bijux-dna-bench\n\
         # purpose = Governed runtime declarations for benchmark-ready FASTQ, BAM, and VCF stage-tool commands.\n\
         # authority = bijux-dna-bench\n\
         # stability = evolving\n\
         # last_updated = 2026-08-13\n\n{body}"
    ))
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_containers_reports_governed_benchmark_ready_row_slice() {
        use super::{render_stage_tool_containers, DEFAULT_STAGE_TOOL_CONTAINERS_PATH};

        let root = repo_root();
        let report =
            render_stage_tool_containers(&root, PathBuf::from(DEFAULT_STAGE_TOOL_CONTAINERS_PATH))
                .expect("render stage-tool containers");

        assert_eq!(report.schema_version, "bijux.bench.readiness.stage_tool_containers.v1");
        assert_eq!(report.config_path, "configs/bench/local/stage-tool-containers.toml");
        assert_eq!(report.classification_scope, "benchmark_ready_runtime_declarations");
        assert_eq!(report.row_count, 159);
        assert_eq!(report.benchmark_ready_row_count, report.row_count);
        assert_eq!(report.external_row_count, report.container_declared_row_count);
        assert_eq!(report.command_entrypoint_row_count, report.row_count);
        assert_eq!(report.domain_counts.values().sum::<usize>(), report.row_count);
        assert_eq!(report.domain_counts.get("vcf"), Some(&39));
        assert!(report.rows.iter().all(|row| {
            row.container_id.is_some()
                || row.command_entrypoint.is_some()
                || row.host_binary_mode.is_some()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.normalize_primers"
                && row.tool_id == "cutadapt"
                && row
                    .container_id
                    .as_deref()
                    .is_some_and(|value| value.starts_with("bijuxdna/cutadapt@sha256:"))
                && row.command_entrypoint.as_deref() == Some("cutadapt")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("fastqc")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789"
                    )
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.filter_reads"
                && row.tool_id == "fastp"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("fastp")
                && row.container_id.as_deref()
                    == Some(
                    "bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10"
                )
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bwa")
                && row.container_id.as_deref() == Some("bijuxdna/bwa")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bowtie2"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bowtie2")
                && row.container_id.as_deref() == Some("bijuxdna/bowtie2")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("king")
                && row.container_id.as_deref() == Some("bijuxdna/king:2.3.0")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "bcftools"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bcftools")
                && row.container_id.as_deref()
                    == Some("quay.io/biocontainers/bcftools:1.20--h8b25389_0@sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6")
        }));
        for (stage_id, external_tool_id) in [
            ("bam.damage", "ngsbriggs"),
            ("bam.kinship", "angsd"),
            ("bam.sex", "angsd"),
            ("fastq.remove_duplicates", "fastuniq"),
            ("vcf.call_gl", "angsd"),
            ("vcf.call_pseudohaploid", "angsd"),
            ("vcf.imputation_metrics", "impute5"),
            ("vcf.impute", "impute5"),
        ] {
            let expected_container = format!("external-artifact://{external_tool_id}");
            assert!(report.rows.iter().any(|row| {
                row.stage_id == stage_id
                    && row.tool_id == external_tool_id
                    && row.execution_mode == "external"
                    && row.container_id.as_deref() == Some(expected_container.as_str())
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.overlap_correction"
                && row.tool_id == "bamutil"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bamutil")
                && row.container_id.as_deref() == Some("bijuxdna/bamutil:1.0.15")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.bias_mitigation"
                && row.tool_id == "mapdamage2"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("mapdamage2")
                && row.container_id.as_deref() == Some("bijuxdna/mapdamage2:2.2.2")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("rxy")
                && row.container_id.as_deref() == Some("bijuxdna/rxy")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.tool_id == "yleaf"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("yleaf")
                && row.container_id.as_deref() == Some("bijuxdna/yleaf")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_polyg_tails"
                && row.tool_id == "fastp"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("fastp")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10"
                    )
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_terminal_damage"
                && row.tool_id == "cutadapt"
                && row.execution_mode == "python"
                && row.command_entrypoint.as_deref() == Some("cutadapt")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba"
                    )
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.execution_mode == "internal"
                && row.command_entrypoint.as_deref() == Some("bijux-dna")
                && row.host_binary_mode.as_deref() == Some("workspace_binary")
                && row.container_id.is_none()
        }));
        for (tool_id, container_id) in [
            ("bedtools", "bijuxdna/bedtools:2.31.1"),
            ("mosdepth", "bijuxdna/mosdepth:0.3.11"),
            ("samtools", "bijuxdna/samtools:1.21"),
        ] {
            assert!(report.rows.iter().any(|row| {
                row.stage_id == "bam.coverage"
                    && row.tool_id == tool_id
                    && row.execution_mode == "containerized"
                    && row.command_entrypoint.as_deref() == Some(tool_id)
                    && row.container_id.as_deref() == Some(container_id)
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.gc_bias"
                && row.tool_id == "picard"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("picard")
                && row.container_id.as_deref() == Some("bijuxdna/picard:3.3.0")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "schmutzi"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("schmutzi")
                && row.container_id.as_deref() == Some("bijuxdna/schmutzi")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.endogenous_content"
                && row.tool_id == "samtools"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("samtools")
                && row.container_id.as_deref() == Some("bijuxdna/samtools:1.21")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.mapping_summary"
                && row.tool_id == "picard"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("picard")
                && row.container_id.as_deref() == Some("bijuxdna/picard:3.3.0")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.insert_size"
                && row.tool_id == "picard"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("picard")
                && row.container_id.as_deref() == Some("bijuxdna/picard:3.3.0")
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_containers_rejects_external_output_before_write() {
        use super::render_stage_tool_containers;

        let root = repo_root();
        let external_root = tempfile::tempdir().expect("external output root");
        let output_path = external_root.path().join("stage-tool-containers.toml");
        let error = render_stage_tool_containers(&root, output_path.clone())
            .expect_err("external output path must be rejected");

        assert!(error.to_string().contains("report identity or path is invalid"));
        assert!(!output_path.exists());
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_containers_writes_governed_toml_contract() {
        use super::{
            render_stage_tool_containers, StageToolContainersConfig,
            DEFAULT_STAGE_TOOL_CONTAINERS_PATH, LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1,
            STAGE_TOOL_CONTAINERS_SCOPE,
        };

        let root = repo_root();
        let output_path = root.join(DEFAULT_STAGE_TOOL_CONTAINERS_PATH);
        let _report =
            render_stage_tool_containers(&root, PathBuf::from(DEFAULT_STAGE_TOOL_CONTAINERS_PATH))
                .expect("render stage-tool containers");

        let raw = std::fs::read_to_string(&output_path).expect("read rendered config");
        let config: StageToolContainersConfig =
            toml::from_str(&raw).expect("parse rendered stage-tool container config");

        assert_eq!(config.schema_version, LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1);
        assert_eq!(config.classification_scope, STAGE_TOOL_CONTAINERS_SCOPE);
        assert!(!config.rows.is_empty());
        assert!(config.rows.iter().all(|row| {
            row.container_id.is_some()
                || row.command_entrypoint.is_some()
                || row.host_binary_mode.is_some()
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "bcftools"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bcftools")
                && row.container_id.as_deref()
                    == Some("quay.io/biocontainers/bcftools:1.20--h8b25389_0@sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6")
        }));
        for (stage_id, external_tool_id) in [
            ("bam.damage", "ngsbriggs"),
            ("bam.kinship", "angsd"),
            ("bam.sex", "angsd"),
            ("fastq.remove_duplicates", "fastuniq"),
            ("vcf.call_gl", "angsd"),
            ("vcf.call_pseudohaploid", "angsd"),
            ("vcf.imputation_metrics", "impute5"),
            ("vcf.impute", "impute5"),
        ] {
            let expected_container = format!("external-artifact://{external_tool_id}");
            assert!(config.rows.iter().any(|row| {
                row.stage_id == stage_id
                    && row.tool_id == external_tool_id
                    && row.execution_mode == "external"
                    && row.container_id.as_deref() == Some(expected_container.as_str())
            }));
        }
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("fastqc")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789"
                    )
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_polyg_tails"
                && row.tool_id == "fastp"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("fastp")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10"
                    )
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_terminal_damage"
                && row.tool_id == "cutadapt"
                && row.execution_mode == "python"
                && row.command_entrypoint.as_deref() == Some("cutadapt")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba"
                    )
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bowtie2-build")
                && row.container_id.as_deref() == Some("bijuxdna/bowtie2_build")
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.execution_mode == "internal"
                && row.command_entrypoint.as_deref() == Some("bijux-dna")
                && row.host_binary_mode.as_deref() == Some("workspace_binary")
                && row.container_id.is_none()
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "schmutzi"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("schmutzi")
                && row.container_id.as_deref() == Some("bijuxdna/schmutzi")
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("rxy")
                && row.container_id.as_deref() == Some("bijuxdna/rxy")
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.overlap_correction"
                && row.tool_id == "bamutil"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("bamutil")
                && row.container_id.as_deref() == Some("bijuxdna/bamutil:1.0.15")
        }));
        for (tool_id, container_id) in [
            ("bedtools", "bijuxdna/bedtools:2.31.1"),
            ("mosdepth", "bijuxdna/mosdepth:0.3.11"),
            ("samtools", "bijuxdna/samtools:1.21"),
        ] {
            assert!(config.rows.iter().any(|row| {
                row.stage_id == "bam.coverage"
                    && row.tool_id == tool_id
                    && row.execution_mode == "containerized"
                    && row.command_entrypoint.as_deref() == Some(tool_id)
                    && row.container_id.as_deref() == Some(container_id)
            }));
        }
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.gc_bias"
                && row.tool_id == "picard"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("picard")
                && row.container_id.as_deref() == Some("bijuxdna/picard:3.3.0")
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.endogenous_content"
                && row.tool_id == "samtools"
                && row.execution_mode == "containerized"
                && row.command_entrypoint.as_deref() == Some("samtools")
                && row.container_id.as_deref() == Some("bijuxdna/samtools:1.21")
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.mapping_summary"
                && row.tool_id == "picard"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("picard")
                && row.container_id.as_deref() == Some("bijuxdna/picard:3.3.0")
        }));
    }
}
