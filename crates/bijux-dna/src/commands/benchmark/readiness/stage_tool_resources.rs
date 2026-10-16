use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    fastq_benchmark_walltime_minutes, validate_stage_tool_resources_report,
    vcf_benchmark_walltime_minutes, STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1,
    STAGE_TOOL_RESOURCES_SCOPE,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    validate_stage_tool_resources_config, StageToolResourceRow, StageToolResourcesConfig,
    StageToolResourcesReport, BAM_STAGE_TOOL_RESOURCE_ORIGIN, FASTQ_STAGE_TOOL_RESOURCE_ORIGIN,
    LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1, VCF_STAGE_TOOL_RESOURCE_ORIGIN,
};
use bijux_dna_core::contract::ToolExecutionSpecV1;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{estimate_bam_stage_resources_with_origin, BamInputOriginV1};
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_planner_bam::stage_api::load_bam_domain_tool_execution_spec;
use bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_execution_spec;
use bijux_dna_planner_vcf::vcf_stage_resource_constraints;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) use bijux_dna_api::v1::api::bench::HPC_JOB_RESOURCE_CONFIG_PATH as DEFAULT_STAGE_TOOL_RESOURCES_PATH;
const TINY_BAM_RESOURCE_INPUT_PATHS: &[&str] = &[
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam",
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam",
];

