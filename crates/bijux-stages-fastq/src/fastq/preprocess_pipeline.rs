use bijux_core::ContainerImageRefV1;

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
        let (plan, next_r1, next_r2, stage_version) = match stage.as_str() {
            "fastq.detect_adapters" => {
                let plan = crate::fastq::detect_adapters::plan(tool, &current_r1, &out_dir);
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::detect_adapters::STAGE_VERSION,
                )
            }
            "fastq.trim" => {
                let plan = crate::fastq::trim::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                    adapter_bank,
                    polyx_bank,
                    contaminant_bank,
                )?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    None,
                    crate::fastq::trim::STAGE_VERSION,
                )
            }
            "fastq.filter" => {
                let mut filter_options = crate::fastq::filter::FilterPlanOptions::default();
                if adapter_bank.is_some() {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some() {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal && contaminant_bank.is_some() {
                    filter_options.kmer_ref = crate::fastq::filter::default_kmer_ref();
                }
                let plan = crate::fastq::filter::plan_filter(
                    tool,
                    &current_r1,
                    &out_dir,
                    &filter_options,
                )?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    None,
                    crate::fastq::filter::STAGE_VERSION,
                )
            }
            "fastq.validate_pre" => {
                let plan = crate::fastq::validate_pre::plan(tool, &current_r1, &out_dir);
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::validate_pre::STAGE_VERSION,
                )
            }
            "fastq.merge" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("merge requires r2"))?;
                let plan = crate::fastq::merge::plan_merge(tool, &current_r1, r2, &out_dir)?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    None,
                    crate::fastq::merge::STAGE_VERSION,
                )
            }
            "fastq.correct" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("correct requires r2"))?;
                let plan = crate::fastq::correct::plan_correct(tool, &current_r1, r2, &out_dir)?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    Some(plan.io.outputs[1].path.clone()),
                    crate::fastq::correct::STAGE_VERSION,
                )
            }
            "fastq.umi" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("umi requires r2"))?;
                let plan = crate::fastq::umi::plan_umi(tool, &current_r1, r2, &out_dir)?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    Some(plan.io.outputs[1].path.clone()),
                    crate::fastq::umi::STAGE_VERSION,
                )
            }
            "fastq.qc_post" => {
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in crate::fastq::qc_post::aux_tool_ids() {
                        if let Some(image) = aux_images.get(*aux_tool) {
                            stage_aux_images.insert(aux_tool.to_string(), image.clone());
                        }
                    }
                }
                let plan = crate::fastq::qc_post::plan_qc_post(
                    tool,
                    &current_r1,
                    &out_dir,
                    stage_aux_images,
                    Some(raw_r1.as_path()),
                )?;
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::qc_post::STAGE_VERSION,
                )
            }
            "fastq.screen" => {
                let plan = crate::fastq::screen::plan_screen(tool, &current_r1, &out_dir)?;
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::screen::STAGE_VERSION,
                )
            }
            "fastq.stats_neutral" => {
                let plan =
                    crate::fastq::stats_neutral::plan_stats_neutral(tool, &current_r1, &out_dir)?;
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::stats_neutral::STAGE_VERSION,
                )
            }
            _ => return Err(anyhow::anyhow!("unsupported stage {stage}")),
        };
        let mut exec_plan = plan;
        exec_plan.stage_id = StageId(stage.clone());
        exec_plan.stage_version = stage_version;
        exec_plan.out_dir = out_dir;
        plans.push(exec_plan);
        current_r1 = next_r1;
        current_r2 = next_r2;
    }
    Ok(plans)
}
