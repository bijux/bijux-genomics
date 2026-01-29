use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_validate_v1, insert_fastq_validate_v1, metric_set, BenchmarkContext,
    BenchmarkRecord, FastqValidateMetrics,
};
use bijux_core::measure::ExecutionMetrics;
use bijux_core::ToolRole;
use bijux_engine::api::{ensure_bench_runner, load_registry};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use bijux_domain_fastq::{inspect_headers, log_header_warnings, preflight_stage, FastqArtifact};
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{cleanup_execution, execution_memory_mb, run_validate_execution};
use bijux_engine::api::{
    hash_file_sha256, input_fastq_stats, parse_fastqvalidator_count, SeqkitMetrics,
};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};

use crate::fastq_exec::helpers::{
    compute_run_id, normalize_validate_tool_list, params_hash, prepare_tool_run_dirs,
    resolve_image_for_run, write_execution_logs, write_explain_md, write_explain_plan_json,
    write_metrics_json, write_retention_report_placeholder, write_run_manifest, ExecutionManifest,
};
use crate::fastq_exec::helpers::{filter_tools_by_role, BenchOutcome};
use bijux_domain_fastq::RawFailure;

/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_validate_pre<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_domain_fastq::args::BenchFastqValidateArgs,
) -> Result<BenchOutcome<FastqValidateMetrics>> {
    let tools = normalize_validate_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.validate_pre", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, args.strict)?;
    log_header_warnings("fastq.validate_pre", &header);
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.validate_pre", &tools, &registry, args.strict)?;
    if args.strict
        && !tools.iter().any(|tool| {
            registry
                .tool_by_id("fastq.validate_pre", tool)
                .is_some_and(|manifest| manifest.role == ToolRole::Authoritative)
        })
    {
        return Err(anyhow!(
            "strict validation requires at least one authoritative tool"
        ));
    }
    let tool_policies = build_validate_tool_policy(&registry);
    let bench_inputs = prepare_validate_bench(catalog, platform, runner_override, args)?;
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.validate_pre")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(
        &bench_inputs.bench_dir,
        "fastq.validate_pre",
        &selected,
        &excluded,
        None,
    )?;
    write_explain_plan_json(
        &bench_inputs.bench_dir,
        "fastq.validate_pre",
        &selected,
        &registry,
        None,
    )?;
    ensure_image_qa_passed("fastq.validate_pre", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.validate_pre", &tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqValidateMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqValidateMetrics>> = Vec::new();
    let mut failures: Vec<RawFailure> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let cached = fetch_fastq_validate_v1(
            &conn,
            &tool,
            &spec.version,
            &image_digest,
            &bench_inputs.input_hash,
        );
        if let Ok(Some(record)) = cached {
            records.push(record);
            continue;
        }
        let policy = tool_policies
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
        match run_validate_tool(catalog, platform, args, &bench_inputs, &tool, *policy) {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(RawFailure {
                stage: "fastq.validate_pre".to_string(),
                tool: tool.to_string(),
                reason: err.to_string(),
            }),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_validate_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_validate_comparability(&records);
    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

struct ValidateBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_validate_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_domain_fastq::args::BenchFastqValidateArgs,
) -> Result<ValidateBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir = bench_base_dir(&args.out, "validate", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "validate", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_validate_tool_list(&args.tools)?.join(", ")
    );

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;

    let input_hash = hash_file_sha256(&r1)?;
    let input_stats = input_fastq_stats(&seqkit_image, &r1_dir, &r1)?;

    Ok(ValidateBenchInputs {
        runner,
        r1,
        r1_dir,
        input_hash,
        input_stats,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_lines)]
fn run_validate_tool<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_domain_fastq::args::BenchFastqValidateArgs,
    bench_inputs: &ValidateBenchInputs,
    tool: &str,
    policy: ToolPolicy,
) -> Result<BenchmarkRecord<FastqValidateMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ validate {tool}");
    let params = serde_json::json!({
        "sample_id": args.sample_id,
        "r1": bench_inputs.r1,
    });
    let param_hash = params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let run_id = compute_run_id(
        "fastq.validate_pre",
        tool,
        &image_digest,
        &bench_inputs.input_hash,
        &param_hash,
    );
    let run_dirs = prepare_tool_run_dirs(&bench_inputs.tools_root, tool, &run_id)?;
    let out_dir = run_dirs.artifacts_dir.clone();
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_validate_execution(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &out_dir,
        &container_name,
    )?;
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = execution_memory_mb(&container_name)?;
    cleanup_execution(&container_name)?;

    if execution.output_fastq.is_some() {
        return Err(anyhow!("fastq.validate_pre must not output FASTQ data"));
    }
    if args.strict && policy.role == ToolRole::Authoritative && execution.exit_code != 0 {
        return Err(anyhow!("strict validation failed for {tool}"));
    }

    let reads_total = validate_reads_total(tool, &bench_inputs.input_stats, &execution.stdout)?;
    let reads_valid = if execution.exit_code == 0 {
        reads_total
    } else {
        0
    };
    let reads_invalid = reads_total.saturating_sub(reads_valid);
    let metrics = FastqValidateMetrics {
        reads_total,
        reads_valid,
        reads_invalid,
        mean_q: bench_inputs.input_stats.mean_q,
    };
    let metric_set = metric_set(metrics);
    bijux_analyze::validate_metric_set(&metric_set)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let manifest = registry
        .tool_by_id("fastq.validate_pre", tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
    validate_execution_outputs(&manifest.execution_contract, &out_dir)?;
    let execution_manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.validate_pre".to_string(),
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest: image_digest.clone(),
        command: execution.command.clone(),
        input_hashes: vec![bench_inputs.input_hash.clone()],
        input_files: vec![bench_inputs.r1.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        arch: platform.arch.clone(),
    };
    fs::write(
        &run_dirs.manifest_path,
        serde_json::to_vec_pretty(&execution_manifest)?,
    )
    .context("write execution manifest")?;
    write_execution_logs(&run_dirs, &execution.stdout, &execution.stderr)?;
    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone(),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics: metric_set,
    };
    record.validate()?;
    let envelope = &record.metrics;
    write_metrics_json(&run_dirs, &record.execution, envelope)?;
    write_retention_report_placeholder(&run_dirs, "fastq.validate_pre", tool, &params)?;
    let adapter_bank_path = bijux_domain_fastq::adapter_bank_path();
    write_run_manifest(&run_dirs, "fastq.validate_pre", tool, &adapter_bank_path)?;
    if execution.exit_code != 0 {
        return Err(anyhow!(
            "tool {tool} failed with status {} (stdout: {}, stderr: {})",
            execution.exit_code,
            execution.stdout.trim(),
            execution.stderr.trim()
        ));
    }
    Ok(record)
}

