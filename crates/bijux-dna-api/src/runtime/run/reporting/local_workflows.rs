use super::Result;
use crate::request_args::{ExecuteRequest, ExecuteResponse};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runtime::run_layout::RunExecutionModeV1;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Execute a tiny local FASTQ workflow end-to-end.
///
/// # Errors
/// Returns an error if graph construction or execution fails.
pub fn execute_local_fastq_workflow(run_dir: &Path) -> Result<ExecuteResponse> {
    bijux_dna_infra::ensure_dir(run_dir)?;
    let input_fastq = run_dir.join("inputs").join("reads.fastq");
    write_text(
        &input_fastq,
        "@r1\nACGT\n+\n!!!!\n@r2\nTGCA\n+\n####\n",
    )?;
    let out = run_dir.join("out");
    let validated = out.join("validated.fastq");
    let filtered = out.join("filtered.fastq");
    let report = out.join("report_qc.json");

    let validate = step(
        "fastq.validate_reads",
        "fastq.validate_reads",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("set -eu; cat '{}' > '{}'", input_fastq.display(), validated.display()),
        ],
        vec![artifact_required("reads", input_fastq, ArtifactRole::Reads)],
        vec![artifact_required("validated", validated.clone(), ArtifactRole::Reads)],
        out.join("validate_reads"),
    );
    let filter = step(
        "fastq.filter_reads",
        "fastq.filter_reads",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("set -eu; cat '{}' > '{}'", validated.display(), filtered.display()),
        ],
        vec![artifact_required("validated", validated, ArtifactRole::Reads)],
        vec![artifact_required("filtered", filtered.clone(), ArtifactRole::Reads)],
        out.join("filter_reads"),
    );
    let report_qc = step(
        "fastq.report_qc",
        "fastq.report_qc",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "set -eu; printf '%s\\n' '{{\"schema_version\":\"bijux.fastq.report_qc.v1\",\"reads_in\":2,\"reads_out\":2}}' > '{}'",
                report.display()
            ),
        ],
        vec![artifact_required("filtered", filtered, ArtifactRole::Reads)],
        vec![artifact_required("report", report, ArtifactRole::ReportJson)],
        out.join("report_qc"),
    );
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__local_workflow__v1",
        "api.local.workflow",
        PlanPolicy::PreferAccuracy,
        vec![validate, filter, report_qc],
        vec![
            ExecutionEdge::new(StepId::new("fastq.validate_reads"), StepId::new("fastq.filter_reads")),
            ExecutionEdge::new(StepId::new("fastq.filter_reads"), StepId::new("fastq.report_qc")),
        ],
    )?;
    execute_graph_local(run_dir, graph)
}

/// Execute a tiny local BAM workflow end-to-end.
///
/// # Errors
/// Returns an error if graph construction or execution fails.
pub fn execute_local_bam_workflow(run_dir: &Path) -> Result<ExecuteResponse> {
    bijux_dna_infra::ensure_dir(run_dir)?;
    let input_bam = run_dir.join("inputs").join("reads.bam");
    write_text(&input_bam, "tiny-bam-placeholder\n")?;
    let out = run_dir.join("out");
    let validated = out.join("validated.bam");
    let mapping = out.join("mapping_summary.json");
    let coverage = out.join("coverage_summary.json");

    let validate = step(
        "bam.validate",
        "bam.validate",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("set -eu; cat '{}' > '{}'", input_bam.display(), validated.display()),
        ],
        vec![artifact_required("reads_bam", input_bam, ArtifactRole::Bam)],
        vec![artifact_required("validated_bam", validated.clone(), ArtifactRole::Bam)],
        out.join("validate"),
    );
    let mapping_summary = step(
        "bam.mapping_summary",
        "bam.mapping_summary",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "set -eu; printf '%s\\n' '{{\"schema_version\":\"bijux.bam.mapping_summary.v1\",\"mapped\":1}}' > '{}'",
                mapping.display()
            ),
        ],
        vec![artifact_required("validated_bam", validated, ArtifactRole::Bam)],
        vec![artifact_required("mapping_summary", mapping.clone(), ArtifactRole::ReportJson)],
        out.join("mapping_summary"),
    );
    let coverage_summary = step(
        "bam.coverage",
        "bam.coverage",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "set -eu; printf '%s\\n' '{{\"schema_version\":\"bijux.bam.coverage_summary.v1\",\"mean_depth\":1.0}}' > '{}'",
                coverage.display()
            ),
        ],
        vec![artifact_required("validated_bam", out.join("validated.bam"), ArtifactRole::Bam)],
        vec![artifact_required("coverage_summary", coverage, ArtifactRole::ReportJson)],
        out.join("coverage"),
    );
    let graph = ExecutionGraph::new(
        "bam-to-bam__local_workflow__v1",
        "api.local.workflow",
        PlanPolicy::PreferAccuracy,
        vec![validate, mapping_summary, coverage_summary],
        vec![
            ExecutionEdge::new(StepId::new("bam.validate"), StepId::new("bam.mapping_summary")),
            ExecutionEdge::new(StepId::new("bam.validate"), StepId::new("bam.coverage")),
        ],
    )?;
    execute_graph_local(run_dir, graph)
}

