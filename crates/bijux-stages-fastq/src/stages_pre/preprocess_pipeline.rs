use bijux_core::{ContainerImageRefV1, StagePlanV1, ToolExecutionSpecV1};

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn plan_preprocess_pipeline<F>(
    stages: &[String],
    tools: &[ToolExecutionSpecV1],
    aux_images: &std::collections::BTreeMap<String, ContainerImageRefV1>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    mut out_dir_for_stage: F,
) -> anyhow::Result<Vec<StagePlanV1>>
where
    F: FnMut(
        &str,
        &ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> anyhow::Result<std::path::PathBuf>,
{
    if stages.len() != tools.len() {
        return Err(anyhow::anyhow!(
            "pipeline stages/tools length mismatch: {} vs {}",
            stages.len(),
            tools.len()
        ));
    }
    let mut current_r1 = r1.to_path_buf();
    let raw_r1 = r1.to_path_buf();
    let mut current_r2 = r2.map(|path| path.to_path_buf());
    let mut plans = Vec::new();
    for (stage, tool) in stages.iter().zip(tools.iter()) {
        let out_dir = out_dir_for_stage(stage, tool, &current_r1, current_r2.as_deref())?;
        let stage_id: &str = stage;
        let (plan, next_r1, next_r2) = match stage_id {
            "fastq.detect_adapters" => {
                let plan = crate::stages_pre::detect_adapters::plan(tool, &current_r1, &out_dir);
                (plan, current_r1.clone(), current_r2.clone())
            }
            "fastq.trim" => {
                let plan = crate::stages_transform::trim::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                    adapter_bank,
                    polyx_bank,
                    contaminant_bank,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            "fastq.filter" => {
                let mut filter_options = crate::stages_transform::filter::FilterPlanOptions::default();
                if adapter_bank.is_some() {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some() {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal && contaminant_bank.is_some() {
                    filter_options.kmer_ref = crate::stages_transform::filter::default_kmer_ref();
                }
                let plan = crate::stages_transform::filter::plan_filter(
                    tool,
                    &current_r1,
                    &out_dir,
                    &filter_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            "fastq.validate_pre" => {
                let plan = crate::stages_pre::validate_pre::plan(tool, &current_r1, &out_dir);
                (plan, current_r1.clone(), current_r2.clone())
            }
            "fastq.merge" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("merge requires r2"))?;
                let plan = crate::stages_transform::merge::plan_merge(tool, &current_r1, r2, &out_dir)?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            "fastq.correct" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("correct requires r2"))?;
                let plan = crate::stages_transform::correct::plan_correct(tool, &current_r1, r2, &out_dir)?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (
                    plan,
                    next_r1,
                    Some(next_r2),
                )
            }
            "fastq.umi" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("umi requires r2"))?;
                let plan = crate::stages_transform::umi::plan_umi(tool, &current_r1, r2, &out_dir)?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (
                    plan,
                    next_r1,
                    Some(next_r2),
                )
            }
            "fastq.qc_post" => {
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in crate::stages_qc::qc_post::aux_tool_ids() {
                        if let Some(image) = aux_images.get(*aux_tool) {
                            stage_aux_images.insert(aux_tool.to_string(), image.clone());
                        }
                    }
                }
                let plan = crate::stages_qc::qc_post::plan_qc_post(
                    tool,
                    &current_r1,
                    &out_dir,
                    stage_aux_images,
                    Some(raw_r1.as_path()),
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            "fastq.screen" => {
                let plan = crate::stages_qc::screen::plan_screen(tool, &current_r1, &out_dir)?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            "fastq.stats_neutral" => {
                let plan =
                    crate::stages_qc::stats_neutral::plan_stats_neutral(tool, &current_r1, &out_dir)?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            _ => {
                return Err(anyhow::anyhow!("unsupported stage in fastq pipeline: {stage}"));
            }
        };
        plans.push(plan);
        current_r1 = next_r1;
        current_r2 = next_r2;
    }
    Ok(plans)
}
