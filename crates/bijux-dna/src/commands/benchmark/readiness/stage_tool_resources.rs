use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ToolExecutionSpecV1;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{estimate_bam_stage_resources_with_origin, BamInputOriginV1};
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_planner_bam::stage_api::load_bam_domain_tool_execution_spec;
use bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_execution_spec;
use bijux_dna_planner_vcf::vcf_stage_resource_constraints;
use serde::{Deserialize, Serialize};

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_TOOL_RESOURCES_PATH: &str =
    "benchmarks/configs/local/stage-tool-resources.toml";
pub(crate) const LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_tool_resources.v1";
const STAGE_TOOL_RESOURCES_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.stage_tool_resources.v1";
const STAGE_TOOL_RESOURCES_SCOPE: &str = "benchmark_ready_command_resources";
const FASTQ_RESOURCE_ORIGIN: &str = "tool_constraints_with_stage_walltime_profile";
const BAM_RESOURCE_ORIGIN: &str = "tool_constraints_with_tiny_bam_stage_estimate";
const VCF_RESOURCE_ORIGIN: &str = "planner_stage_constraints_with_stage_walltime_profile";
const FASTQ_MINUTES_QC: u32 = 10;
const FASTQ_MINUTES_TRANSFORM: u32 = 15;
const FASTQ_MINUTES_REFERENCE: u32 = 20;
const FASTQ_MINUTES_HEAVY_ANALYSIS: u32 = 30;
const FASTQ_MINUTES_DADA2: u32 = 45;
const VCF_MINUTES_STANDARD: u32 = 20;
const VCF_MINUTES_PREPARE_PANEL: u32 = 30;
const VCF_MINUTES_PANEL_WORKFLOW: u32 = 60;
const TINY_BAM_RESOURCE_INPUT_PATHS: &[&str] = &[
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam",
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/adna_like_damage.sam",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageToolResourcesConfig {
    pub(crate) schema_version: String,
    pub(crate) classification_scope: String,
    pub(crate) rows: Vec<StageToolResourceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageToolResourceRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) threads: u32,
    pub(crate) memory_gb: u32,
    pub(crate) walltime_minutes: u32,
    pub(crate) scratch_gb: u32,
    pub(crate) resource_origin: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageToolResourcesReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) classification_scope: &'static str,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) nonzero_resource_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) resource_origin_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<StageToolResourceRow>,
}

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
        schema_version: LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION.to_string(),
        classification_scope: STAGE_TOOL_RESOURCES_SCOPE.to_string(),
        rows: rows.clone(),
    };
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

    Ok(StageToolResourcesReport {
        schema_version: STAGE_TOOL_RESOURCES_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, &output_path),
        classification_scope: STAGE_TOOL_RESOURCES_SCOPE,
        row_count: rows.len(),
        benchmark_ready_row_count: rows.len(),
        nonzero_resource_row_count,
        domain_counts,
        resource_origin_counts,
        rows,
    })
}

fn collect_stage_tool_resource_rows(repo_root: &Path) -> Result<Vec<StageToolResourceRow>> {
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let vcf_rows = collect_vcf_expected_benchmark_result_rows(repo_root)?;
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
        walltime_minutes: fastq_walltime_minutes(stage_id.as_str(), tool_id.as_str()),
        scratch_gb: spec.resources.tmp_gb.max(1),
        resource_origin: FASTQ_RESOURCE_ORIGIN.to_string(),
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
        resource_origin: BAM_RESOURCE_ORIGIN.to_string(),
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
        walltime_minutes: vcf_walltime_minutes(stage),
        scratch_gb: resources.tmp_gb.max(1),
        resource_origin: VCF_RESOURCE_ORIGIN.to_string(),
    }
}

fn fastq_walltime_minutes(stage_id: &str, tool_id: &str) -> u32 {
    match (stage_id, tool_id) {
        ("fastq.infer_asvs", "dada2") => FASTQ_MINUTES_DADA2,
        ("fastq.correct_errors", _) | ("fastq.cluster_otus", _) | ("fastq.remove_chimeras", _) => {
            FASTQ_MINUTES_HEAVY_ANALYSIS
        }
        ("fastq.deplete_host", _)
        | ("fastq.deplete_reference_contaminants", _)
        | ("fastq.deplete_rrna", _)
        | ("fastq.index_reference", _)
        | ("fastq.screen_taxonomy", _) => FASTQ_MINUTES_REFERENCE,
        ("fastq.merge_pairs", _) | ("fastq.normalize_primers", _) | ("fastq.trim_reads", _) => {
            FASTQ_MINUTES_TRANSFORM
        }
        ("fastq.profile_read_lengths", _)
        | ("fastq.profile_reads", _)
        | ("fastq.validate_reads", _) => FASTQ_MINUTES_QC,
        _ => FASTQ_MINUTES_TRANSFORM,
    }
}

