use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::BenchmarkRecord;
use bijux_core::ContainerImageRefV1;
use bijux_engine::api::resolve_image_for_run;
use bijux_engine::api::{
    bench_base_dir, bench_tools_dir, build_tool_execution_spec, execute_plan, PlatformSpec,
    RunnerKind, ToolImageSpec,
};
use bijux_engine::api::{ensure_bench_runner, filter_tools_by_role, load_registry};
use bijux_engine::api::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_stages_fastq::fastq::correct::{normalize_correct_tool_list, plan_correct};
use bijux_stages_fastq::fastq::filter::{
    normalize_filter_tool_list, plan_filter, FilterPlanOptions,
};
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
    log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_stages_fastq::{FastqArtifact, FastqLayout};

mod banks;
mod explain;
mod summary;

use banks::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};
pub use explain::{write_explain_md, write_explain_plan_json};
use summary::{write_run_summary, StageExecutionSummary};
pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: std::path::PathBuf,
    pub explain: bool,
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

    let adapter_bank = adapter_bank_context(
        args.adapter_bank_preset.as_deref(),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_bank = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_bank = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.trim", tool, &registry, catalog, platform)?;
        if let Some(msg) = polyx_unsupported_warning(
            &tool_spec.tool_id.0,
            polyx_bank.as_ref(),
            args.polyx_preset.is_some(),
        ) {
            eprintln!("{msg}");
        }
        let plan = plan(
            &tool_spec,
            &args.r1,
            &out_dir,
            adapter_bank.as_ref(),
            polyx_bank.as_ref(),
            contaminant_bank.as_ref(),
        )?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
        let tool_spec =
            build_tool_execution_spec("fastq.validate_pre", tool, &registry, catalog, platform)?;
        let plan = plan_validate_pre(&tool_spec, &args.r1, &out_dir);
        let execution = execute_plan(&plan, platform.runner, None)?;
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

    let filter_options = FilterPlanOptions {
        max_n: args.max_n,
        low_complexity_threshold: args.low_complexity_threshold,
        kmer_ref: args.kmer_ref.clone(),
        redundant_filters: Vec::new(),
    };
    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.filter", tool, &registry, catalog, platform)?;
        let plan = plan_filter(&tool_spec, &args.r1, &out_dir, &filter_options)?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
        let tool_spec =
            build_tool_execution_spec("fastq.merge", tool, &registry, catalog, platform)?;
        let plan = plan_merge(&tool_spec, &args.r1, &args.r2, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
        let tool_spec =
            build_tool_execution_spec("fastq.screen", tool, &registry, catalog, platform)?;
        let plan = plan_screen(&tool_spec, &args.r1, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
        let tool_spec = build_tool_execution_spec("fastq.umi", tool, &registry, catalog, platform)?;
        let plan = plan_umi(&tool_spec, &args.r1, r2, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
        let tool_spec =
            build_tool_execution_spec("fastq.correct", tool, &registry, catalog, platform)?;
        let plan = plan_correct(&tool_spec, &args.r1, r2, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
        let tool_spec =
            build_tool_execution_spec("fastq.qc_post", tool, &registry, catalog, platform)?;
        let mut aux_images = std::collections::BTreeMap::new();
        if tool == "multiqc" {
            for aux_tool in aux_tool_ids() {
                let spec = catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.yaml"))?;
                let image = resolve_image_for_run(spec, platform)?;
                aux_images.insert(
                    (*aux_tool).to_string(),
                    ContainerImageRefV1 {
                        image: image.full_name,
                        digest: spec.digest.clone(),
                    },
                );
            }
        }
        let plan = plan_qc_post(&tool_spec, &args.r1, &out_dir, aux_images, None)?;
        let execution = execute_plan(&plan, platform.runner, None)?;
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
#[allow(clippy::too_many_lines)]
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

    let adapter_bank = adapter_bank_context(
        args.adapter_bank_preset.as_deref(),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_bank = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_bank = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let mut failures = Vec::new();
    let mut tool_specs = Vec::new();
    for (stage, tool) in pipeline.stages.iter().zip(selected_tools.iter()) {
        let spec = build_tool_execution_spec(stage, tool, &registry, catalog, platform)?;
        if stage == "fastq.trim" {
            if let Some(msg) = polyx_unsupported_warning(
                &spec.tool_id.0,
                polyx_bank.as_ref(),
                args.polyx_preset.is_some(),
            ) {
                eprintln!("{msg}");
            }
        }
        tool_specs.push(spec);
    }
    let mut aux_tools = std::collections::BTreeMap::new();
    for aux_tool in bijux_stages_fastq::fastq::qc_post::aux_tool_ids() {
        let spec = catalog
            .get(*aux_tool)
            .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        aux_tools.insert(
            (*aux_tool).to_string(),
            ContainerImageRefV1 {
                image: image.full_name,
                digest: spec.digest.clone(),
            },
        );
    }
    let planned_stages = plan_preprocess_pipeline(
        &pipeline.stages,
        &tool_specs,
        &aux_tools,
        adapter_bank.as_ref(),
        polyx_bank.as_ref(),
        contaminant_bank.as_ref(),
        &args.r1,
        args.r2.as_deref(),
        |stage, tool, _r1, _r2| {
            let stage_dir = stage.trim_start_matches("fastq.");
            let stage_root = bench_tools_dir(&args.out, stage_dir, &args.sample_id);
            let out_dir = stage_root.join(&tool.tool_id.0);
            fs::create_dir_all(&out_dir).context("create stage output dir")?;
            Ok(out_dir)
        },
    )?;

    let telemetry = bijux_engine::api::build_telemetry_adapter();
    let mut pipeline_attrs = std::collections::BTreeMap::new();
    pipeline_attrs.insert("sample_id".to_string(), args.sample_id.clone());
    pipeline_attrs.insert("pipeline".to_string(), "fastq.preprocess".to_string());
    let pipeline_span = telemetry.start_pipeline("fastq.preprocess", &pipeline_attrs);

    let mut stage_runs = Vec::new();
    for planned in planned_stages {
        let stage_id = planned.stage_id.0.clone();
        let tool = planned.tool_id.0.clone();
        let mut stage_attrs = std::collections::BTreeMap::new();
        stage_attrs.insert("stage".to_string(), stage_id.clone());
        stage_attrs.insert("tool".to_string(), tool.clone());
        let stage_span = telemetry.start_stage(&stage_id, &stage_attrs);
        let execution = execute_plan(&planned, platform.runner, None);
        stage_span.end();
        let execution = execution?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: stage_id,
                tool: tool.clone(),
                reason: format!("tool failed with status {}", execution.exit_code),
            });
        }
        stage_runs.push(StageExecutionSummary {
            plan: planned,
            result: execution,
        });
    }
    pipeline_span.end();

    write_run_summary(&args.out, &stage_runs, &failures)?;
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
        let objective = bijux_core::selection::objective_spec(args.objective);
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
            let selection = bijux_core::selection::select_stage(
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

#[cfg(test)]
mod tests {
    use super::{fastq_preprocess_plan, polyx_unsupported_warning};
    use std::path::PathBuf;

    fn base_args() -> bijux_stages_fastq::args::BenchFastqPreprocessArgs {
        bijux_stages_fastq::args::BenchFastqPreprocessArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: None,
            out: PathBuf::from("out"),
            strict: false,
            auto: true,
            objective: bijux_core::selection::Objective::Balanced,
            bench_corpus: None,
            allow_partial: false,
            adapter_bank_preset: None,
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            no_qc_post: false,
        }
    }

    #[test]
    fn polyx_warning_emits_for_unsupported_tools() {
        let polyx_bank = serde_json::json!({
            "bank_id": "polyx.default",
            "preset": "illumina_twocolor",
        });
        let warning = polyx_unsupported_warning("cutadapt", Some(&polyx_bank), true);
        assert!(warning.is_some());
        let warning = polyx_unsupported_warning("fastp", Some(&polyx_bank), true);
        assert!(warning.is_none());
        let warning = polyx_unsupported_warning("cutadapt", None, true);
        assert!(warning.is_none());
        let warning = polyx_unsupported_warning("cutadapt", Some(&polyx_bank), false);
        assert!(warning.is_none());
    }

    #[test]
    fn preprocess_plan_single_end_has_no_merge() {
        let args = base_args();
        let plan = fastq_preprocess_plan(&args);
        assert!(plan.stages.contains(&"fastq.trim".to_string()));
        assert!(!plan.stages.contains(&"fastq.merge".to_string()));
    }

    #[test]
    fn preprocess_plan_paired_includes_merge() {
        let mut args = base_args();
        args.r2 = Some(PathBuf::from("reads_R2.fastq.gz"));
        let plan = fastq_preprocess_plan(&args);
        assert!(plan.stages.contains(&"fastq.merge".to_string()));
    }
}
