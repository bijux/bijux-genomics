use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_engine::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use bijux_engine::api::{
    bench_base_dir, bench_tools_dir, compute_run_id, execute_stage_plan, hash_file_sha256,
    params_hash, prepare_tool_run_dirs, resolve_image_for_run, write_execution_logs,
    write_stage_plan_json, ExecutionManifest, RunDirs, StagePlan as ExecPlan,
};

use bijux_core::{build_run_metadata_v1, RunMetadataV1, ToolInvocationV1};
use bijux_stages_fastq::FastqLayout;

use crate::fastq_exec::helpers::write_explain_plan_json;
use bijux_core::events::RunEvent;
use bijux_stages_fastq::{
    adapter_bank_path, append_event, assess_input_dir, bench_corpus, canonical_tool_defaults,
    create_run_layout, load_adapter_bank, now_string, update_run_index, write_input_assessment,
    write_run_metadata, AdapterBankV1, RunArtifactEntry, RunEnvironment, RunIndexEntry, RunLayout,
    RunManifest, RunStageEntry, ToolImageDigest,
};

struct StageExecMeta {
    run_id: String,
    run_dirs: RunDirs,
    params: serde_json::Value,
    input_hash: String,
    image: bijux_engine::api::ResolvedImage,
    image_digest: String,
    tool_version: String,
}

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    _runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    fastq_preprocess_run(catalog, platform, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Returns an error if any stage fails.
#[allow(clippy::too_many_lines)]
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;
    let started_at = chrono::Utc::now();
    let (run_id, layout) = create_run_layout(&args.out)?;
    let adapter_bank_path = args.adapter_bank.clone().unwrap_or_else(adapter_bank_path);
    let adapter_bank = load_adapter_bank(&adapter_bank_path)?;
    let input_dir = args
        .r1
        .parent()
        .map_or_else(|| args.out.clone(), PathBuf::from);
    let assessment = assess_input_dir(&input_dir)?;
    if layout.assessment_path.exists() {
        return Err(anyhow!(
            "input assessment already exists at {}",
            layout.assessment_path.display()
        ));
    }
    write_input_assessment(&layout.assessment_path, &assessment)?;
    let matched_sample = assessment
        .samples
        .iter()
        .find(|sample| sample.id.r1_path == args.r1);
    if args.r2.is_some()
        && matched_sample
            .as_ref()
            .and_then(|sample| sample.id.r2_path.clone())
            .is_none()
    {
        return Err(anyhow!(
            "input assessment did not find a paired R2 for the provided R1"
        ));
    }
    let derived_r2 = match (
        args.r2.clone(),
        matched_sample.and_then(|s| s.id.r2_path.clone()),
    ) {
        (Some(r2), _) | (None, Some(r2)) => Some(r2),
        (None, None) => None,
    };
    let layout_kind = if derived_r2.is_some() {
        FastqLayout::PairedEnd
    } else {
        FastqLayout::SingleEnd
    };
    let preprocess_plan = bijux_stages_fastq::fastq::preprocess::plan_preprocess(args);
    let pipeline = preprocess_plan.pipeline.clone();
    let explain = format!(
        "# Explain: fastq.preprocess\n\nPipeline:\n- {}",
        pipeline.stages.join("\n- ")
    );
    fs::write(out_dir.join("explain.md"), explain).context("write explain.md")?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&preprocess_plan);
    let plan_path = layout
        .run_dir
        .join("run_artifacts")
        .join("plans")
        .join("fastq_preprocess.plan.json");
    if let Some(parent) = plan_path.parent() {
        std::fs::create_dir_all(parent).context("create preprocess plan dir")?;
    }
    std::fs::write(&plan_path, serde_json::to_vec_pretty(&plan_json)?)
        .context("write preprocess plan json")?;
    let registry = bijux_engine::api::load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow::anyhow!("manifest validation failed: {err}"))?;
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
                .ok_or_else(|| anyhow::anyhow!("no default tool for stage {stage}"))
        })
        .collect::<Result<_>>()?;

    let mut objective_name: Option<String> = None;
    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow::anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bench_corpus(corpus_id);
        let objective = bijux_analyze::selection::objective_spec(args.objective);
        objective_name = Some(objective.name.clone());
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
                return Err(anyhow::anyhow!(
                    "no eligible tools for {stage}; check bench corpus/results"
                ));
            }
            selections.push(selection);
        }
        bijux_analyze::selection::write_selection_report(
            &out_dir,
            &objective,
            corpus_id.as_str(),
            selections.clone(),
        )?;
        selected_tools = selections
            .into_iter()
            .filter_map(|selection| selection.selected)
            .collect();
    }

    append_event(
        &layout,
        &RunEvent {
            timestamp: now_string(),
            event: "pipeline_started".to_string(),
            stage: None,
            tool: None,
            detail: Some("fastq.preprocess".to_string()),
        },
    )?;

    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;
    let mut stage_meta = Vec::new();
    let planned_stages = bijux_stages_fastq::fastq::preprocess::plan_preprocess_pipeline(
        &pipeline.stages,
        &selected_tools,
        &args.r1,
        derived_r2.as_deref(),
        |stage, tool, r1, r2| {
            let spec = catalog
                .get(tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
            let image = resolve_image_for_run(spec, platform)?;
            let params = serde_json::json!({
                "sample_id": args.sample_id,
                "r1": r1,
                "r2": r2,
                "adapter_preset": args.adapter_preset,
                "adapter_bank": args.adapter_bank,
                "enable_adapters": args.enable_adapters,
                "disable_adapters": args.disable_adapters,
            });
            let params_hash = params_hash(&params)?;
            let input_hash_r1 = hash_file_sha256(r1)?;
            let input_hash = match r2 {
                Some(r2) => {
                    let input_hash_r2 = hash_file_sha256(r2)?;
                    format!("{input_hash_r1},{input_hash_r2}")
                }
                None => input_hash_r1,
            };
            let run_id = compute_run_id(
                stage,
                tool,
                spec.digest.as_deref().unwrap_or("unknown"),
                &input_hash,
                &params_hash,
            );
            let stage_dir = stage.trim_start_matches("fastq.");
            let tools_root = bench_tools_dir(&args.out, stage_dir, &args.sample_id);
            let run_dirs = prepare_tool_run_dirs(&tools_root, tool, &run_id)?;
            let out_dir = run_dirs.artifacts_dir.clone();
            stage_meta.push(StageExecMeta {
                run_id,
                run_dirs,
                params,
                input_hash,
                image,
                image_digest: spec.digest.clone().unwrap_or_else(|| "unknown".to_string()),
                tool_version: spec.version.clone(),
            });
            Ok(out_dir)
        },
    )?;
    let mut stage_entries = Vec::with_capacity(planned_stages.len());
    for (step, meta) in planned_stages.iter().zip(stage_meta.iter()) {
        append_event(
            &layout,
            &RunEvent {
                timestamp: now_string(),
                event: "tool_selected".to_string(),
                stage: Some(step.stage.0.clone()),
                tool: Some(step.tool.0.clone()),
                detail: None,
            },
        )?;
        append_event(
            &layout,
            &RunEvent {
                timestamp: now_string(),
                event: "stage_started".to_string(),
                stage: Some(step.stage.0.clone()),
                tool: Some(step.tool.0.clone()),
                detail: None,
            },
        )?;
        let plan_name = format!("{}.plan.json", step.stage.0.replace('.', "_"));
        let _plan_path = write_stage_plan_json(&meta.run_dirs, &plan_name, &step.plan)?;

        let mut aux_images = HashMap::new();
        if step.stage.0 == bijux_stages_fastq::fastq::qc_post::STAGE_ID {
            for aux_tool in bijux_stages_fastq::fastq::qc_post::aux_tool_ids() {
                let spec = catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("{aux_tool} missing from images.yaml"))?;
                let image = resolve_image_for_run(spec, platform)?;
                aux_images.insert((*aux_tool).to_string(), image);
            }
        }

        let exec_plan = ExecPlan {
            stage_id: step.stage.0.clone(),
            tool: step.tool.0.clone(),
            image: meta.image.clone(),
            runner: platform.runner,
            inputs: step.inputs.clone(),
            out_dir: meta.run_dirs.artifacts_dir.clone(),
            outputs: step.outputs.clone(),
            params: meta.params.clone(),
            aux_images,
        };
        let execution = execute_stage_plan(&exec_plan)?;
        let manifest = ExecutionManifest {
            run_id: meta.run_id.clone(),
            stage: step.stage.0.clone(),
            tool: step.tool.0.clone(),
            tool_version: meta.tool_version.clone(),
            image_digest: meta.image_digest.clone(),
            command: execution.command,
            input_hashes: vec![meta.input_hash.clone()],
            input_files: exec_plan
                .inputs
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            output_dir: meta.run_dirs.artifacts_dir.display().to_string(),
            runner: platform.runner.to_string(),
            platform: platform.name.clone(),
            arch: platform.arch.clone(),
        };
        std::fs::write(
            &meta.run_dirs.manifest_path,
            serde_json::to_vec_pretty(&manifest)?,
        )
        .context("write execution manifest")?;
        write_execution_logs(&meta.run_dirs, &execution.stdout, &execution.stderr)?;
        stage_entries.push(RunStageEntry {
            stage_id: step.stage.0.clone(),
            tool_id: step.tool.0.clone(),
            execution_metrics_path: meta.run_dirs.metrics_path.clone(),
            domain_metrics_path: meta.run_dirs.metrics_path.clone(),
            logs_dir: meta.run_dirs.logs_dir.clone(),
            outputs_dir: meta.run_dirs.artifacts_dir.clone(),
            tool_invocation_path: meta.run_dirs.manifest_path.clone(),
        });
        append_event(
            &layout,
            &RunEvent {
                timestamp: now_string(),
                event: "stage_finished".to_string(),
                stage: Some(step.stage.0.clone()),
                tool: Some(step.tool.0.clone()),
                detail: None,
            },
        )?;
    }

    populate_run_layout(&layout, &mut stage_entries)?;

    let finished_at = chrono::Utc::now();
    let retention_report_path =
        write_retention_report(&layout.summary_dir, "fastq.preprocess", &adapter_bank)?;
    let env = RunEnvironment {
        hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        runner: platform.runner.to_string(),
        platform: platform.name.clone(),
        tool_images: selected_tools
            .iter()
            .filter_map(|tool| {
                catalog.get(tool).map(|spec| ToolImageDigest {
                    tool: tool.clone(),
                    image: format!(
                        "{}/{}:{}-{}",
                        platform.image_prefix, spec.tool, spec.version, platform.arch
                    ),
                    digest: spec.digest.clone().unwrap_or_else(|| "unknown".to_string()),
                })
            })
            .collect(),
    };
    bijux_stages_fastq::write_environment(&layout, &env)?;

    let artifacts = build_run_artifacts(
        &layout,
        &stage_entries,
        &retention_report_path,
        &adapter_bank_path,
    )?;
    let manifest = RunManifest {
        run_id: run_id.clone(),
        started_at: started_at.to_rfc3339(),
        finished_at: finished_at.to_rfc3339(),
        pipeline: "fastq.preprocess".to_string(),
        layout: layout_kind,
        stages: stage_entries,
        artifacts,
    };
    bijux_stages_fastq::write_manifest(&layout, &manifest)?;

    let deltas_path = layout.summary_dir.join("metrics_deltas.json");
    if !deltas_path.exists() {
        std::fs::write(&deltas_path, "{}")?;
    }
    let report_path = layout.summary_dir.join("report.json");
    if !report_path.exists() {
        std::fs::write(&report_path, "{}")?;
    }

    let platform_runner = platform.runner.to_string();
    let git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let metadata: RunMetadataV1 = build_run_metadata_v1(
        Uuid::parse_str(&run_id)?,
        started_at,
        finished_at,
        &platform_runner,
        "unknown",
        env!("CARGO_PKG_VERSION"),
        &git_commit,
    );
    write_run_metadata(&layout, &metadata)?;

    update_run_index(
        &args.out,
        RunIndexEntry {
            run_id,
            domain: "fastq".to_string(),
            pipeline: "fastq.preprocess".to_string(),
            stages: pipeline.stages,
            layout: layout_kind,
            tools: selected_tools,
            objective: objective_name,
            platform: platform.runner.to_string(),
            success: true,
        },
    )?;

    append_event(
        &layout,
        &RunEvent {
            timestamp: now_string(),
            event: "pipeline_finished".to_string(),
            stage: None,
            tool: None,
            detail: Some("fastq.preprocess".to_string()),
        },
    )?;

    Ok(())
}

