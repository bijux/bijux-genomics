//! Stage planning and contract access.

pub mod artifacts;
pub mod fastq;
pub mod fastq_tools_registry;
mod plan;
pub mod stages_pre;
pub mod stages_qc;
pub mod stages_transform;

pub use bijux_core::{ArtifactRef, StageIO, StagePlanV1};
pub use plan::StagePlanJson;

pub use bijux_domain_fastq as domain_fastq;
pub use bijux_domain_fastq::*;

pub const TOOL_SEQKIT: &str = "seqkit";

pub mod contracts {
    pub use bijux_domain_fastq::contract_for_stage;
    pub use bijux_domain_fastq::FastqStageContract as StageContract;
}

pub struct StagePlanRequest<'a> {
    pub stage_id: &'a str,
    pub tool: &'a bijux_core::ToolExecutionSpecV1,
    pub r1: Option<&'a std::path::Path>,
    pub r2: Option<&'a std::path::Path>,
    pub out_dir: &'a std::path::Path,
    pub adapter_bank: Option<&'a serde_json::Value>,
    pub polyx_bank: Option<&'a serde_json::Value>,
    pub contaminant_bank: Option<&'a serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub aux_images: &'a std::collections::BTreeMap<String, bijux_core::ContainerImageRefV1>,
    pub raw_r1: Option<&'a std::path::Path>,
    pub pipeline: Option<&'a bijux_core::domain::PipelineSpec>,
}

/// # Errors
/// Returns an error if the stage cannot be planned with the provided inputs.
pub fn plan_stage(request: StagePlanRequest<'_>) -> anyhow::Result<StagePlanV1> {
    match request.stage_id {
        "fastq.validate_pre" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("validate_pre requires r1"))?;
            Ok(stages_pre::validate_pre::plan(request.tool, r1, request.out_dir))
        }
        "fastq.detect_adapters" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("detect_adapters requires r1"))?;
            Ok(stages_pre::detect_adapters::plan(request.tool, r1, request.out_dir))
        }
        "fastq.trim" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("trim requires r1"))?;
            stages_transform::trim::plan(
                request.tool,
                r1,
                request.out_dir,
                request.adapter_bank,
                request.polyx_bank,
                request.contaminant_bank,
            )
        }
        "fastq.filter" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("filter requires r1"))?;
            let mut options = stages_transform::filter::FilterPlanOptions::default();
            if request.enable_contaminant_removal && request.contaminant_bank.is_some() {
                options.kmer_ref = stages_transform::filter::default_kmer_ref();
            }
            stages_transform::filter::plan_filter(request.tool, r1, request.out_dir, &options)
        }
        "fastq.merge" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("merge requires r1"))?;
            let r2 = request.r2.ok_or_else(|| anyhow::anyhow!("merge requires r2"))?;
            stages_transform::merge::plan_merge(request.tool, r1, r2, request.out_dir)
        }
        "fastq.correct" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("correct requires r1"))?;
            let r2 = request.r2.ok_or_else(|| anyhow::anyhow!("correct requires r2"))?;
            stages_transform::correct::plan_correct(request.tool, r1, r2, request.out_dir)
        }
        "fastq.umi" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("umi requires r1"))?;
            let r2 = request.r2.ok_or_else(|| anyhow::anyhow!("umi requires r2"))?;
            stages_transform::umi::plan_umi(request.tool, r1, r2, request.out_dir)
        }
        "fastq.qc_post" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("qc_post requires r1"))?;
            stages_qc::qc_post::plan_qc_post(
                request.tool,
                r1,
                request.out_dir,
                request.aux_images.clone(),
                request.raw_r1,
            )
        }
        "fastq.screen" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("screen requires r1"))?;
            stages_qc::screen::plan_screen(request.tool, r1, request.out_dir)
        }
        "fastq.stats_neutral" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("stats_neutral requires r1"))?;
            stages_qc::stats_neutral::plan_stats_neutral(request.tool, r1, request.out_dir)
        }
        "fastq.preprocess" => {
            let r1 = request.r1.ok_or_else(|| anyhow::anyhow!("preprocess requires r1"))?;
            let pipeline = request
                .pipeline
                .ok_or_else(|| anyhow::anyhow!("preprocess requires pipeline spec"))?;
            let plan = stages_pre::preprocess_plan::PreprocessPlan {
                r1: r1.to_path_buf(),
                r2: request.r2.map(|path| path.to_path_buf()),
                pipeline: pipeline.clone(),
                merge_decision: None,
                correct_decision: None,
                enable_contaminant_removal: request.enable_contaminant_removal,
            };
            Ok(stages_pre::preprocess_plan::plan_preprocess_stage(
                &plan,
                request.tool,
            ))
        }
        _ => Err(anyhow::anyhow!(
            "unsupported fastq stage for planner: {}",
            request.stage_id
        )),
    }
}

/// # Errors
/// Returns an error if the pipeline cannot be planned with the provided inputs.
pub fn plan_pipeline<F>(
    stages: &[String],
    tools: &[bijux_core::ToolExecutionSpecV1],
    aux_images: &std::collections::BTreeMap<String, bijux_core::ContainerImageRefV1>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    out_dir_for_stage: F,
) -> anyhow::Result<Vec<StagePlanV1>>
where
    F: FnMut(
        &str,
        &bijux_core::ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> anyhow::Result<std::path::PathBuf>,
{
    stages_pre::preprocess_pipeline::plan_preprocess_pipeline(
        stages,
        tools,
        aux_images,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        enable_contaminant_removal,
        r1,
        r2,
        out_dir_for_stage,
    )
}
