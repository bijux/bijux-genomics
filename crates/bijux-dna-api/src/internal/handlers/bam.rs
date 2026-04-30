use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tool_selection::filter_tools_by_role;
use crate::{execution_kernel, execution_kernel::NetworkPolicy};
use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_dna_environment::api::{load_image_catalog, load_platform, RuntimeKind};
use bijux_dna_pipelines::registry;
use bijux_dna_pipelines::Domain;
use bijux_dna_planner_bam::stage_api::STAGE_PREFIX;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use std::path::PathBuf;

use crate::request_args::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};
use crate::v1::bam::plan::plan_for_bam_stage;

/// Output paths for BAM benchmarking.
///
/// Stability: v1 (stable).
/// Stability: v1
pub struct BamBenchOutcome {
    #[allow(dead_code)]
    pub run_dirs: Vec<PathBuf>,
}

/// # Errors
/// Returns an error if planning or execution fails for the requested stage.
#[allow(clippy::too_many_lines)]
pub fn bench_bam_stage(
    args: &BenchBamStageArgs,
    registry: &ToolRegistry,
    platform_path: Option<&str>,
) -> Result<BamBenchOutcome> {
    let platform =
        load_platform(platform_path).map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog =
        load_image_catalog().map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let stage = args.stage;
    let stage_id = stage.as_str();
    let mut tools = args.tools.clone();
    if tools.is_empty() {
        tools = bijux_dna_planner_bam::stage_api::allowed_tools_for_stage(stage)
            .into_iter()
            .map(|tool| tool.to_string())
            .collect();
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
                .join(stage_id.trim_start_matches(STAGE_PREFIX))
                .join(&tool)
                .join(format!("replicate_{rep}"));
            bijux_dna_infra::ensure_dir(&run_dir).context("create bam bench run dir")?;
            let run_args: BamRunArgs = args.into();
            let plan = plan_for_bam_stage(stage, &spec, &run_args, run_dir.as_path())?;
            if args.explain || args.dry_run {
                let plan_path = run_dir.join("plan.json");
                bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
                let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
                let graph = ExecutionGraph::new(
                    stage_id.to_string(),
                    bijux_dna_planner_bam::PLANNER_VERSION,
                    PlanPolicy::PreferAccuracy,
                    vec![step.clone()],
                    Vec::<ExecutionEdge>::new(),
                )?;
                let graph_path = run_dir.join("graph.json");
                bijux_dna_infra::atomic_write_json(&graph_path, &graph)?;
                let output_artifacts: Vec<serde_json::Value> = step
                    .io
                    .outputs
                    .iter()
                    .map(|artifact| {
                        serde_json::json!({
                            "stage_id": step.step_id.to_string(),
                            "name": artifact.name.to_string(),
                            "role": artifact.role.as_str(),
                            "optional": artifact.optional,
                            "path": artifact.path,
                            "sha256": serde_json::Value::Null,
                        })
                    })
                    .collect();
                let stage_contract_hash = bijux_dna_domain_bam::stage_contract_hash(stage_id)
                    .and_then(std::result::Result::ok);
                let graph_hash = graph.hash()?;
                let manifest = serde_json::json!({
                    "schema_version": "bijux.run_manifest.v3",
                    "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
                    "run_id": "dry-run",
                    "pipeline_id": stage_id,
                    "profile_id": args.stage.as_str(),
                    "graph_hash": graph_hash,
                    "cache_key": serde_json::Value::Null,
                    "stage_contract_hash": stage_contract_hash,
                    "toolchain_versions": [],
                    "dataset_fingerprints": [],
                    "tool_invocations": [],
                    "output_artifacts": output_artifacts,
                    "stages": [],
                    "failures": [],
                });
                let manifest_path = run_dir.join("run_manifest.json");
                let payload =
                    bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
                bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
            } else {
                let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
                let ctx = execution_kernel::ToolContext {
                    run_id: format!("bam-bench-{stage_id}-{tool}-rep-{rep}"),
                    stage_id: stage_id.to_string(),
                    tool_id: tool.clone(),
                    sample_id: Some(args.sample_id.clone()),
                    stage_root: bijux_dna_runtime::recording::run_artifacts_dir_for_out(&run_dir),
                    input_root: args
                        .bam
                        .parent()
                        .map_or_else(|| args.out.clone(), std::path::Path::to_path_buf),
                    output_root: run_dir.clone(),
                    tmp_root: run_dir.join("tmp"),
                    threads: plan.resources.threads.max(1),
                    memory_hint_mb: Some(u64::from(plan.resources.mem_gb).saturating_mul(1024)),
                    compression_threads: Some(1),
                    seed: None,
                    network_policy: NetworkPolicy::Allow,
                };
                execution_kernel::ToolExec::invoke(&execution_kernel::ToolInvocationRequest {
                    step: step.clone(),
                    runner: RuntimeKind::Docker,
                    context: ctx,
                    timeout: None,
                    mode: execution_kernel::ToolExecMode::Execute,
                })?;
            }
            run_dirs.push(run_dir);
        }
    }
    Ok(BamBenchOutcome { run_dirs })
}

