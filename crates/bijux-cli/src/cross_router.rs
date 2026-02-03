use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_core::alignment::AlignmentBoundary;
use bijux_core::ToolRegistry;
use bijux_engine::api::{bench_base_dir, hash_file_sha256};
use bijux_engine::api::{build_tool_execution_spec, execute_stage_plan};
use bijux_env_runtime::api::{load_image_catalog, load_platform};
use bijux_pipelines::registry;
use bijux_pipelines::{Domain, PipelineProfile};

use crate::cli::parse::FastqPreprocessArgs;
use crate::cli::plan::preprocess_args_from_cli;
use crate::fastq_router::{fastq_preprocess_run, StageExecutionSummary};
use crate::{downstream_enabled, init_logging, plan_for_bam_stage_with_profile, Cli};

const CROSS_STAGE_ID: &str = "cross.align_stub";

pub fn run_fastq_to_bam_profile(
    cli: &Cli,
    registry_core: &ToolRegistry,
    args: &FastqPreprocessArgs,
    profile: &PipelineProfile,
) -> Result<()> {
    let platform = load_platform(cli.platform.as_deref())
        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog =
        load_image_catalog().map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let runner = crate::cli::parse_runner_override(args.env.as_deref())?;

    let bench_args = preprocess_args_from_cli(args)?;
    let out_dir = bench_base_dir(&bench_args.out, "preprocess", &bench_args.sample_id);
    fs::create_dir_all(&out_dir).context("create cross pipeline out dir")?;
    let log_path = out_dir.join("bijux_cross.log");
    let _log_guard = init_logging(&log_path)?;

    fastq_preprocess_run(&catalog, &platform, runner, &bench_args)?;

    let alignment_boundary = build_alignment_boundary(args)?;
    let boundary_path = write_alignment_boundary(&out_dir, &alignment_boundary)?;

    let bam_profile = select_bam_profile(profile)?;
    let bam_stage_runs = run_bam_truth_stages(
        registry_core,
        &catalog,
        &platform,
        &bam_profile,
        &alignment_boundary,
        &out_dir,
    )?;

    let summary_path = out_dir.join("run_artifacts").join("run_summary.json");
    let summary_raw = fs::read_to_string(&summary_path)
        .with_context(|| format!("read {}", summary_path.display()))?;
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary_raw).context("parse run_summary.json")?;

    write_cross_run_manifest(
        &out_dir,
        profile,
        &summary_json,
        &bam_stage_runs,
        &boundary_path,
    )?;

    println!("cross-domain run complete: {}", out_dir.display());
    println!("alignment_boundary: {}", boundary_path.display());
    Ok(())
}

fn build_alignment_boundary(args: &FastqPreprocessArgs) -> Result<AlignmentBoundary> {
    let bam_path = args
        .alignment_bam
        .as_ref()
        .ok_or_else(|| anyhow!("--alignment-bam is required for cross-domain profiles"))?;
    let mut aligner_meta = BTreeMap::new();
    for entry in &args.alignment_meta {
        let Some((key, value)) = entry.split_once('=') else {
            return Err(anyhow!("--alignment-meta must be KEY=VALUE (got {entry})"));
        };
        aligner_meta.insert(key.to_string(), value.to_string());
    }
    Ok(AlignmentBoundary {
        bam_path: bam_path.display().to_string(),
        bai_path: args
            .alignment_bai
            .as_ref()
            .map(|path| path.display().to_string()),
        reference: args
            .alignment_reference
            .as_ref()
            .map(|path| path.display().to_string()),
        rg_policy: args.alignment_rg_policy.clone(),
        aligner_meta: if aligner_meta.is_empty() {
            None
        } else {
            Some(aligner_meta)
        },
    })
}

