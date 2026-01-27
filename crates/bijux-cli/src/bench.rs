use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, insert_fastq_filter_v1, insert_fastq_merge_v1, insert_fastq_trim_v1,
    insert_fastq_validate_v1, metric_kind_for_stage, open_sqlite, stage_metric_spec,
    BenchmarkContext, BenchmarkRecord, ExecutionMetrics, FastqFilterMetrics, FastqMergeMetrics,
    FastqTrimMetrics, FastqValidateMetrics, StageMetricSchema,
};
use bijux_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, RunnerKind, ToolImageSpec,
};
use tracing::warn;
use uuid::Uuid;

use crate::cli::{BenchFastqFilterArgs, BenchFastqMergeArgs, BenchFastqValidateArgs};
use crate::utils::{
    bench_base_dir, bench_tools_dir, docker_rm, docker_stats_mb, hash_file_sha256,
    input_fastq_stats, output_fastq_stats, parse_fastqvalidator_count, run_merge_container,
    run_tool_container, run_validate_container, SeqkitMetrics,
};

pub fn bench_fastq_trim(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqTrimArgs,
) -> Result<()> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let tools = normalize_tool_list(&args.tools)?;
    let bench_dir = bench_base_dir(&args.out, "trim", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "trim", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!("planned tools: {}", tools.join(", "));

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

    let mut records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;

        let tool_dir = tools_root.join(&tool);
        let out_dir = tool_dir.join("out");
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let start = Instant::now();
        let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
        let execution = run_tool_container(&tool, &image, &r1_dir, &r1, &out_dir, &container_name)?;
        let runtime_s = start.elapsed().as_secs_f64();
        let memory_mb = docker_stats_mb(&container_name)?;
        docker_rm(&container_name)?;

        let out_fastq = execution
            .output_fastq
            .as_ref()
            .ok_or_else(|| anyhow!("output fastq missing"))?;
        let output_stats = output_fastq_stats(&seqkit_image, &out_dir, out_fastq)?;

        let metrics = FastqTrimMetrics {
            reads_in: input_stats.reads,
            reads_out: output_stats.reads,
            bases_in: input_stats.bases,
            bases_out: output_stats.bases,
            mean_q_before: input_stats.mean_q,
            mean_q_after: output_stats.mean_q,
        };
        metrics.validate()?;

        let image_digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .to_string();
        let context = BenchmarkContext {
            tool: tool.clone(),
            tool_version: spec.version.clone(),
            image_digest,
            runner: runner.to_string(),
            platform: platform.name.clone(),
            input_hash: input_hash.clone(),
            parameters: serde_json::json!({
                "sample_id": args.sample_id,
                "r1": r1,
            }),
        };
        let execution_metrics = ExecutionMetrics {
            runtime_s,
            memory_mb,
            exit_code: execution.exit_code,
        };
        let record = BenchmarkRecord {
            context,
            execution: execution_metrics,
            metrics,
        };
        record.validate()?;
        records.push(record);
    }

    let bench_path = bench_dir.join("bench.jsonl");
    for record in &records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = open_sqlite(&sqlite_path).context("open bench sqlite")?;
    for record in &records {
        insert_fastq_trim_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_trim_comparability(&records);
    write_report(&bench_dir, &records)?;
    Ok(())
}

