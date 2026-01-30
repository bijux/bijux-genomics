use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::BenchmarkRecord;
use bijux_engine::api::{
    bench_base_dir, bench_tools_dir, execute_plan, PlatformSpec, RunnerKind, ToolImageSpec,
};
use bijux_engine::api::{ensure_bench_runner, filter_tools_by_role, load_registry};
use bijux_engine::api::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_engine::api::{resolve_image_for_run, ExplainExclusion, ExplainPlan, StagePlan};
use bijux_stages_fastq::fastq::correct::{normalize_correct_tool_list, plan_correct};
use bijux_stages_fastq::fastq::filter::{normalize_filter_tool_list, plan_filter};
use bijux_stages_fastq::fastq::merge::{normalize_merge_tool_list, plan_merge};
use bijux_stages_fastq::fastq::preprocess::{plan_preprocess, plan_preprocess_pipeline};
use bijux_stages_fastq::fastq::qc_post::{aux_tool_ids, normalize_qc_post_tool_list, plan_qc_post};
use bijux_stages_fastq::fastq::screen::{normalize_screen_tool_list, plan_screen};
use bijux_stages_fastq::fastq::trim::plan;
use bijux_stages_fastq::fastq::umi::{normalize_umi_tool_list, plan_umi};
use bijux_stages_fastq::fastq::validate_pre::{
    normalize_validate_tool_list, plan as plan_validate_pre,
};
use bijux_stages_fastq::{
    bench_corpus, canonical_tool_defaults, ensure_umi_headers, inspect_headers,
    log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure, StagePlanJson,
};
use bijux_stages_fastq::{FastqArtifact, FastqLayout};

pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: std::path::PathBuf,
    pub explain: bool,
}

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
}

/// # Errors
/// Returns an error if the explain markdown cannot be written.
pub(crate) fn write_explain_md(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    excluded: &[String],
    policy: Option<bijux_engine::api::Policy>,
) -> Result<()> {
    let path = base_dir.join("explain.md");
    let mut lines = Vec::new();
    lines.push(format!("# Explain: {stage}"));
    if let Some(policy) = policy {
        lines.push(format!("\nPolicy: `{policy:?}`"));
    }
    lines.push("\n## Selected tools".to_string());
    for tool in selected {
        lines.push(format!("- {tool}"));
    }
    if !excluded.is_empty() {
        lines.push("\n## Excluded tools".to_string());
        for tool in excluded {
            lines.push(format!("- {tool}"));
        }
    }
    std::fs::write(&path, lines.join("\n")).context("write explain.md")?;
    Ok(())
}