fn vcf_walltime_minutes(stage: VcfDomainStage) -> u32 {
    match stage {
        VcfDomainStage::Phasing | VcfDomainStage::Impute | VcfDomainStage::ImputationMetrics => {
            VCF_MINUTES_PANEL_WORKFLOW
        }
        VcfDomainStage::PrepareReferencePanel => VCF_MINUTES_PREPARE_PANEL,
        _ => VCF_MINUTES_STANDARD,
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
         # stability = evolving\n\n{body}"
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
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_resources_reports_governed_benchmark_ready_row_slice() {
        use super::{
            render_stage_tool_resources, BAM_RESOURCE_ORIGIN, DEFAULT_STAGE_TOOL_RESOURCES_PATH,
            FASTQ_RESOURCE_ORIGIN,
        };

        let root = repo_root();
        let report =
            render_stage_tool_resources(&root, PathBuf::from(DEFAULT_STAGE_TOOL_RESOURCES_PATH))
                .expect("render stage-tool resources");

        assert_eq!(report.schema_version, "bijux.bench.readiness.stage_tool_resources.v1");
        assert_eq!(report.config_path, "benchmarks/configs/local/stage-tool-resources.toml");
        assert_eq!(report.classification_scope, "benchmark_ready_command_resources");
        assert_eq!(report.benchmark_ready_row_count, report.row_count);
        assert_eq!(report.nonzero_resource_row_count, report.row_count);
        assert_eq!(report.domain_counts.get("fastq"), Some(&63));
        assert_eq!(report.domain_counts.get("bam"), Some(&49));
        assert_eq!(report.domain_counts.get("vcf"), Some(&19));
        assert!(report.rows.iter().all(|row| {
            row.threads > 0 && row.memory_gb > 0 && row.walltime_minutes > 0 && row.scratch_gb > 0
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.profile_read_lengths"
                && row.tool_id == "seqfu"
                && row.resource_origin == FASTQ_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.prepare_reference_panel"
                && row.tool_id == "bcftools"
                && row.threads == 2
                && row.memory_gb == 4
                && row.walltime_minutes == 30
                && row.scratch_gb == 8
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
                && row.resource_origin == FASTQ_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.filter_reads"
                && row.tool_id == "fastp"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
                && row.resource_origin == FASTQ_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bowtie2"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.overlap_correction"
                && row.tool_id == "bamutil"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.damage"
                && row.tool_id == "ngsbriggs"
                && row.threads == 1
                && row.memory_gb == 1
                && row.walltime_minutes == 8
                && row.scratch_gb == 1
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.bias_mitigation"
                && row.tool_id == "mapdamage2"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "angsd"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.tool_id == "yleaf"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_polyg_tails"
                && row.tool_id == "fastp"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
                && row.resource_origin == FASTQ_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_terminal_damage"
                && row.tool_id == "cutadapt"
                && row.threads == 4
                && row.memory_gb == 8
                && row.walltime_minutes == 15
                && row.scratch_gb == 4
                && row.resource_origin == FASTQ_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.threads == 1
                && row.memory_gb == 1
                && row.walltime_minutes == 15
                && row.scratch_gb == 1
                && row.resource_origin == FASTQ_RESOURCE_ORIGIN
        }));
        for tool_id in ["bedtools", "mosdepth", "samtools"] {
            assert!(report.rows.iter().any(|row| {
                row.stage_id == "bam.coverage"
                    && row.tool_id == tool_id
                    && row.threads == 1
                    && row.memory_gb == 1
                    && row.walltime_minutes == 6
                    && row.scratch_gb == 1
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.gc_bias"
                && row.tool_id == "picard"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "schmutzi"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
                && row.resource_origin == BAM_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.endogenous_content"
                && row.tool_id == "samtools"
                && row.threads == 1
                && row.memory_gb == 1
                && row.walltime_minutes == 5
                && row.scratch_gb == 1
        }));
        for tool_id in ["picard", "samtools"] {
            assert!(report.rows.iter().any(|row| {
                row.stage_id == "bam.mapping_summary"
                    && row.tool_id == tool_id
                    && row.threads == 3
                    && row.memory_gb == 2
                    && row.walltime_minutes == 7
                    && row.scratch_gb == 2
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.insert_size"
                && row.tool_id == "picard"
                && row.threads == 3
                && row.memory_gb == 2
                && row.walltime_minutes == 7
                && row.scratch_gb == 2
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.tool_id == "bcftools"
                && row.threads == 2
                && row.memory_gb == 4
                && row.walltime_minutes == 20
                && row.scratch_gb == 8
                && row.resource_origin == super::VCF_RESOURCE_ORIGIN
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.stats"
                && row.tool_id == "bcftools"
                && row.threads == 2
                && row.memory_gb == 4
                && row.walltime_minutes == 20
                && row.scratch_gb == 8
                && row.resource_origin == super::VCF_RESOURCE_ORIGIN
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_resources_writes_governed_toml_contract() {
        use super::{
            render_stage_tool_resources, StageToolResourcesConfig,
            DEFAULT_STAGE_TOOL_RESOURCES_PATH, LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION,
            STAGE_TOOL_RESOURCES_SCOPE,
        };

        let root = repo_root();
        let output_path = root.join(DEFAULT_STAGE_TOOL_RESOURCES_PATH);
        let _report =
            render_stage_tool_resources(&root, PathBuf::from(DEFAULT_STAGE_TOOL_RESOURCES_PATH))
                .expect("render stage-tool resources");

        let raw = std::fs::read_to_string(&output_path).expect("read rendered config");
        let config: StageToolResourcesConfig =
            toml::from_str(&raw).expect("parse rendered stage-tool resource config");

        assert_eq!(config.schema_version, LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION);
        assert_eq!(config.classification_scope, STAGE_TOOL_RESOURCES_SCOPE);
        assert_eq!(config.rows.len(), 120);
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
                && row.tool_id == "angsd"
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
                && row.resource_origin == super::VCF_RESOURCE_ORIGIN
        }));
    }
}