fn write_alignment_boundary(out_dir: &Path, boundary: &AlignmentBoundary) -> Result<PathBuf> {
    let boundaries_dir = out_dir.join("run_artifacts").join("boundaries");
    fs::create_dir_all(&boundaries_dir).context("create boundaries dir")?;
    let path = boundaries_dir.join("alignment_boundary.json");
    fs::write(&path, serde_json::to_vec_pretty(boundary)?)
        .context("write alignment_boundary.json")?;
    Ok(path)
}

fn select_bam_profile(profile: &PipelineProfile) -> Result<PipelineProfile> {
    let id = if profile.invariants_preset == Some("adna") {
        "adna-shotgun"
    } else {
        "default"
    };
    registry::profile_by_id(Domain::Bam, id)
}

fn run_bam_truth_stages(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec>,
    platform: &bijux_engine::api::PlatformSpec,
    profile: &PipelineProfile,
    boundary: &AlignmentBoundary,
    out_dir: &Path,
) -> Result<Vec<StageExecutionSummary>> {
    let bam_path = PathBuf::from(&boundary.bam_path);
    let bai_path = boundary.bai_path.as_ref().map(PathBuf::from);
    let reference = boundary.reference.as_ref().map(PathBuf::from);

    let stages = [
        bijux_domain_bam::BamStage::QcPre,
        bijux_domain_bam::BamStage::Coverage,
        bijux_domain_bam::BamStage::Damage,
    ];

    let mut runs = Vec::new();
    for stage in stages {
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
        fs::create_dir_all(&stage_dir).context("create bam stage dir")?;

        let args = crate::cli::parse::BamRunArgs {
            stage: stage.into(),
            profile: profile.id.to_string(),
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
            params_json: None,
        };

        let plan = plan_for_bam_stage_with_profile(stage, &spec, &args, profile, &stage_dir)?;
        let result = execute_stage_plan(&plan, platform.runner, None)?;
        runs.push(StageExecutionSummary { plan, result });
    }

    Ok(runs)
}

fn write_cross_run_manifest(
    out_dir: &Path,
    profile: &PipelineProfile,
    fastq_summary: &serde_json::Value,
    bam_runs: &[StageExecutionSummary],
    boundary_path: &Path,
) -> Result<()> {
    let run_id = fastq_summary
        .get("run_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    let mut stages = Vec::new();
    if let Some(fastq_stages) = fastq_summary
        .get("stages")
        .and_then(serde_json::Value::as_array)
    {
        for stage in fastq_stages {
            let stage_id = stage
                .get("stage_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let tool_id = stage
                .get("tool_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            stages.push(serde_json::json!({
                "stage_id": stage_id,
                "tool_id": tool_id,
                "domain": "fastq",
                "artifacts": stage.get("artifacts").cloned().unwrap_or(serde_json::json!({})),
            }));
        }
    }

    stages.push(serde_json::json!({
        "stage_id": CROSS_STAGE_ID,
        "tool_id": "alignment_boundary",
        "domain": "cross",
        "artifacts": {
            "alignment_boundary": boundary_path,
        },
    }));

    for entry in bam_runs {
        stages.push(serde_json::json!({
            "stage_id": entry.plan.stage_id.0,
            "tool_id": entry.plan.tool_id.0,
            "domain": "bam",
            "artifacts": {
                "out_dir": entry.plan.out_dir,
                "metrics": entry.plan.out_dir.join("run_artifacts").join("metrics.json"),
                "stage_report": entry.plan.out_dir.join("run_artifacts").join("stage_report.json"),
            },
        }));
    }

    let boundary_hash = hash_file_sha256(boundary_path)?;
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v2",
        "run_id": run_id,
        "profile_id": profile.id,
        "domains": profile.domains,
        "stages": stages,
        "domain_transitions": [{
            "from": "fastq",
            "to": "bam",
            "boundary": boundary_path,
        }],
        "boundaries": [{
            "name": "alignment_boundary",
            "path": boundary_path,
            "sha256": boundary_hash,
        }],
    });
    let path = out_dir.join("run_manifest.json");
    fs::write(&path, serde_json::to_vec_pretty(&manifest)?).context("write run_manifest.json")?;
    Ok(())
}