/// # Errors
/// Returns an error if the explain plan JSON cannot be written.
pub(crate) fn write_explain_plan_json(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    registry: &bijux_core::ToolRegistry,
    policy: Option<bijux_engine::api::Policy>,
) -> Result<()> {
    let mut excluded = Vec::new();
    for tool in registry.tools_for_stage(stage) {
        if !selected.iter().any(|t| t == &tool.tool_id) {
            excluded.push(ExplainExclusion {
                tool: tool.tool_id.clone(),
                reason: "not selected".to_string(),
            });
        }
    }
    let invariants = vec![
        "stage_contract".to_string(),
        "header_inspection".to_string(),
        "output_normalization".to_string(),
    ];
    let plan = ExplainPlan {
        stage: stage.to_string(),
        selected_tools: selected.to_vec(),
        excluded_tools: excluded,
        policy,
        invariants,
    };
    let path = base_dir.join("explain_plan.json");
    bijux_engine::api::write_explain_plan(&path, &plan)
}

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> bijux_core::domain::PipelineSpec {
    plan_preprocess(args).pipeline
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqTrimMetrics>> {
    let tools = bijux_engine::api::normalize_trim_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.trim", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.trim", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.trim", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "trim", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "trim", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.trim", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.trim", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.trim", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.trim", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan(tool, &args.r1, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let exec_plan = StagePlan {
            stage_id: "fastq.trim".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::trim::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image,
            runner: platform.runner,
            inputs: vec![args.r1.clone()],
            out_dir: out_dir.clone(),
            outputs: vec![plan.output.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.trim".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_validate_pre<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqValidateArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqValidateMetrics>> {
    let tools = normalize_validate_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.validate_pre", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.validate_pre", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.validate_pre", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "validate_pre", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "validate_pre", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.validate_pre", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.validate_pre", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.validate_pre", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.validate_pre", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_validate_pre(tool, &args.r1, &out_dir);
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let exec_plan = StagePlan {
            stage_id: "fastq.validate_pre".to_string(),
            stage_version: stage_version_i32(
                bijux_stages_fastq::fastq::validate_pre::STAGE_VERSION,
            ),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image,
            runner: platform.runner,
            inputs: vec![args.r1.clone()],
            out_dir: out_dir.clone(),
            outputs: Vec::new(),
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.validate_pre".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqFilterMetrics>> {
    let tools = normalize_filter_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.filter", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.filter", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.filter", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "filter", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "filter", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.filter", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.filter", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.filter", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.filter", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_filter(tool, &args.r1, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let exec_plan = StagePlan {
            stage_id: "fastq.filter".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::filter::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image,
            runner: platform.runner,
            inputs: vec![args.r1.clone()],
            out_dir: out_dir.clone(),
            outputs: vec![plan.output.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.filter".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_merge<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqMergeArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqMergeMetrics>> {
    let tools = normalize_merge_tool_list(&args.tools)?;
    preflight_stage("fastq.merge", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, Some(&args.r2), false)?;
    log_header_warnings("fastq.merge", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.merge", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "merge", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "merge", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.merge", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.merge", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.merge", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.merge", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_merge(tool, &args.r1, &args.r2, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let exec_plan = StagePlan {
            stage_id: "fastq.merge".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::merge::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image: resolve_image_for_run(spec, platform)?,
            runner: platform.runner,
            inputs: vec![args.r1.clone(), args.r2.clone()],
            out_dir: out_dir.clone(),
            outputs: vec![plan.output.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.merge".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqScreenArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqScreenMetrics>> {
    let tools = normalize_screen_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.screen", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.screen", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.screen", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "screen", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "screen", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.screen", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.screen", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.screen", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.screen", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_screen(tool, &args.r1, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let exec_plan = StagePlan {
            stage_id: "fastq.screen".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::screen::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image: resolve_image_for_run(spec, platform)?,
            runner: platform.runner,
            inputs: vec![args.r1.clone()],
            out_dir: out_dir.clone(),
            outputs: Vec::new(),
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.screen".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqUmiMetrics>> {
    let tools = normalize_umi_tool_list(&args.tools)?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.umi"))?;
    preflight_stage("fastq.umi", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    ensure_umi_headers(&args.r1, args.r2.as_deref())?;
    log_header_warnings("fastq.umi", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.umi", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "umi", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "umi", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.umi", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.umi", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.umi", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.umi", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_umi(tool, &args.r1, r2, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let exec_plan = StagePlan {
            stage_id: "fastq.umi".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::umi::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image: resolve_image_for_run(spec, platform)?,
            runner: platform.runner,
            inputs: vec![args.r1.clone(), r2.clone()],
            out_dir: out_dir.clone(),
            outputs: vec![plan.output_r1.clone(), plan.output_r2.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.umi".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqCorrectMetrics>> {
    let tools = normalize_correct_tool_list(&args.tools)?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.correct"))?;
    preflight_stage("fastq.correct", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings("fastq.correct", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.correct", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "correct", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "correct", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.correct", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.correct", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.correct", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.correct", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_correct(tool, &args.r1, r2, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let exec_plan = StagePlan {
            stage_id: "fastq.correct".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::correct::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image: resolve_image_for_run(spec, platform)?,
            runner: platform.runner,
            inputs: vec![args.r1.clone(), r2.clone()],
            out_dir: out_dir.clone(),
            outputs: vec![plan.output_r1.clone(), plan.output_r2.clone()],
            params: plan_json.parameters,
            aux_images: HashMap::new(),
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.correct".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_qc_post<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqQcPostMetrics>> {
    let tools = normalize_qc_post_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.qc_post", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.qc_post", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.qc_post", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "qc_post", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "qc_post", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.qc_post", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.qc_post", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.qc_post", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.qc_post", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let plan = plan_qc_post(tool, &args.r1, &out_dir)?;
        let plan_json = StagePlanJson::from_plan(&plan);
        let mut aux_images = HashMap::new();
        for aux_tool in aux_tool_ids() {
            let image = resolve_image_for_run(
                catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.yaml"))?,
                platform,
            )?;
            aux_images.insert((*aux_tool).to_string(), image);
        }
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let exec_plan = StagePlan {
            stage_id: "fastq.qc_post".to_string(),
            stage_version: stage_version_i32(bijux_stages_fastq::fastq::qc_post::STAGE_VERSION),
            tool: tool.to_string(),
            tool_version: spec.version.clone(),
            image: resolve_image_for_run(spec, platform)?,
            runner: platform.runner,
            inputs: vec![args.r1.clone()],
            out_dir: out_dir.clone(),
            outputs: Vec::new(),
            params: plan_json.parameters,
            aux_images,
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.qc_post".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// Run the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    fastq_preprocess_run(catalog, platform, runner_override, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;

    ensure_bench_runner(platform, runner_override)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let pipeline = plan_preprocess(args).pipeline;
    let mut selected_tools = select_preprocess_tools(&registry, &pipeline, args)?;
    selected_tools = filter_tools_by_role("fastq.preprocess", &selected_tools, &registry, false)?;

    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;

    ensure_image_qa_passed("fastq.preprocess", &selected_tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.preprocess", &selected_tools, platform, catalog)?;

    let tools_root = bench_tools_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&tools_root).context("create preprocess tools dir")?;

    let mut failures = Vec::new();
    let planned_stages = plan_preprocess_pipeline(
        &pipeline.stages,
        &selected_tools,
        &args.r1,
        args.r2.as_deref(),
        |stage, tool, _r1, _r2| {
            let stage_dir = stage.trim_start_matches("fastq.");
            let stage_root = bench_tools_dir(&args.out, stage_dir, &args.sample_id);
            let out_dir = stage_root.join(tool);
            fs::create_dir_all(&out_dir).context("create stage output dir")?;
            Ok(out_dir)
        },
    )?;

    for planned in planned_stages {
        let tool = planned.tool.0.clone();
        let image = resolve_image_for_run(
            catalog
                .get(&tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?,
            platform,
        )?;
        let mut aux_images = HashMap::new();
        if planned.stage.0 == bijux_stages_fastq::fastq::qc_post::STAGE_ID {
            for aux_tool in bijux_stages_fastq::fastq::qc_post::aux_tool_ids() {
                let spec = catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("{aux_tool} missing from images.yaml"))?;
                let image = resolve_image_for_run(spec, platform)?;
                aux_images.insert((*aux_tool).to_string(), image);
            }
        }
        let exec_plan = StagePlan {
            stage_id: planned.stage.0.clone(),
            stage_version: stage_version_i32(planned.stage_version),
            tool: tool.clone(),
            tool_version: catalog
                .get(&tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?
                .version
                .clone(),
            image,
            runner: platform.runner,
            inputs: planned.inputs.clone(),
            out_dir: planned.out_dir.clone(),
            outputs: planned.outputs.clone(),
            params: planned.plan.parameters.clone(),
            aux_images,
        };
        let execution = execute_plan(&exec_plan)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: planned.stage.0.clone(),
                tool,
                reason: format!("tool failed with status {}", execution.exit_code),
            });
        }
    }

    if !failures.is_empty() {
        return Err(anyhow!(
            "preprocess pipeline failed: {} failures",
            failures.len()
        ));
    }

    let _ = if args.r2.is_some() {
        FastqLayout::PairedEnd
    } else {
        FastqLayout::SingleEnd
    };

    Ok(())
}

fn select_preprocess_tools(
    registry: &bijux_core::ToolRegistry,
    pipeline: &bijux_core::domain::PipelineSpec,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<Vec<String>> {
    let defaults = canonical_tool_defaults();
    let mut selected_tools: Vec<String> = pipeline
        .stages
        .iter()
        .map(|stage| {
            defaults
                .get(stage.as_str())
                .map(|tool| (*tool).to_string())
                .or_else(|| {
                    registry
                        .tools_for_stage(stage)
                        .first()
                        .map(|tool| tool.tool_id.clone())
                })
                .ok_or_else(|| anyhow!("no default tool for stage {stage}"))
        })
        .collect::<Result<_>>()?;

    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bench_corpus(corpus_id);
        let objective = bijux_analyze::selection::objective_spec(args.objective);
        let mut selections = Vec::new();
        for stage in &pipeline.stages {
            let tool_ids: Vec<String> = registry
                .tools_for_stage(stage)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = bijux_stages_fastq::get_results(stage, tool, &corpus, &args.out)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_analyze::selection::select_stage(
                stage,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            if selection.selected.is_none() {
                return Err(anyhow!(
                    "no eligible tools for {stage}; check bench corpus/results"
                ));
            }
            selections.push(selection);
        }
        selected_tools = selections
            .into_iter()
            .filter_map(|selection| selection.selected)
            .collect();
    }

    Ok(selected_tools)
}

/// Run the stats-neutral pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqStatsMetrics>> {
    crate::fastq_stats_neutral::bench_fastq_stats_neutral(catalog, platform, runner_override, args)
}