pub(crate) fn run_render_stage_tool_resources(
    args: &parse::BenchReadinessRenderStageToolResourcesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_tool_resources(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_TOOL_RESOURCES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_stage_tool_resources(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageToolResourcesReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_stage_tool_resource_rows(repo_root)?;

    let config = StageToolResourcesConfig {
        schema_version: LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1.to_string(),
        classification_scope: STAGE_TOOL_RESOURCES_SCOPE.to_string(),
        rows: rows.clone(),
    };
    validate_stage_tool_resources_config(&config)?;
    let rendered = render_stage_tool_resources_toml(&config)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut resource_origin_counts = BTreeMap::<String, usize>::new();
    let mut nonzero_resource_row_count = 0_usize;
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *resource_origin_counts.entry(row.resource_origin.clone()).or_default() += 1;
        if row.threads > 0 && row.memory_gb > 0 && row.walltime_minutes > 0 && row.scratch_gb > 0 {
            nonzero_resource_row_count += 1;
        }
    }

    let report = StageToolResourcesReport {
        schema_version: STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1.to_string(),
        config_path: path_relative_to_repo(repo_root, &output_path),
        classification_scope: STAGE_TOOL_RESOURCES_SCOPE.to_string(),
        row_count: rows.len(),
        benchmark_ready_row_count: rows.len(),
        nonzero_resource_row_count,
        domain_counts,
        resource_origin_counts,
        rows,
    };
    validate_stage_tool_resources_report(&report)?;
    Ok(report)
}

fn collect_stage_tool_resource_rows(repo_root: &Path) -> Result<Vec<StageToolResourceRow>> {
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let vcf_rows = collect_vcf_tool_serving_map_rows()?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready");
    let bam_input_bytes = detect_tiny_bam_resource_input_bytes(repo_root)?;

    let mut rows = Vec::new();
    for row in fastq_rows
        .into_iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
    {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)
            .with_context(|| {
                format!(
                    "load FASTQ execution spec for benchmark-ready row `{}` / `{}`",
                    row.stage_id, row.tool_id
                )
            })?;
        rows.push(render_fastq_resource_row(&stage_id, &tool_id, &spec));
    }

    for row in bam_rows
        .into_iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
    {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let spec = load_bam_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)
            .with_context(|| {
                format!(
                    "load BAM execution spec for benchmark-ready row `{}` / `{}`",
                    row.stage_id, row.tool_id
                )
            })?;
        rows.push(render_bam_resource_row(&stage_id, &tool_id, &spec, bam_input_bytes));
    }

    for row in vcf_rows {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let stage = VcfDomainStage::try_from(row.stage_id.as_str()).with_context(|| {
            format!("parse VCF stage id `{}` for benchmark-ready resource coverage", row.stage_id)
        })?;
        rows.push(render_vcf_resource_row(&stage_id, &tool_id, stage));
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

fn render_fastq_resource_row(
    stage_id: &StageId,
    tool_id: &ToolId,
    spec: &ToolExecutionSpecV1,
) -> StageToolResourceRow {
    StageToolResourceRow {
        domain: "fastq".to_string(),
        stage_id: stage_id.as_str().to_string(),
        tool_id: tool_id.as_str().to_string(),
        threads: spec.resources.threads.max(1),
        memory_gb: spec.resources.mem_gb.max(1),
        walltime_minutes: fastq_benchmark_walltime_minutes(stage_id.as_str(), tool_id.as_str()),
        scratch_gb: spec.resources.tmp_gb.max(1),
        resource_origin: FASTQ_STAGE_TOOL_RESOURCE_ORIGIN.to_string(),
    }
}

fn render_bam_resource_row(
    stage_id: &StageId,
    tool_id: &ToolId,
    spec: &ToolExecutionSpecV1,
    input_bytes: u64,
) -> StageToolResourceRow {
    let estimate = estimate_bam_stage_resources_with_origin(
        stage_id.as_str(),
        input_bytes,
        BamInputOriginV1::Synthetic,
    );
    StageToolResourceRow {
        domain: "bam".to_string(),
        stage_id: stage_id.as_str().to_string(),
        tool_id: tool_id.as_str().to_string(),
        threads: spec.resources.threads.max(estimate.cpu_threads).max(1),
        memory_gb: spec.resources.mem_gb.max(estimate.memory_gb).max(1),
        walltime_minutes: estimate.walltime_minutes.max(1),
        scratch_gb: spec.resources.tmp_gb.max(estimate.scratch_gb).max(1),
        resource_origin: BAM_STAGE_TOOL_RESOURCE_ORIGIN.to_string(),
    }
}

fn render_vcf_resource_row(
    stage_id: &StageId,
    tool_id: &ToolId,
    stage: VcfDomainStage,
) -> StageToolResourceRow {
    let resources = vcf_stage_resource_constraints(stage);
    StageToolResourceRow {
        domain: "vcf".to_string(),
        stage_id: stage_id.as_str().to_string(),
        tool_id: tool_id.as_str().to_string(),
        threads: resources.threads.max(1),
        memory_gb: resources.mem_gb.max(1),
        walltime_minutes: vcf_benchmark_walltime_minutes(stage),
        scratch_gb: resources.tmp_gb.max(1),
        resource_origin: VCF_STAGE_TOOL_RESOURCE_ORIGIN.to_string(),
    }
}

fn detect_tiny_bam_resource_input_bytes(repo_root: &Path) -> Result<u64> {
    let mut largest_input_bytes = 0_u64;
    for relative_path in TINY_BAM_RESOURCE_INPUT_PATHS {
        let path = repo_root.join(relative_path);
        let bytes = fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?.len();
        largest_input_bytes = largest_input_bytes.max(bytes);
    }
    if largest_input_bytes == 0 {
        return Err(anyhow!("governed tiny BAM resource fixtures resolved to zero bytes"));
    }
    Ok(largest_input_bytes)
}

fn ensure_unique_stage_tool_rows(rows: &[StageToolResourceRow]) -> Result<()> {
    let mut seen = BTreeSet::<(String, String)>::new();
    for row in rows {
        let pair = (row.stage_id.clone(), row.tool_id.clone());
        if !seen.insert(pair.clone()) {
            return Err(anyhow!(
                "stage-tool resource rows repeat benchmark-ready pair `{}` / `{}`",
                pair.0,
                pair.1
            ));
        }
    }
    Ok(())
}

fn render_stage_tool_resources_toml(config: &StageToolResourcesConfig) -> Result<String> {
    let body = toml::to_string_pretty(config).context("serialize stage-tool resources config")?;
    Ok(format!(
        "# schema_version = 1\n\
         # owner = bijux-dna-bench\n\
         # purpose = Governed resource hints for benchmark-ready FASTQ, BAM, and VCF stage-tool commands.\n\
         # authority = bijux-dna-bench\n\
         # stability = evolving\n\
         # last_updated = 2026-06-25\n\n{body}"
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
    if let Ok(relative) = path.strip_prefix(repo_root) {
        return relative.to_string_lossy().replace('\\', "/");
    }
    path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[cfg(feature = "bam_downstream")]
    use super::{
        render_stage_tool_resources, BAM_STAGE_TOOL_RESOURCE_ORIGIN,
        FASTQ_STAGE_TOOL_RESOURCE_ORIGIN, VCF_STAGE_TOOL_RESOURCE_ORIGIN,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn stage_tool_resources_report_paths_preserve_repository_boundaries() {
        let root = std::path::Path::new("repository");
        for (output, expected) in [
            ("repository/configs/resources.toml", "configs/resources.toml"),
            ("other/resources.toml", "resources.toml"),
            ("repository-other/resources.toml", "resources.toml"),
        ] {
            assert_eq!(super::path_relative_to_repo(root, std::path::Path::new(output)), expected);
        }
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_resources_keeps_representative_resource_bindings() {
        let root = repo_root();
        let artifact_root = root.join("artifacts").join("testing");
        std::fs::create_dir_all(&artifact_root).expect("artifact output root");
        let output = tempfile::Builder::new()
            .prefix("stage-resources-")
            .tempdir_in(&artifact_root)
            .expect("temporary artifact output");
        let output_path = output.path().join("resources.toml");
        let requested_path = output_path.strip_prefix(&root).expect("repository-owned output");
        let report = render_stage_tool_resources(&root, requested_path.to_path_buf())
            .expect("render stage-tool resources");

        assert_eq!(report.schema_version, "bijux.bench.readiness.stage_tool_resources.v1");
        assert_eq!(report.config_path, requested_path.to_string_lossy().replace('\\', "/"));
        assert_eq!(report.classification_scope, "benchmark_ready_command_resources");
        assert_eq!(report.benchmark_ready_row_count, report.row_count);
        assert_eq!(report.nonzero_resource_row_count, report.row_count);
        assert!(
            !report.rows.is_empty(),
            "stage tool resource report should contain benchmark rows"
        );
        assert!(report.rows.iter().all(|row| {
            row.threads > 0 && row.memory_gb > 0 && row.walltime_minutes > 0 && row.scratch_gb > 0
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.profile_read_lengths"
                && row.tool_id == "seqfu"
                && row.resource_origin == FASTQ_STAGE_TOOL_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.resource_origin == BAM_STAGE_TOOL_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.stats"
                && row.tool_id == "bcftools"
                && row.resource_origin == VCF_STAGE_TOOL_RESOURCE_ORIGIN
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_resources_writes_governed_toml_contract() {
        use super::{
            render_stage_tool_resources, StageToolResourcesConfig,
            LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1, STAGE_TOOL_RESOURCES_SCOPE,
        };

        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let output_path = output.path().join("resources.toml");
        let _report = render_stage_tool_resources(&root, output_path.clone())
            .expect("render stage-tool resources");

        let raw = std::fs::read_to_string(&output_path).expect("read rendered config");
        let config: StageToolResourcesConfig =
            toml::from_str(&raw).expect("parse rendered stage-tool resource config");

        assert_eq!(config.schema_version, LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1);
        assert_eq!(config.classification_scope, STAGE_TOOL_RESOURCES_SCOPE);
        assert!(
            !config.rows.is_empty(),
            "rendered stage-tool resource config should contain resource rows"
        );
        assert!(config.rows.iter().all(|row| {
            row.threads > 0 && row.memory_gb > 0 && row.walltime_minutes > 0 && row.scratch_gb > 0
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_polyg_tails"
                && row.tool_id == "fastp"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_terminal_damage"
                && row.tool_id == "cutadapt"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.threads == 1
                && row.memory_gb == 1
                && row.walltime_minutes == 15
                && row.scratch_gb == 1
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.overlap_correction"
                && row.tool_id == "bamutil"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "bcftools"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        for tool_id in ["bedtools", "mosdepth", "samtools"] {
            assert!(config.rows.iter().any(|row| {
                row.stage_id == "bam.coverage"
                    && row.tool_id == tool_id
                    && row.threads == 1
                    && row.memory_gb == 1
                    && row.walltime_minutes == 6
                    && row.scratch_gb == 1
            }));
        }
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.gc_bias"
                && row.tool_id == "picard"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "schmutzi"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "bam.endogenous_content"
                && row.tool_id == "samtools"
                && row.threads == 1
                && row.memory_gb == 1
                && row.walltime_minutes == 5
                && row.scratch_gb == 1
        }));
        for tool_id in ["picard", "samtools"] {
            assert!(config.rows.iter().any(|row| {
                row.stage_id == "bam.mapping_summary"
                    && row.tool_id == tool_id
                    && row.threads == 3
                    && row.memory_gb == 2
                    && row.walltime_minutes == 7
                    && row.scratch_gb == 2
            }));
        }
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.tool_id == "bcftools"
                && row.threads == 2
                && row.memory_gb == 4
                && row.walltime_minutes == 20
                && row.scratch_gb == 8
                && row.resource_origin == super::VCF_STAGE_TOOL_RESOURCE_ORIGIN
        }));
    }
}
