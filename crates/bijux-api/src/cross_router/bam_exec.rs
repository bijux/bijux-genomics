use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_core::alignment::AlignmentBoundary;
use bijux_core::ToolRegistry;
use bijux_runner_docker::primitives::{build_tool_execution_spec, execute_stage_plan};
use bijux_env_runtime::ReferenceRecord;
use bijux_pipelines::PipelineProfile;

use crate::args::{BamRunArgs, FastqCrossArgs};
use crate::bam_plan::plan_for_bam_stage_with_profile;
use crate::bam_support::downstream_enabled;
use crate::fastq_router::StageExecutionSummary;

pub fn run_bam_truth_stages<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_engine::primitives::ToolImageSpec, S>,
    platform: &bijux_engine::primitives::PlatformSpec,
    profile: &PipelineProfile,
    boundary: &AlignmentBoundary,
    out_dir: &Path,
) -> Result<Vec<StageExecutionSummary>> {
    let bam_path = PathBuf::from(&boundary.bam_path);
    let bai_path = boundary.bai_path.as_ref().map(PathBuf::from);
    let reference = boundary.reference.as_ref().map(PathBuf::from);

    let mut runs = Vec::new();
    for node in &profile.graph {
        let stage = bijux_domain_bam::BamStage::try_from(node.stage_id.as_str())?;
        if stage == bijux_domain_bam::BamStage::Align {
            continue;
        }
        if !downstream_enabled()
            && matches!(
                stage,
                bijux_domain_bam::BamStage::Haplogroups
                    | bijux_domain_bam::BamStage::Genotyping
                    | bijux_domain_bam::BamStage::Kinship
            )
        {
            continue;
        }
        let tool_id = profile
            .defaults
            .tools
            .get(stage.as_str())
            .cloned()
            .unwrap_or_else(|| bijux_domain_bam::stage_spec(stage).default_tool.to_string());
        let spec =
            build_tool_execution_spec(stage.as_str(), &tool_id, registry_core, catalog, platform)?;

        let stage_dir = out_dir
            .join("bam")
            .join(stage.as_str().trim_start_matches("bam."));
        bijux_infra::ensure_dir(&stage_dir).context("create bam stage dir")?;

        let args = BamRunArgs {
            stage,
            profile: profile.id.to_string(),
            sample_id: None,
            r1: None,
            r2: None,
            bam: bam_path.clone(),
            out: stage_dir.clone(),
            tool: Some(tool_id),
            dry_run: false,
            bai: bai_path.clone(),
            reference: reference.clone(),
            regions: None,
            udg_model: None,
            pmd_threshold_5p: None,
            pmd_threshold_3p: None,
            trim_5p: None,
            trim_3p: None,
            contamination_scope: None,
            contamination_panel: Vec::new(),
            contamination_prior: None,
            sex_specific_contamination: false,
            contamination_assumptions: None,
            expected_sex: None,
            sex_method: "rxy".to_string(),
            min_mapq: None,
            min_length: None,
            include_flags: Vec::new(),
            exclude_flags: Vec::new(),
            remove_duplicates: false,
            base_quality_threshold: None,
            optical_duplicates: None,
            umi_policy: None,
            duplicate_action: None,
            complexity_min_reads: None,
            complexity_projection_points: Vec::new(),
            depth_thresholds: Vec::new(),
            bqsr_mode: None,
            known_sites: Vec::new(),
            bqsr_min_mean_coverage: None,
            bqsr_min_breadth_1x: None,
            haplogroup_panel: None,
            haplogroup_min_coverage: None,
            kinship_panel: None,
            min_overlap_snps: None,
            caller: None,
            min_posterior: None,
            min_call_rate: None,
            gc_bias_correction: false,
            map_bias_correction: false,
            authenticity_mode: None,
            aligner_preset: None,
            rg_id: None,
            rg_sm: None,
            rg_pl: None,
            rg_lb: None,
            rg_policy: None,
            build_reference_indices: false,
            params_json: None,
        };

        let plan = plan_for_bam_stage_with_profile(stage, &spec, &args, profile, &stage_dir)?;
        let result = execute_stage_plan(&plan, platform.runner, None)?;
        runs.push(StageExecutionSummary { plan, result });
    }

    Ok(runs)
}

