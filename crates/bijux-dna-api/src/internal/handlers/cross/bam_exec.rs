use std::path::{Path, PathBuf};

use super::AlignmentBoundary;
use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_environment::resolve::ReferenceRecord;
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::execution_kernel::{invoke_tool, NetworkPolicy, ToolContext, ToolInvocationRequest};
use crate::internal::handlers::fastq::StageExecutionSummary;
use crate::request_args::{BamRunArgs, FastqCrossArgs};
use crate::v1::bam::downstream_enabled;
use crate::v1::bam::plan::plan_for_bam_stage_with_profile;
use bijux_dna_planner_bam::stage_api::STAGE_PREFIX;

fn base_bam_args(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    profile: &PipelineProfile,
    bam: PathBuf,
    out: PathBuf,
    bai: Option<PathBuf>,
    reference: Option<PathBuf>,
) -> BamRunArgs {
    BamRunArgs {
        stage,
        profile: profile.id.to_string(),
        sample_id: None,
        r1: None,
        r2: None,
        bam,
        out,
        tool: None,
        dry_run: false,
        allow_planned: false,
        bai,
        reference,
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
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum AlignmentRegime {
    Adna,
    Modern,
    Edna,
}

fn alignment_meta_value(args: &FastqCrossArgs, key: &str) -> Option<String> {
    for entry in &args.alignment_meta {
        if let Some((found_key, found_value)) = entry.split_once('=') {
            if found_key == key {
                return Some(found_value.to_string());
            }
        }
    }
    None
}

fn infer_alignment_regime(profile: &PipelineProfile, args: &FastqCrossArgs) -> AlignmentRegime {
    if let Some(explicit) = alignment_meta_value(args, "alignment_regime") {
        return match explicit.as_str() {
            "adna" => AlignmentRegime::Adna,
            "edna" | "pollen" => AlignmentRegime::Edna,
            "modern" => AlignmentRegime::Modern,
            _ => AlignmentRegime::Modern,
        };
    }
    let profile_id = profile.id.as_str().to_ascii_lowercase();
    if profile_id.contains("edna") || profile_id.contains("pollen")
    {
        return AlignmentRegime::Edna;
    }
    if profile_id.contains("adna") {
        return AlignmentRegime::Adna;
    }
    AlignmentRegime::Modern
}

fn write_stage_accounting(
    stage_dir: &Path,
    stage_id: &str,
    result: &bijux_dna_runner::execute::StageResultV1,
) -> Result<()> {
    let payload = serde_json::json!({
        "stage_id": stage_id,
        "exit_code": result.exit_code,
        "runtime_s": result.runtime_s,
        "memory_mb": result.memory_mb,
        "output_count": result.outputs.len(),
        "outputs": result.outputs,
    });
    let path = stage_dir.join("stage_loss_accounting.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn enforce_stage_refusal_rules(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
) -> Result<()> {
    if !bam_path.exists() {
        return Err(anyhow!(
            "bam input missing for {}: {}",
            stage.as_str(),
            bam_path.display()
        ));
    }
    if matches!(
        stage,
        bijux_dna_planner_bam::stage_api::BamStage::Validate
            | bijux_dna_planner_bam::stage_api::BamStage::QcPre
            | bijux_dna_planner_bam::stage_api::BamStage::MappingSummary
            | bijux_dna_planner_bam::stage_api::BamStage::MapqFilter
            | bijux_dna_planner_bam::stage_api::BamStage::Filter
            | bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection
            | bijux_dna_planner_bam::stage_api::BamStage::LengthFilter
    ) && bai_path.is_none()
    {
        return Err(anyhow!(
            "{} requires BAM index (.bai) but none was provided",
            stage.as_str()
        ));
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Align && reference.is_none() {
        return Err(anyhow!("bam.align requires resolved reference fasta"));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn run_bam_truth_stages<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
    profile: &PipelineProfile,
    boundary: &AlignmentBoundary,
    out_dir: &Path,
) -> Result<Vec<StageExecutionSummary>> {
    let bam_path = PathBuf::from(&boundary.bam_path);
    let bai_path = boundary.bai_path.as_ref().map(PathBuf::from);
    let reference = boundary.reference.as_ref().map(PathBuf::from);

    let mut runs = Vec::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog(profile.id.as_str()) {
        let stage = bijux_dna_planner_bam::stage_api::BamStage::try_from(stage_id.as_str())?;
        if should_skip_bam_truth_stage(stage) {
            continue;
        }
        let summary = run_bam_truth_stage(
            registry_core,
            catalog,
            platform,
            profile,
            stage,
            &bam_path,
            bai_path.as_ref(),
            reference.as_ref(),
            out_dir,
        )?;
        runs.push(summary);
    }

    Ok(runs)
}

fn should_skip_bam_truth_stage(stage: bijux_dna_planner_bam::stage_api::BamStage) -> bool {
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Align {
        return true;
    }
    if !downstream_enabled()
        && matches!(
            stage,
            bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
                | bijux_dna_planner_bam::stage_api::BamStage::Genotyping
                | bijux_dna_planner_bam::stage_api::BamStage::Kinship
        )
    {
        return true;
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn run_bam_truth_stage<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
    profile: &PipelineProfile,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
    out_dir: &Path,
) -> Result<StageExecutionSummary> {
    enforce_stage_refusal_rules(stage, bam_path, bai_path, reference)?;
    let stage_key = bijux_dna_core::ids::StageId::from_static(stage.as_str());
    let tool_id = profile
        .defaults
        .tools
        .get(&stage_key)
        .cloned()
        .unwrap_or_else(|| bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage));
    let spec = build_tool_execution_spec(
        stage.as_str(),
        tool_id.as_str(),
        registry_core,
        catalog,
        platform,
    )?;

    let stage_dir = out_dir
        .join("bam")
        .join(stage.as_str().trim_start_matches(STAGE_PREFIX));
    bijux_dna_infra::ensure_dir(&stage_dir).context("create bam stage dir")?;

    let mut args = base_bam_args(
        stage,
        profile,
        bam_path.to_path_buf(),
        stage_dir.clone(),
        bai_path.cloned(),
        reference.cloned(),
    );
    args.tool = Some(tool_id.as_str().to_string());

    let plan = plan_for_bam_stage_with_profile(stage, &spec, &args, profile, &stage_dir)?;
    let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
    let context = ToolContext {
        run_id: format!("bam-{}-{}", stage.as_str(), tool_id.as_str()),
        stage_id: stage.as_str().to_string(),
        tool_id: tool_id.as_str().to_string(),
        sample_id: None,
        stage_root: bijux_dna_runtime::recording::run_artifacts_dir_for_out(&stage_dir),
        input_root: bam_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| out_dir.to_path_buf()),
        output_root: stage_dir.clone(),
        tmp_root: stage_dir.join("tmp"),
        threads: plan.resources.threads.max(1),
        memory_hint_mb: Some(u64::from(plan.resources.mem_gb).saturating_mul(1024)),
        seed: None,
        network_policy: NetworkPolicy::Allow,
    };
    let result = invoke_tool(&ToolInvocationRequest {
        step: step.clone(),
        runner: platform.runner,
        context,
        timeout: None,
    })?
    .stage_result;
    write_stage_accounting(&stage_dir, stage.as_str(), &result)?;
    Ok(StageExecutionSummary { plan: step, result })
}

#[allow(clippy::too_many_lines)]
pub(crate) fn run_bam_align_and_truth_stages<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
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
    let regime = infer_alignment_regime(profile, args);
    let align_out = out_dir.join("bam").join("align");
    bijux_dna_infra::ensure_dir(&align_out)?;
    let align_stage = bijux_dna_core::ids::StageId::from_static(
        bijux_dna_planner_bam::stage_api::BamStage::Align.as_str(),
    );
    let tool_id = profile
        .defaults
        .tools
        .get(&align_stage)
        .cloned()
        .unwrap_or_else(|| bijux_dna_core::ids::ToolId::from_static("bwa"));
    let spec = build_tool_execution_spec(
        bijux_dna_planner_bam::stage_api::BamStage::Align.as_str(),
        tool_id.as_str(),
        registry_core,
        catalog,
        platform,
    )?;
    let mut align_args = base_bam_args(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        profile,
        align_out.join("align.bam"),
        align_out.clone(),
        None,
        Some(reference.fasta.clone()),
    );
    align_args.sample_id = Some(sample_id.clone());
    align_args.r1 = Some(r1.clone());
    align_args.r2.clone_from(&args.r2);
    align_args.tool = Some(tool_id.as_str().to_string());
    align_args.build_reference_indices = true;
    align_args.aligner_preset = Some(match regime {
        AlignmentRegime::Adna => "adna_sensitive".to_string(),
        AlignmentRegime::Modern => "modern_default".to_string(),
        AlignmentRegime::Edna => "edna_metagenomic".to_string(),
    });
    let align_plan = plan_for_bam_stage_with_profile(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &spec,
        &align_args,
        profile,
        &align_out,
    )?;
    let align_step = bijux_dna_stage_contract::execution_step_from_stage_plan(&align_plan);
    let align_context = ToolContext {
        run_id: format!("bam-{}-{}", align_step.step_id, tool_id.as_str()),
        stage_id: align_step.step_id.to_string(),
        tool_id: tool_id.as_str().to_string(),
        sample_id: Some(sample_id.clone()),
        stage_root: bijux_dna_runtime::recording::run_artifacts_dir_for_out(&align_out),
        input_root: r1
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| out_dir.to_path_buf()),
        output_root: align_out.clone(),
        tmp_root: align_out.join("tmp"),
        threads: align_plan.resources.threads.max(1),
        memory_hint_mb: Some(u64::from(align_plan.resources.mem_gb).saturating_mul(1024)),
        seed: None,
        network_policy: NetworkPolicy::Allow,
    };
    let align_result = invoke_tool(&ToolInvocationRequest {
        step: align_step.clone(),
        runner: platform.runner,
        context: align_context,
        timeout: None,
    })?
    .stage_result;
    write_stage_accounting(&align_out, align_step.step_id.as_str(), &align_result)?;
    let header_normalization = serde_json::json!({
        "stage_id": align_step.step_id,
        "regime": regime,
        "policy": "read_group_normalization",
        "rg_id": align_args.rg_id.as_deref().unwrap_or("auto"),
        "rg_sm": align_args.rg_sm.as_deref().unwrap_or(&sample_id),
        "rg_pl": align_args.rg_pl.as_deref().unwrap_or("ILLUMINA"),
        "rg_lb": align_args.rg_lb.as_deref().unwrap_or("lib1"),
    });
    let header_norm_path = align_out.join("header_normalization.json");
    bijux_dna_infra::atomic_write_json(&header_norm_path, &header_normalization)
        .with_context(|| format!("write {}", header_norm_path.display()))?;
    let regime_path = align_out.join("alignment_regime.json");
    bijux_dna_infra::atomic_write_json(
        &regime_path,
        &serde_json::json!({
            "regime": regime,
            "profile_id": profile.id,
            "source": "explicit_or_profile_inference",
        }),
    )
    .with_context(|| format!("write {}", regime_path.display()))?;
    let mut runs = vec![StageExecutionSummary {
        plan: align_step,
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