#[derive(Debug, Clone, Copy)]
struct ToolPolicy {
    role: ToolRole,
}

fn build_validate_tool_policy(
    registry: &bijux_core::ToolRegistry,
) -> std::collections::HashMap<String, ToolPolicy> {
    let mut policies = std::collections::HashMap::new();
    for tool in registry.tools_for_stage("fastq.validate_pre") {
        policies.insert(tool.tool_id.clone(), ToolPolicy { role: tool.role });
    }
    policies
}

fn validate_reads_total(tool: &str, input_stats: &SeqkitMetrics, stdout: &str) -> Result<u64> {
    let reads_total = match tool {
        "seqtk" | "fastqc" => input_stats.reads,
        "fastqvalidator" | "fastqvalidator_official" => match parse_fastqvalidator_count(stdout) {
            Ok(count) => count,
            Err(err) => {
                tracing::warn!(error = %err, "fastqvalidator count missing; falling back to input reads");
                input_stats.reads
            }
        },
        "fqtools" => stdout
            .lines()
            .next()
            .ok_or_else(|| anyhow!("fqtools output missing"))?
            .parse::<u64>()?,
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };
    Ok(reads_total)
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::validate_reads_total;
    use anyhow::Result;
    use bijux_core::measure::SeqkitMetrics;

    #[test]
    fn validate_reads_total_uses_input_for_fastqc() -> Result<()> {
        let input = SeqkitMetrics {
            reads: 12,
            bases: 120,
            mean_q: 30.0,
            gc_percent: 50.0,
        };
        let count = validate_reads_total("fastqc", &input, "")?;
        assert_eq!(count, 12);
        Ok(())
    }

    #[test]
    fn validate_reads_total_parses_fqtools() -> Result<()> {
        let input = SeqkitMetrics {
            reads: 1,
            bases: 10,
            mean_q: 30.0,
            gc_percent: 50.0,
        };
        let count = validate_reads_total("fqtools", &input, "42\n")?;
        assert_eq!(count, 42);
        Ok(())
    }

    #[test]
    fn validate_reads_total_rejects_unknown_tool() {
        let input = SeqkitMetrics {
            reads: 1,
            bases: 10,
            mean_q: 30.0,
            gc_percent: 50.0,
        };
        match validate_reads_total("mystery", &input, "") {
            Ok(_) => panic!("expected unsupported tool"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }
}

fn check_fastq_validate_comparability(records: &[BenchmarkRecord<FastqValidateMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_total = first.metrics.metrics.reads_total;
    let mut mean_q = first.metrics.metrics.mean_q;

    for record in records.iter().skip(1) {
        if record.metrics.metrics.reads_total != reads_total {
            tracing::warn!(
                tool = record.context.tool,
                reads_total = record.metrics.metrics.reads_total,
                "reads_total differs from baseline"
            );
            reads_total = record.metrics.metrics.reads_total;
        }
        if (record.metrics.metrics.mean_q - mean_q).abs() > 1e-6 {
            tracing::warn!(
                tool = record.context.tool,
                mean_q = record.metrics.metrics.mean_q,
                "mean_q differs from baseline"
            );
            mean_q = record.metrics.metrics.mean_q;
        }
        if record.metrics.metrics.reads_invalid > 0 {
            tracing::warn!(
                tool = record.context.tool,
                reads_invalid = record.metrics.metrics.reads_invalid,
                "reads_invalid is non-zero"
            );
        }
    }
}