#[allow(clippy::too_many_lines)]
pub fn run_bam_align_and_truth_stages<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_engine::primitives::ToolImageSpec, S>,
    platform: &bijux_engine::primitives::PlatformSpec,
    profile: &PipelineProfile,
    reference: &ReferenceRecord,
    args: &FastqCrossArgs,
    out_dir: &Path,
) -> Result<Vec<StageExecutionSummary>> {
    let r1 = args
        .r1
        .as_ref()
        .ok_or_else(|| anyhow!("--r1 required for cross align"))?;
    let sample_id = args.sample_id.as_deref().unwrap_or("sample").to_string();
    let align_out = out_dir.join("bam").join("align");
    bijux_infra::ensure_dir(&align_out)?;
    let tool_id = profile
        .defaults
        .tools
        .get("bam.align")
        .cloned()
        .unwrap_or_else(|| "bwa".to_string());
    let spec = build_tool_execution_spec("bam.align", &tool_id, registry_core, catalog, platform)?;
    let align_args = BamRunArgs {
        stage: bijux_domain_bam::BamStage::Align,
        profile: profile.id.to_string(),
        sample_id: Some(sample_id.clone()),
        r1: Some(r1.clone()),
        r2: args.r2.clone(),
        bam: align_out.join("align.bam"),
        out: align_out.clone(),
        tool: Some(tool_id),
        bai: None,
        reference: Some(reference.fasta.clone()),
        regions: None,
        udg_model: None,
        pmd_threshold_5p: None,
        pmd_threshold_3p: None,
        trim_5p: None,
        trim_3p: None,
        contamination_scope: None,
        contamination_panel: Vec::new(),
        contamination_prior: None,
        sex_specific_contamination: false,
        contamination_assumptions: None,
        expected_sex: None,
        sex_method: "rxy".to_string(),
        min_mapq: None,
        min_length: None,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        remove_duplicates: false,
        base_quality_threshold: None,
        optical_duplicates: None,
        umi_policy: None,
        duplicate_action: None,
        complexity_min_reads: None,
        complexity_projection_points: Vec::new(),
        depth_thresholds: Vec::new(),
        bqsr_mode: None,
        known_sites: Vec::new(),
        bqsr_min_mean_coverage: None,
        bqsr_min_breadth_1x: None,
        haplogroup_panel: None,
        haplogroup_min_coverage: None,
        kinship_panel: None,
        min_overlap_snps: None,
        caller: None,
        min_posterior: None,
        min_call_rate: None,
        gc_bias_correction: false,
        map_bias_correction: false,
        authenticity_mode: None,
        aligner_preset: None,
        rg_id: None,
        rg_sm: None,
        rg_pl: None,
        rg_lb: None,
        rg_policy: None,
        build_reference_indices: true,
        params_json: None,
        dry_run: false,
    };
    let align_plan = plan_for_bam_stage_with_profile(
        bijux_domain_bam::BamStage::Align,
        &spec,
        &align_args,
        profile,
        &align_out,
    )?;
    let align_result = execute_stage_plan(&align_plan, platform.runner, None)?;
    let mut runs = vec![StageExecutionSummary {
        plan: align_plan,
        result: align_result,
    }];

    let boundary = AlignmentBoundary {
        bam_path: align_out.join("align.bam").display().to_string(),
        bai_path: Some(align_out.join("align.bam.bai").display().to_string()),
        reference: Some(reference.fasta.display().to_string()),
        rg_policy: args.alignment_rg_policy.clone(),
        aligner_meta: None,
    };
    let mut rest = run_bam_truth_stages(
        registry_core,
        catalog,
        platform,
        profile,
        &boundary,
        out_dir,
    )?;
    runs.append(&mut rest);
    Ok(runs)
}