fn populate_run_layout(layout: &RunLayout, entries: &mut [RunStageEntry]) -> Result<()> {
    for entry in entries {
        let stage_name = entry.stage_id.trim_start_matches("fastq.");
        let stage_dir = layout.stages_dir.join(stage_name);
        let outputs_dir = stage_dir.join("outputs");
        let logs_dir = stage_dir.join("logs");
        std::fs::create_dir_all(&outputs_dir).context("create stage outputs dir")?;
        std::fs::create_dir_all(&logs_dir).context("create stage logs dir")?;
        let execution_metrics_path = stage_dir.join("execution_metrics.json");
        let domain_metrics_path = stage_dir.join("metrics.json");
        let tool_invocation_path = stage_dir.join("tool_invocation.json");

        if entry.execution_metrics_path.exists() {
            let data = std::fs::read_to_string(&entry.execution_metrics_path)?;
            let payload: serde_json::Value = serde_json::from_str(&data)?;
            let execution = payload
                .get("execution")
                .cloned()
                .ok_or_else(|| anyhow!("missing execution metrics"))?;
            let metrics = payload
                .get("metrics")
                .cloned()
                .ok_or_else(|| anyhow!("missing domain metrics"))?;
            std::fs::write(
                &execution_metrics_path,
                serde_json::to_vec_pretty(&execution)?,
            )
            .context("write execution_metrics.json")?;
            std::fs::write(&domain_metrics_path, serde_json::to_vec_pretty(&metrics)?)
                .context("write metrics.json")?;
        }
        let source_run_dir = entry
            .execution_metrics_path
            .parent()
            .ok_or_else(|| anyhow!("missing run dir for metrics"))?;
        let manifest_path = source_run_dir.join("manifest.json");
        if manifest_path.exists() {
            let manifest_data = std::fs::read_to_string(&manifest_path)?;
            let manifest: bijux_engine::api::ExecutionManifest =
                serde_json::from_str(&manifest_data)?;
            let invocation = ToolInvocationV1 {
                stage: manifest.stage,
                tool: manifest.tool,
                version: manifest.tool_version,
                image: manifest.image_digest,
                command: manifest.command,
                threads: 0,
                inputs: manifest.input_files,
                outputs: vec![manifest.output_dir],
            };
            std::fs::write(
                &tool_invocation_path,
                serde_json::to_vec_pretty(&invocation)?,
            )
            .context("write tool_invocation.json")?;
        }
        let tool_log = entry.logs_dir.join("tool.log");
        if tool_log.exists() {
            std::fs::copy(&tool_log, logs_dir.join("tool.log"))
                .context("copy tool.log into run layout")?;
        }

        entry.execution_metrics_path = execution_metrics_path;
        entry.domain_metrics_path = domain_metrics_path;
        entry.logs_dir = logs_dir;
        entry.outputs_dir = outputs_dir;
        entry.tool_invocation_path = tool_invocation_path;
    }
    Ok(())
}