pub fn bench_fastq_validate(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqValidateArgs,
) -> Result<()> {
    let tools = normalize_validate_tool_list(&args.tools)?;
    let bench_inputs = prepare_validate_bench(catalog, platform, runner_override, args)?;
    let mut records: Vec<BenchmarkRecord<FastqValidateMetrics>> = Vec::new();

    for tool in tools {
        let record = run_validate_tool(catalog, platform, args, &bench_inputs, &tool)?;
        records.push(record);
    }

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = open_sqlite(&sqlite_path).context("open bench sqlite")?;
    for record in &records {
        insert_fastq_validate_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_validate_comparability(&records);
    write_validate_report(&bench_inputs.bench_dir, &records)?;
    Ok(())
}

pub fn bench_fastq_filter(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqFilterArgs,
) -> Result<()> {
    let tools = normalize_filter_tool_list(&args.tools)?;
    let bench_inputs = prepare_filter_bench(catalog, platform, runner_override, args)?;
    let mut records: Vec<BenchmarkRecord<FastqFilterMetrics>> = Vec::new();

    for tool in tools {
        let record = run_filter_tool(catalog, platform, args, &bench_inputs, &tool)?;
        records.push(record);
    }

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = open_sqlite(&sqlite_path).context("open bench sqlite")?;
    for record in &records {
        insert_fastq_filter_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_filter_comparability(&records);
    write_filter_report(&bench_inputs.bench_dir, &records)?;
    Ok(())
}

pub fn bench_fastq_merge(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqMergeArgs,
) -> Result<()> {
    let tools = normalize_merge_tool_list(&args.tools)?;
    let bench_inputs = prepare_merge_bench(catalog, platform, runner_override, args)?;
    let mut records: Vec<BenchmarkRecord<FastqMergeMetrics>> = Vec::new();

    for tool in tools {
        let record = run_merge_tool(catalog, platform, args, &bench_inputs, &tool)?;
        records.push(record);
    }

    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    for record in &records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = open_sqlite(&sqlite_path).context("open bench sqlite")?;
    for record in &records {
        insert_fastq_merge_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_merge_comparability(&records);
    write_merge_report(&bench_inputs.bench_dir, &records)?;
    Ok(())
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

struct FilterBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

struct MergeBenchInputs {
    runner: RunnerKind,
    r1: PathBuf,
    r2: PathBuf,
    r1_dir: PathBuf,
    input_hash: String,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: SeqkitMetrics,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

fn prepare_validate_bench(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqValidateArgs,
) -> Result<ValidateBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
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

fn prepare_filter_bench(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqFilterArgs,
) -> Result<FilterBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "filter", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "filter", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_filter_tool_list(&args.tools)?.join(", ")
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

    Ok(FilterBenchInputs {
        runner,
        r1,
        r1_dir,
        input_hash,
        input_stats,
        bench_dir,
        tools_root,
    })
}

fn prepare_merge_bench(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqMergeArgs,
) -> Result<MergeBenchInputs> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let bench_dir = bench_base_dir(&args.out, "merge", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "merge", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!(
        "planned tools: {}",
        normalize_merge_tool_list(&args.tools)?.join(", ")
    );

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r2 = args.r2.canonicalize().context("resolve r2 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;

    let input_hash = hash_file_sha256(&r1)?;
    let input_stats_r1 = input_fastq_stats(&seqkit_image, &r1_dir, &r1)?;
    let input_stats_r2 = input_fastq_stats(&seqkit_image, &r1_dir, &r2)?;

    Ok(MergeBenchInputs {
        runner,
        r1,
        r2,
        r1_dir,
        input_hash,
        input_stats_r1,
        input_stats_r2,
        bench_dir,
        tools_root,
    })
}

fn run_validate_tool(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    args: &BenchFastqValidateArgs,
    bench_inputs: &ValidateBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqValidateMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ validate {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let out_dir = tool_dir.join("out");
    fs::create_dir_all(&out_dir).context("create tool output dir")?;
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_validate_container(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &out_dir,
        &container_name,
    )?;
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = docker_stats_mb(&container_name)?;
    docker_rm(&container_name)?;

    let reads_total =
        validate_reads_total(tool, &bench_inputs.input_stats, &execution.stdout, &out_dir)?;
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
    metrics.validate()?;

    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: serde_json::json!({
            "sample_id": args.sample_id,
            "r1": bench_inputs.r1,
        }),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics,
    };
    record.validate()?;
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

fn run_filter_tool(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    args: &BenchFastqFilterArgs,
    bench_inputs: &FilterBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqFilterMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ filter {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let out_dir = tool_dir.join("out");
    fs::create_dir_all(&out_dir).context("create tool output dir")?;
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_tool_container(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &out_dir,
        &container_name,
    )?;
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = docker_stats_mb(&container_name)?;
    docker_rm(&container_name)?;

    let out_fastq = execution
        .output_fastq
        .as_ref()
        .ok_or_else(|| anyhow!("output fastq missing"))?;
    let output_stats = output_fastq_stats(
        &resolve_image_for_run(
            catalog
                .get("seqkit")
                .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?,
            platform,
        )?,
        &out_dir,
        out_fastq,
    )?;

    let reads_in = bench_inputs.input_stats.reads;
    let reads_out = output_stats.reads;
    let reads_dropped = reads_in.saturating_sub(reads_out);
    let metrics = FastqFilterMetrics {
        reads_in,
        reads_out,
        reads_dropped,
        mean_q_before: bench_inputs.input_stats.mean_q,
        mean_q_after: output_stats.mean_q,
    };
    metrics.validate()?;

    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: serde_json::json!({
            "sample_id": args.sample_id,
            "r1": bench_inputs.r1,
        }),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics,
    };
    record.validate()?;
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

fn run_merge_tool(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    args: &BenchFastqMergeArgs,
    bench_inputs: &MergeBenchInputs,
    tool: &str,
) -> Result<BenchmarkRecord<FastqMergeMetrics>> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;

    println!("→ merge {tool}");
    let tool_dir = bench_inputs.tools_root.join(tool);
    let out_dir = tool_dir.join("out");
    fs::create_dir_all(&out_dir).context("create tool output dir")?;
    let start = Instant::now();
    let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
    let execution = run_merge_container(
        tool,
        &image,
        &bench_inputs.r1_dir,
        &bench_inputs.r1,
        &bench_inputs.r2,
        &out_dir,
        &container_name,
    )?;
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = docker_stats_mb(&container_name)?;
    docker_rm(&container_name)?;

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;

    let merged_stats = output_fastq_stats(&seqkit_image, &out_dir, &execution.merged_fastq)?;
    let unmerged_r1_stats = output_fastq_stats(&seqkit_image, &out_dir, &execution.unmerged_r1)?;
    let unmerged_r2_stats = output_fastq_stats(&seqkit_image, &out_dir, &execution.unmerged_r2)?;

    let reads_r1 = bench_inputs.input_stats_r1.reads;
    let reads_r2 = bench_inputs.input_stats_r2.reads;
    let reads_merged = merged_stats.reads;
    let reads_unmerged = unmerged_r1_stats.reads.min(unmerged_r2_stats.reads);
    if unmerged_r1_stats.reads != unmerged_r2_stats.reads {
        warn!(
            tool = tool,
            unmerged_r1 = unmerged_r1_stats.reads,
            unmerged_r2 = unmerged_r2_stats.reads,
            "unmerged read counts differ between r1 and r2"
        );
    }
    let min_reads = reads_r1.min(reads_r2);
    let merge_rate = if min_reads > 0 {
        ratio_u64(reads_merged, min_reads)
    } else {
        0.0
    };

    let metrics = FastqMergeMetrics {
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        merge_rate,
    };
    metrics.validate()?;

    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: serde_json::json!({
            "sample_id": args.sample_id,
            "r1": bench_inputs.r1,
            "r2": bench_inputs.r2,
        }),
    };
    let execution_metrics = ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let record = BenchmarkRecord {
        context,
        execution: execution_metrics,
        metrics,
    };
    record.validate()?;
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

fn validate_reads_total(
    tool: &str,
    input_stats: &SeqkitMetrics,
    stdout: &str,
    _out_dir: &Path,
) -> Result<u64> {
    match tool {
        "fastqvalidator" => parse_fastqvalidator_count(stdout),
        "seqtk" | "fastqc" | "fqtools" => Ok(input_stats.reads),
        _ => Err(anyhow!("unsupported tool: {tool}")),
    }
}

fn normalize_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for tool in tools {
        let name = tool.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }
        if name != "fastp"
            && name != "cutadapt"
            && name != "bbduk"
            && name != "adapterremoval"
            && name != "trimmomatic"
            && name != "trim_galore"
        {
            return Err(anyhow!("unsupported tool: {name}"));
        }
        result.push(name);
    }
    if result.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    Ok(result)
}

fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for tool in tools {
        let name = tool.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }
        if name != "seqtk" && name != "fastqc" && name != "fastqvalidator" && name != "fqtools" {
            return Err(anyhow!("unsupported tool: {name}"));
        }
        result.push(name);
    }
    if result.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    Ok(result)
}

fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for tool in tools {
        let name = tool.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }
        if name != "bbduk" {
            return Err(anyhow!("unsupported tool: {name}"));
        }
        result.push(name);
    }
    if result.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    Ok(result)
}

fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for tool in tools {
        let name = tool.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }
        if name != "pear" {
            return Err(anyhow!("unsupported tool: {name}"));
        }
        result.push(name);
    }
    if result.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    Ok(result)
}

fn check_fastq_trim_comparability(records: &[BenchmarkRecord<FastqTrimMetrics>]) {
    if records.is_empty() {
        return;
    }
    let first = &records[0];
    let input_hash = &first.context.input_hash;
    let reads_in = first.metrics.reads_in;
    let bases_in = first.metrics.bases_in;
    let mean_q_before = first.metrics.mean_q_before;

    for record in records {
        if record.context.input_hash != *input_hash {
            warn!(
                tool = record.context.tool,
                input_hash = record.context.input_hash,
                expected = input_hash,
                "input hash mismatch across tools"
            );
        }
        if record.metrics.reads_in != reads_in {
            warn!(
                tool = record.context.tool,
                reads_in = record.metrics.reads_in,
                expected = reads_in,
                "reads_in differs across tools"
            );
        }
        if record.metrics.bases_in != bases_in {
            warn!(
                tool = record.context.tool,
                bases_in = record.metrics.bases_in,
                expected = bases_in,
                "bases_in differs across tools"
            );
        }
        if (record.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.mean_q_before,
                expected = mean_q_before,
                "mean_q_before differs across tools"
            );
        }
        if record.metrics.reads_in > 0 {
            #[allow(clippy::cast_precision_loss)]
            let loss = 1.0 - (record.metrics.reads_out as f64 / record.metrics.reads_in as f64);
            if loss >= 0.9 {
                warn!(
                    tool = record.context.tool,
                    reads_in = record.metrics.reads_in,
                    reads_out = record.metrics.reads_out,
                    loss = loss,
                    "extreme read loss detected"
                );
            }
        } else {
            warn!(
                tool = record.context.tool,
                "reads_in is zero; comparability checks skipped"
            );
        }
    }
}

