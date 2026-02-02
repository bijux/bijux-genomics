use anyhow::{anyhow, Context, Result};
use bijux_core::ToolRegistry;
use bijux_engine::api::{
    build_tool_execution_spec, ensure_image_qa_passed, ensure_tool_qa_passed, execute_stage_plan,
    filter_tools_by_role,
};
use bijux_env_runtime::api::{load_image_catalog, load_platform, RunnerKind};
use std::path::PathBuf;

use crate::cli::parse::{BenchBamPipelineArgs, BenchBamStageArgs};
use crate::plan_for_bam_stage;

pub struct BamBenchOutcome {
    #[allow(dead_code)]
    pub run_dirs: Vec<PathBuf>,
}

pub fn bench_bam_stage(
    args: &BenchBamStageArgs,
    registry: &ToolRegistry,
    platform_path: Option<&str>,
) -> Result<BamBenchOutcome> {
    let platform =
        load_platform(platform_path).map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog =
        load_image_catalog().map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let stage = args.stage.stage();
    let stage_id = stage.as_str();
    let mut tools = args.tools.clone();
    if tools.is_empty() {
        tools = bijux_stages_bam::bam_tools_registry::allowed_tools_for_stage(stage);
    }
    let prev_silver = std::env::var("BIJUX_ALLOW_SILVER").ok();
    let prev_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").ok();
    if args.allow_silver {
        std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    }
    if args.allow_experimental {
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
    }
    let tools = filter_tools_by_role(stage_id, &tools, registry, false)?;
    if let Some(value) = prev_silver {
        std::env::set_var("BIJUX_ALLOW_SILVER", value);
    }
    if let Some(value) = prev_experimental {
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", value);
    }
    ensure_image_qa_passed(stage_id, &tools, &platform, &catalog)?;
    ensure_tool_qa_passed(stage_id, &tools, &platform, &catalog)?;

    let mut run_dirs = Vec::new();
    for tool in tools {
        for rep in 0..args.replicates {
            let spec = build_tool_execution_spec(stage_id, &tool, registry, &catalog, &platform)?;
            let run_dir = args
                .out
                .join(stage_id.trim_start_matches("bam."))
                .join(&tool)
                .join(format!("replicate_{rep}"));
            std::fs::create_dir_all(&run_dir).context("create bam bench run dir")?;
            let run_args: crate::cli::parse::BamRunArgs = args.into();
            let plan = plan_for_bam_stage(stage, &spec, &run_args, run_dir.as_path())?;
            if args.explain || args.dry_run {
                let plan_path = run_dir.join("plan.json");
                std::fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;
            } else {
                execute_stage_plan(&plan, RunnerKind::Docker, None)?;
            }
            run_dirs.push(run_dir);
        }
    }
    Ok(BamBenchOutcome { run_dirs })
}

pub fn bench_bam_pipeline(
    args: &BenchBamPipelineArgs,
    registry: &ToolRegistry,
    platform_path: Option<&str>,
) -> Result<BamBenchOutcome> {
    let profile = bijux_pipelines_bam::profile_by_id(&args.profile)?;
    let tool_matrix = parse_tool_matrix(&args.tools)?;
    let mut run_dirs = Vec::new();
    for stage in profile.stages {
        let stage_id = stage.as_str();
        let tools = tool_matrix
            .get(stage_id)
            .cloned()
            .unwrap_or_default();
        let stage_args = BenchBamStageArgs {
            sample_id: args.sample_id.clone(),
            stage: stage.into(),
            bam: args.bam.clone(),
            out: args.out.clone(),
            tools,
            explain: args.explain,
            allow_silver: args.allow_silver,
            allow_experimental: args.allow_experimental,
            replicates: args.replicates,
            jobs: args.jobs,
            dry_run: args.dry_run,
        };
        let outcome = bench_bam_stage(&stage_args, registry, platform_path)?;
        run_dirs.extend(outcome.run_dirs);
    }
    Ok(BamBenchOutcome { run_dirs })
}

impl From<&BenchBamStageArgs> for crate::cli::parse::BamRunArgs {
    fn from(value: &BenchBamStageArgs) -> Self {
        crate::cli::parse::BamRunArgs {
            stage: value.stage,
            profile: "default".to_string(),
            bam: value.bam.clone(),
            out: value.out.clone(),
            tool: None,
            bai: None,
            reference: None,
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
            params_json: None,
            dry_run: value.dry_run,
        }
    }
}

impl From<bijux_domain_bam::BamStage> for crate::cli::parse::BamStageArg {
    fn from(value: bijux_domain_bam::BamStage) -> Self {
        match value {
            bijux_domain_bam::BamStage::Validate => crate::cli::parse::BamStageArg::Validate,
            bijux_domain_bam::BamStage::QcPre => crate::cli::parse::BamStageArg::QcPre,
            bijux_domain_bam::BamStage::Filter => crate::cli::parse::BamStageArg::Filter,
            bijux_domain_bam::BamStage::Markdup => crate::cli::parse::BamStageArg::Markdup,
            bijux_domain_bam::BamStage::Complexity => crate::cli::parse::BamStageArg::Complexity,
            bijux_domain_bam::BamStage::Coverage => crate::cli::parse::BamStageArg::Coverage,
            bijux_domain_bam::BamStage::Damage => crate::cli::parse::BamStageArg::Damage,
            bijux_domain_bam::BamStage::Authenticity => {
                crate::cli::parse::BamStageArg::Authenticity
            }
            bijux_domain_bam::BamStage::Contamination => {
                crate::cli::parse::BamStageArg::Contamination
            }
            bijux_domain_bam::BamStage::Sex => crate::cli::parse::BamStageArg::Sex,
            bijux_domain_bam::BamStage::BiasMitigation => {
                crate::cli::parse::BamStageArg::BiasMitigation
            }
            bijux_domain_bam::BamStage::Recalibration => {
                crate::cli::parse::BamStageArg::Recalibration
            }
            bijux_domain_bam::BamStage::Haplogroups => {
                crate::cli::parse::BamStageArg::Haplogroups
            }
            bijux_domain_bam::BamStage::Genotyping => {
                crate::cli::parse::BamStageArg::Genotyping
            }
            bijux_domain_bam::BamStage::Kinship => crate::cli::parse::BamStageArg::Kinship,
        }
    }
}

fn parse_tool_matrix(entries: &[String]) -> Result<std::collections::BTreeMap<String, Vec<String>>> {
    let mut map = std::collections::BTreeMap::new();
    for entry in entries {
        let mut parts = entry.split('=');
        let stage_raw = parts
            .next()
            .ok_or_else(|| anyhow!("invalid tool matrix entry: {entry}"))?;
        let tools_raw = parts
            .next()
            .ok_or_else(|| anyhow!("invalid tool matrix entry: {entry}"))?;
        let stage_id = if stage_raw.contains('.') {
            stage_raw.to_string()
        } else {
            format!("bam.{stage_raw}")
        };
        let tools = tools_raw
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        map.insert(stage_id, tools);
    }
    Ok(map)
}