fn write_retention_report(
    summary_dir: &Path,
    pipeline: &str,
    adapter_bank: &AdapterBankV1,
) -> Result<PathBuf> {
    let report_path = summary_dir.join("retention_report.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.retention_report.v1",
        "definition": "unknown/TBD",
        "numerator": "unknown/TBD",
        "denominator": "unknown/TBD",
        "scope": "unknown/TBD",
        "stage_boundary": format!("{pipeline}:unknown/TBD"),
        "tool": {
            "id": "unknown/TBD",
            "stage": "unknown/TBD",
            "version": "unknown/TBD",
            "params": {
                "adapter_bank_schema": adapter_bank.schema_version,
                "adapter_bank_count": adapter_bank.adapters.len()
            }
        }
    });
    std::fs::write(&report_path, serde_json::to_vec_pretty(&payload)?)
        .context("write retention_report.json")?;
    Ok(report_path)
}

fn build_run_artifacts(
    layout: &RunLayout,
    entries: &[RunStageEntry],
    retention_report_path: &Path,
    adapter_bank_path: &Path,
) -> Result<Vec<RunArtifactEntry>> {
    let mut artifacts = Vec::new();

    let execution_manifest_path = layout.manifest_path.clone();
    let execution_manifest_hash = bijux_engine::api::hash_file_sha256(&execution_manifest_path)?;
    artifacts.push(RunArtifactEntry {
        name: "execution_manifest".to_string(),
        path: execution_manifest_path,
        sha256: execution_manifest_hash,
    });

    for entry in entries {
        let metrics_path = entry.domain_metrics_path.clone();
        let metrics_hash = bijux_engine::api::hash_file_sha256(&metrics_path)?;
        artifacts.push(RunArtifactEntry {
            name: format!("metrics:{}", entry.stage_id),
            path: metrics_path,
            sha256: metrics_hash,
        });
    }

    let retention_hash = bijux_engine::api::hash_file_sha256(retention_report_path)?;
    artifacts.push(RunArtifactEntry {
        name: "retention_report".to_string(),
        path: retention_report_path.to_path_buf(),
        sha256: retention_hash,
    });

    let adapter_hash = bijux_engine::api::hash_file_sha256(adapter_bank_path)?;
    artifacts.push(RunArtifactEntry {
        name: "adapter_bank".to_string(),
        path: adapter_bank_path.to_path_buf(),
        sha256: adapter_hash,
    });

    Ok(artifacts)
}