fn check_fastq_validate_comparability(records: &[BenchmarkRecord<FastqValidateMetrics>]) {
    if records.is_empty() {
        return;
    }
    let first = &records[0];
    let input_hash = &first.context.input_hash;
    let reads_total = first.metrics.reads_total;
    let mean_q = first.metrics.mean_q;

    for record in records {
        if record.context.input_hash != *input_hash {
            warn!(
                tool = record.context.tool,
                input_hash = record.context.input_hash,
                expected = input_hash,
                "input hash mismatch across tools"
            );
        }
        if record.metrics.reads_total != reads_total {
            warn!(
                tool = record.context.tool,
                reads_total = record.metrics.reads_total,
                expected = reads_total,
                "reads_total differs across tools"
            );
        }
        if (record.metrics.mean_q - mean_q).abs() > 1e-6 {
            warn!(
                tool = record.context.tool,
                mean_q = record.metrics.mean_q,
                expected = mean_q,
                "mean_q differs across tools"
            );
        }
        if record.metrics.reads_invalid > 0 {
            warn!(
                tool = record.context.tool,
                reads_invalid = record.metrics.reads_invalid,
                "invalid reads reported by validation tool"
            );
        }
    }
}

fn check_fastq_filter_comparability(records: &[BenchmarkRecord<FastqFilterMetrics>]) {
    if records.is_empty() {
        return;
    }
    let first = &records[0];
    let input_hash = &first.context.input_hash;
    let reads_in = first.metrics.reads_in;
    let mean_q_before = first.metrics.mean_q_before;

    for record in records {
        if record.context.input_hash != *input_hash {
            warn!(
                tool = record.context.tool,
                input_hash = record.context.input_hash,
                expected = input_hash,
                "input hash mismatch across tools"
            );
        }
        if record.metrics.reads_in != reads_in {
            warn!(
                tool = record.context.tool,
                reads_in = record.metrics.reads_in,
                expected = reads_in,
                "reads_in differs across tools"
            );
        }
        if (record.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.mean_q_before,
                expected = mean_q_before,
                "mean_q_before differs across tools"
            );
        }
        if record.metrics.reads_out > record.metrics.reads_in {
            warn!(
                tool = record.context.tool,
                reads_in = record.metrics.reads_in,
                reads_out = record.metrics.reads_out,
                "filter should not increase reads"
            );
        }
    }
}