/// # Errors
/// Returns an error if planning or execution fails for the requested pipeline.
pub fn bench_bam_pipeline(
    args: &BenchBamPipelineArgs,
    registry: &ToolRegistry,
    platform_path: Option<&str>,
) -> Result<BamBenchOutcome> {
    let profile = registry::profile_by_id(Domain::Bam, &args.profile)?;
    let tool_matrix = parse_tool_matrix(&args.tools)?;
    let mut run_dirs = Vec::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog(profile.id.as_str()) {
        let stage = bijux_dna_planner_bam::stage_api::BamStage::try_from(stage_id.as_str())?;
        let stage_id = stage.as_str();
        let tools = tool_matrix.get(stage_id).cloned().unwrap_or_default();
        let stage_args = BenchBamStageArgs {
            sample_id: args.sample_id.clone(),
            stage,
            bam: args.bam.clone(),
            out: args.out.clone(),
            tools,
            explain: args.explain,
            allow_silver: args.allow_silver,
            allow_experimental: args.allow_experimental,
            replicates: args.replicates,
            jobs: args.jobs,
            dry_run: args.dry_run,
            allow_planned: args.allow_planned,
        };
        let outcome = bench_bam_stage(&stage_args, registry, platform_path)?;
        run_dirs.extend(outcome.run_dirs);
    }
    Ok(BamBenchOutcome { run_dirs })
}

impl From<&BenchBamStageArgs> for BamRunArgs {
    fn from(value: &BenchBamStageArgs) -> Self {
        BamRunArgs {
            stage: value.stage,
            profile: "default".to_string(),
            sample_id: Some(value.sample_id.clone()),
            r1: None,
            r2: None,
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
            aligner_preset: None,
            alignment_sensitivity_profile: None,
            alignment_seed_length: None,
            rg_id: None,
            rg_sm: None,
            rg_pl: None,
            rg_lb: None,
            rg_pu: None,
            lane_id: None,
            run_id: None,
            subject_id: None,
            cohort_id: None,
            rg_policy: None,
            build_reference_indices: false,
            params_json: None,
            dry_run: value.dry_run,
            allow_planned: value.allow_planned,
        }
    }
}

fn parse_tool_matrix(
    entries: &[String],
) -> Result<std::collections::BTreeMap<String, Vec<String>>> {
    let mut map = std::collections::BTreeMap::new();
    for entry in entries {
        let mut parts = entry.split('=');
        let stage_raw =
            parts.next().ok_or_else(|| anyhow!("invalid tool matrix entry: {entry}"))?;
        let tools_raw =
            parts.next().ok_or_else(|| anyhow!("invalid tool matrix entry: {entry}"))?;
        let stage_id = if stage_raw.contains('.') {
            stage_raw.to_string()
        } else {
            format!("{STAGE_PREFIX}{stage_raw}")
        };
        let tools = tools_raw
            .split(',')
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        map.insert(stage_id, tools);
    }
    Ok(map)
}