/// Execute a tiny local VCF workflow end-to-end.
///
/// # Errors
/// Returns an error if graph construction or execution fails.
pub fn execute_local_vcf_workflow(run_dir: &Path) -> Result<ExecuteResponse> {
    bijux_dna_infra::ensure_dir(run_dir)?;
    let input_vcf = run_dir.join("inputs").join("variants.vcf");
    write_text(
        &input_vcf,
        "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n1\t1\t.\tA\tG\t60\tPASS\t.\n",
    )?;
    let out = run_dir.join("out");
    let validated = out.join("validated.vcf");
    let normalized = out.join("normalized.vcf");
    let filtered = out.join("filtered.vcf");
    let stats = out.join("stats.json");
    let report = out.join("report.json");

    let validate = step(
        "vcf.validate",
        "vcf.validate",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("set -eu; cat '{}' > '{}'", input_vcf.display(), validated.display()),
        ],
        vec![artifact_required("variants", input_vcf, ArtifactRole::Variant)],
        vec![artifact_required("validated", validated.clone(), ArtifactRole::Variant)],
        out.join("validate"),
    );
    let normalize = step(
        "vcf.normalize",
        "vcf.normalize",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("set -eu; cat '{}' > '{}'", validated.display(), normalized.display()),
        ],
        vec![artifact_required("validated", validated, ArtifactRole::Variant)],
        vec![artifact_required("normalized", normalized.clone(), ArtifactRole::Variant)],
        out.join("normalize"),
    );
    let filter = step(
        "vcf.filter",
        "vcf.filter",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("set -eu; cat '{}' > '{}'", normalized.display(), filtered.display()),
        ],
        vec![artifact_required("normalized", normalized, ArtifactRole::Variant)],
        vec![artifact_required("filtered", filtered.clone(), ArtifactRole::Variant)],
        out.join("filter"),
    );
    let stats_step = step(
        "vcf.stats",
        "vcf.stats",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "set -eu; printf '%s\\n' '{{\"schema_version\":\"bijux.vcf.stats.v1\",\"variants\":1}}' > '{}'",
                stats.display()
            ),
        ],
        vec![artifact_required("filtered", filtered, ArtifactRole::Variant)],
        vec![artifact_required("stats", stats, ArtifactRole::ReportJson)],
        out.join("stats"),
    );
    let report_step = step(
        "vcf.report",
        "vcf.report",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "set -eu; printf '%s\\n' '{{\"schema_version\":\"bijux.vcf.report.v1\",\"status\":\"ok\"}}' > '{}'",
                report.display()
            ),
        ],
        vec![
            artifact_required("stats", out.join("stats.json"), ArtifactRole::ReportJson),
            artifact_required("filtered", out.join("filtered.vcf"), ArtifactRole::Variant),
        ],
        vec![artifact_required("report", report, ArtifactRole::ReportJson)],
        out.join("report"),
    );
    let graph = ExecutionGraph::new(
        "vcf-to-vcf__local_workflow__v1",
        "api.local.workflow",
        PlanPolicy::PreferAccuracy,
        vec![validate, normalize, filter, stats_step, report_step],
        vec![
            ExecutionEdge::new(StepId::new("vcf.validate"), StepId::new("vcf.normalize")),
            ExecutionEdge::new(StepId::new("vcf.normalize"), StepId::new("vcf.filter")),
            ExecutionEdge::new(StepId::new("vcf.filter"), StepId::new("vcf.stats")),
            ExecutionEdge::new(StepId::new("vcf.stats"), StepId::new("vcf.report")),
        ],
    )?;
    execute_graph_local(run_dir, graph)
}

fn execute_graph_local(base_dir: &Path, graph: ExecutionGraph) -> Result<ExecuteResponse> {
    let run_dir = base_dir.join("run");
    super::execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir,
        mode: RunExecutionModeV1::Enforced,
    })
}

fn step(
    step_id: &str,
    stage_id: &str,
    command: Vec<String>,
    inputs: Vec<ArtifactSpec>,
    outputs: Vec<ArtifactSpec>,
    out_dir: PathBuf,
) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::new(step_id),
        stage_id: StageId::new(stage_id),
        command: CommandSpecV1 { template: command },
        image: ContainerImageRefV1 {
            image: format!("example/{stage_id}:1"),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO { inputs, outputs },
        out_dir,
        aux_images: BTreeMap::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

fn artifact_required(id: &str, path: PathBuf, role: ArtifactRole) -> ArtifactSpec {
    ArtifactSpec::required(ArtifactId::new(id), path, role)
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::write_bytes(path, content.as_bytes())?;
    Ok(())
}