fn check_fastq_merge_comparability(records: &[BenchmarkRecord<FastqMergeMetrics>]) {
    if records.is_empty() {
        return;
    }
    let first = &records[0];
    let input_hash = &first.context.input_hash;
    let reads_r1 = first.metrics.reads_r1;
    let reads_r2 = first.metrics.reads_r2;

    for record in records {
        if record.context.input_hash != *input_hash {
            warn!(
                tool = record.context.tool,
                input_hash = record.context.input_hash,
                expected = input_hash,
                "input hash mismatch across tools"
            );
        }
        if record.metrics.reads_r1 != reads_r1 {
            warn!(
                tool = record.context.tool,
                reads_r1 = record.metrics.reads_r1,
                expected = reads_r1,
                "reads_r1 differs across tools"
            );
        }
        if record.metrics.reads_r2 != reads_r2 {
            warn!(
                tool = record.context.tool,
                reads_r2 = record.metrics.reads_r2,
                expected = reads_r2,
                "reads_r2 differs across tools"
            );
        }
        let min_reads = record.metrics.reads_r1.min(record.metrics.reads_r2);
        if record.metrics.reads_merged > min_reads {
            warn!(
                tool = record.context.tool,
                reads_merged = record.metrics.reads_merged,
                min_reads = min_reads,
                "merge should not exceed input pairs"
            );
        }
    }
}

fn write_report(base_dir: &Path, records: &[BenchmarkRecord<FastqTrimMetrics>]) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert(
        "derived_metrics",
        serde_json::to_value(records.iter().map(derived_trim_metrics).collect::<Vec<_>>())?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    Ok(())
}

fn resolve_image_for_run(spec: &ToolImageSpec, platform: &PlatformSpec) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!(
                "digest image missing locally; falling back to tag {}",
                fallback.full_name
            );
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
}

fn write_validate_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    Ok(())
}

pub fn print_bench_schema(stage: &str) -> Result<()> {
    let kind = metric_kind_for_stage(stage).ok_or_else(|| anyhow!("unknown stage {stage}"))?;
    let spec = stage_metric_spec(kind);
    let metrics: Vec<_> = spec
        .metrics
        .iter()
        .map(|metric| {
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
            })
        })
        .collect();
    let json = serde_json::json!({
        "stage": spec.stage,
        "metrics": metrics,
        "invariants": spec.invariants,
    });
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn write_filter_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqFilterMetrics>],
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert(
        "derived_metrics",
        serde_json::to_value(
            records
                .iter()
                .map(derived_filter_metrics)
                .collect::<Vec<_>>(),
        )?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    Ok(())
}

fn write_merge_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert(
        "derived_metrics",
        serde_json::to_value(
            records
                .iter()
                .map(derived_merge_metrics)
                .collect::<Vec<_>>(),
        )?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    Ok(())
}

fn derived_trim_metrics(record: &BenchmarkRecord<FastqTrimMetrics>) -> serde_json::Value {
    let reads_in = record.metrics.reads_in;
    let bases_in = record.metrics.bases_in;
    serde_json::json!({
        "tool": record.context.tool,
        "read_retention": if reads_in > 0 {
            ratio_u64(record.metrics.reads_out, reads_in)
        } else {
            0.0
        },
        "base_retention": if bases_in > 0 {
            ratio_u64(record.metrics.bases_out, bases_in)
        } else {
            0.0
        },
    })
}

fn derived_filter_metrics(record: &BenchmarkRecord<FastqFilterMetrics>) -> serde_json::Value {
    let reads_in = record.metrics.reads_in;
    serde_json::json!({
        "tool": record.context.tool,
        "read_retention": if reads_in > 0 {
            ratio_u64(record.metrics.reads_out, reads_in)
        } else {
            0.0
        },
    })
}

fn derived_merge_metrics(record: &BenchmarkRecord<FastqMergeMetrics>) -> serde_json::Value {
    let min_reads = record.metrics.reads_r1.min(record.metrics.reads_r2);
    serde_json::json!({
        "tool": record.context.tool,
        "merge_efficiency": if min_reads > 0 {
            ratio_u64(record.metrics.reads_merged, min_reads)
        } else {
            0.0
        },
    })
}

#[allow(clippy::cast_precision_loss)]
fn ratio_u64(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
