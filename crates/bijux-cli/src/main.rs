use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_jsonl, insert_fastq_trim_v1, insert_fastq_validate_v1, open_sqlite, BenchmarkContext,
    BenchmarkRecord, ExecutionMetrics, FastqTrimMetrics, FastqValidateMetrics, StageMetricSchema,
};
use bijux_core::{
    build_execution_plan, load_manifests, load_profile, new_run_id, DryRunExecutor, Executor,
    PathSpec, RunSpec, StageId, ToolId,
};
use bijux_environment::api::{
    available_runners, cache_dir, docker_image_exists, load_image_catalog, load_platform,
    resolve_image, select_best_runner, PlatformSpec, ResolvedImage, RunnerKind, ToolImageSpec,
};
use clap::{Args, Parser, Subcommand};
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(name = "bijux", version, about = "Bijux DNA CLI")]
struct Cli {
    #[arg(long, default_value = "local")]
    profile: String,
    #[arg(long)]
    platform: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Fastq {
        #[command(subcommand)]
        command: FastqCommand,
    },
    ValidateManifests,
    Platform,
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    Bench {
        #[command(subcommand)]
        command: BenchCommand,
    },
}

#[derive(Debug, Subcommand)]
enum EnvCommand {
    Images,
    Info,
    Doctor,
}

#[derive(Debug, Subcommand)]
enum BenchCommand {
    Fastq {
        #[command(subcommand)]
        command: BenchFastqCommand,
    },
}

#[derive(Debug, Subcommand)]
enum BenchFastqCommand {
    Trim(BenchFastqTrimArgs),
    Validate(BenchFastqValidateArgs),
}

#[derive(Debug, Args)]
struct BenchFastqTrimArgs {
    #[arg(long, alias = "sample")]
    sample_id: String,
    #[arg(long)]
    r1: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    tools: Vec<String>,
}

#[derive(Debug, Args)]
struct BenchFastqValidateArgs {
    #[arg(long, alias = "sample")]
    sample_id: String,
    #[arg(long)]
    r1: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    tools: Vec<String>,
}

#[derive(Debug, Args, Clone, Default)]
struct CommonArgs {
    #[arg(long)]
    list_tools: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Subcommand)]
enum FastqCommand {
    Filter(CommonArgs),
    Merge(CommonArgs),
    Trim(FastqTrimArgs),
    Contam(CommonArgs),
    Preprocess(CommonArgs),
    Umi(CommonArgs),
    #[command(name = "error-correct")]
    ErrorCorrect(CommonArgs),
    Qc(CommonArgs),
    Validate(FastqValidateArgs),
    Align(CommonArgs),
}

#[derive(Debug, Args, Clone)]
struct FastqTrimArgs {
    #[command(flatten)]
    common: CommonArgs,
    #[arg(long)]
    env: Option<String>,
    #[arg(long, alias = "sample")]
    sample_id: Option<String>,
    #[arg(long)]
    r1: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    tools: Vec<String>,
}

#[derive(Debug, Args, Clone)]
struct FastqValidateArgs {
    #[command(flatten)]
    common: CommonArgs,
    #[arg(long)]
    env: Option<String>,
    #[arg(long, alias = "sample")]
    sample_id: Option<String>,
    #[arg(long)]
    r1: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    tools: Vec<String>,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let modules_dir = cwd.join("modules");

    match &cli.command {
        Commands::ValidateManifests => {
            let registry = load_manifests(&modules_dir)
                .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
            println!(
                "validated {} stages and {} tools",
                registry.stages().len(),
                registry
                    .stages()
                    .keys()
                    .map(|stage| registry.tools_for_stage(stage).len())
                    .sum::<usize>()
            );
            return Ok(());
        }
        Commands::Platform => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            println!("{}", serde_json::to_string_pretty(&platform)?);
            return Ok(());
        }
        Commands::Env { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                EnvCommand::Images => {
                    print_env_images(&catalog, &platform)?;
                }
                EnvCommand::Info => {
                    print_env_info(&catalog, &platform);
                }
                EnvCommand::Doctor => {
                    env_doctor(&catalog, &platform);
                }
            }
            return Ok(());
        }
        Commands::Bench { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                BenchCommand::Fastq { command } => match command {
                    BenchFastqCommand::Trim(args) => {
                        bench_fastq_trim(&catalog, &platform, None, args)?;
                    }
                    BenchFastqCommand::Validate(args) => {
                        bench_fastq_validate(&catalog, &platform, None, args)?;
                    }
                },
            }
            return Ok(());
        }
        Commands::Fastq { .. } => {}
    }

    let profile_path = cwd
        .join("configs")
        .join("profiles")
        .join(format!("{}.yaml", cli.profile));
    let mut profile = load_profile(&profile_path)
        .map_err(|err| anyhow!("failed to load profile {}: {err}", profile_path.display()))?;
    profile.run_base_dir = normalize_run_base_dir(&cwd, &profile.run_base_dir)?;

    let registry =
        load_manifests(&modules_dir).map_err(|err| anyhow!("manifest validation failed: {err}"))?;

    if let Commands::Fastq { command } = &cli.command {
        if let FastqCommand::Trim(args) = command {
            if is_bench_requested_trim(args) {
                let platform = load_platform(cli.platform.as_deref())
                    .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                let catalog =
                    load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
                let runner = parse_runner_override(args.env.as_deref())?;
                let bench_args = bench_args_from_trim(args)?;
                bench_fastq_trim(&catalog, &platform, runner, &bench_args)?;
                return Ok(());
            }
        }
        if let FastqCommand::Validate(args) = command {
            if is_bench_requested_validate(args) {
                let platform = load_platform(cli.platform.as_deref())
                    .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                let catalog =
                    load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
                let runner = parse_runner_override(args.env.as_deref())?;
                let bench_args = bench_args_from_validate(args)?;
                bench_fastq_validate(&catalog, &platform, runner, &bench_args)?;
                return Ok(());
            }
        }
    }

    let (stage, tool, common) = resolve_stage_tool(&cli.command);
    if common.list_tools {
        let mut tool_ids: Vec<_> = registry
            .tools_for_stage(&stage.0)
            .into_iter()
            .map(|tool| tool.tool_id.clone())
            .collect();
        tool_ids.sort();
        for tool_id in tool_ids {
            println!("{tool_id}");
        }
        return Ok(());
    }

    let run_id = new_run_id();
    let run_spec = RunSpec {
        stage: stage.clone(),
        tool: tool.clone(),
        paths: PathSpec {
            input: Vec::new(),
            output: Vec::new(),
            work: PathBuf::new(),
        },
        params: BTreeMap::new(),
    };

    let plan = build_execution_plan(run_spec, &registry, profile, run_id.clone())
        .map_err(|err| anyhow!("failed to build plan: {err}"))?;

    std::fs::create_dir_all(&plan.logs_dir).context("create logs directory")?;
    let log_path = plan.logs_dir.join("bijux.log");
    let _log_guard = init_logging(&log_path)?;

    println!("{}", serde_json::to_string_pretty(&plan)?);
    println!("manifests: {}", modules_dir.display());

    if !common.dry_run {
        warn!(
            run_id = %plan.run_id.0,
            stage = %plan.stage.stage_id,
            tool = %plan.tool.tool_id,
            "no executor implemented yet, falling back to dry-run"
        );
    }

    let executor = DryRunExecutor;
    executor.run(&plan)?;
    info!(
        run_id = %plan.run_id.0,
        stage = %plan.stage.stage_id,
        tool = %plan.tool.tool_id,
        "report written"
    );

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn bench_fastq_trim(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqTrimArgs,
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
    let seqkit_image = resolve_image(seqkit_spec, platform)?;

    let input_hash = hash_file_sha256(&r1)?;
    let input_stats = seqkit_stats(&seqkit_image, &r1_dir, &r1)?;

    let mut records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image(spec, platform)?;
        if !docker_image_exists(&image) {
            return Err(anyhow!("docker image not found: {}", image.full_name));
        }

        let tool_dir = tools_root.join(&tool);
        let out_dir = tool_dir.join("out");
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let out_fastq = out_dir.join(format!("trimmed_{}.fastq.gz", args.sample_id));
        let start = Instant::now();
        let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
        let execution = run_tool_container(&tool, &image, &r1_dir, &r1, &out_dir, &container_name)?;
        let runtime_s = start.elapsed().as_secs_f64();
        let memory_mb = docker_stats_mb(&container_name)?;
        docker_rm(&container_name)?;
        if execution.exit_code != 0 {
            return Err(anyhow!(
                "tool {tool} failed with status {} (stdout: {}, stderr: {})",
                execution.exit_code,
                execution.stdout.trim(),
                execution.stderr.trim()
            ));
        }
        if execution.output_fastq != out_fastq && execution.output_fastq.exists() {
            fs::rename(&execution.output_fastq, &out_fastq)
                .with_context(|| format!("rename {tool} output"))?;
        }

        let output_stats = seqkit_stats(&seqkit_image, &out_dir, &out_fastq)?;
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

fn bench_fastq_validate(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &BenchFastqValidateArgs,
) -> Result<()> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    let tools = normalize_validate_tool_list(&args.tools)?;
    let bench_dir = bench_base_dir(&args.out, "validate", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "validate", &args.sample_id);
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
    let seqkit_image = resolve_image(seqkit_spec, platform)?;

    let input_hash = hash_file_sha256(&r1)?;
    let input_stats = seqkit_stats(&seqkit_image, &r1_dir, &r1)?;

    let mut records: Vec<BenchmarkRecord<FastqValidateMetrics>> = Vec::new();

    for tool in tools {
        let spec = catalog
            .get(&tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image(spec, platform)?;
        if !docker_image_exists(&image) {
            return Err(anyhow!("docker image not found: {}", image.full_name));
        }

        let tool_dir = tools_root.join(&tool);
        let out_dir = tool_dir.join("out");
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let start = Instant::now();
        let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
        let execution =
            run_validate_container(&tool, &image, &r1_dir, &r1, &out_dir, &container_name)?;
        let runtime_s = start.elapsed().as_secs_f64();
        let memory_mb = docker_stats_mb(&container_name)?;
        docker_rm(&container_name)?;

        let metrics = FastqValidateMetrics {
            reads: input_stats.reads,
            bases: input_stats.bases,
            mean_q: input_stats.mean_q,
            format_valid: execution.exit_code == 0,
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
        if execution.exit_code != 0 {
            return Err(anyhow!(
                "tool {tool} failed with status {} (stdout: {}, stderr: {})",
                execution.exit_code,
                execution.stdout.trim(),
                execution.stderr.trim()
            ));
        }
        records.push(record);
    }

    let bench_path = bench_dir.join("bench.jsonl");
    for record in &records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = open_sqlite(&sqlite_path).context("open bench sqlite")?;
    for record in &records {
        insert_fastq_validate_v1(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_validate_comparability(&records);
    write_validate_report(&bench_dir, &records)?;
    Ok(())
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

fn hash_file_sha256(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let read = std::io::Read::read(&mut file, &mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    let digest = hasher.finalize();
    Ok(format!("sha256:{digest:x}"))
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
    let reads = first.metrics.reads;
    let bases = first.metrics.bases;
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
        if record.metrics.reads != reads {
            warn!(
                tool = record.context.tool,
                reads = record.metrics.reads,
                expected = reads,
                "reads differs across tools"
            );
        }
        if record.metrics.bases != bases {
            warn!(
                tool = record.context.tool,
                bases = record.metrics.bases,
                expected = bases,
                "bases differs across tools"
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
        if !record.metrics.format_valid {
            warn!(tool = record.context.tool, "format validation failed");
        }
    }
}

struct ExecutionOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
    output_fastq: PathBuf,
}

fn run_tool_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", r1_dir.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");
    let output_path = "/data/output/trimmed.fastq.gz";
    let mut out_fastq = out_dir.join("trimmed.fastq.gz");

    match tool {
        "fastp" => {
            cmd.arg("fastp")
                .arg("--in1")
                .arg(&input_path)
                .arg("--out1")
                .arg(output_path);
        }
        "cutadapt" => {
            cmd.arg("cutadapt")
                .arg("-o")
                .arg(output_path)
                .arg(&input_path);
        }
        "bbduk" => {
            cmd.arg("bbduk")
                .arg(format!("in={input_path}"))
                .arg(format!("out={output_path}"))
                .arg("overwrite=t");
        }
        "adapterremoval" => {
            cmd.arg("adapterremoval")
                .arg("--file1")
                .arg(&input_path)
                .arg("--output1")
                .arg(output_path);
        }
        "trimmomatic" => {
            cmd.arg("trimmomatic")
                .arg("SE")
                .arg("-phred33")
                .arg(&input_path)
                .arg(output_path)
                .arg("SLIDINGWINDOW:4:20")
                .arg("MINLEN:30");
        }
        "trim_galore" => {
            let basename = "trimmed";
            cmd.arg("trim_galore")
                .arg("--gzip")
                .arg("--output_dir")
                .arg("/data/output")
                .arg("--basename")
                .arg(basename)
                .arg(&input_path);
            out_fastq = out_dir.join(format!("{basename}_trimmed.fq.gz"));
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    }

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {tool}"));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {tool}"));
    }
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id, LogStream::Stdout)?;
    let stderr = docker_logs(&id, LogStream::Stderr)?;
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: out_fastq,
    })
}

fn run_validate_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let input_mount = format!("{}:/data/input:ro", r1_dir.display());
    let output_mount = format!("{}:/data/output", out_dir.display());

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("-d")
        .arg("--rm=false")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg(input_mount)
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name);

    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");

    match tool {
        "seqtk" => {
            cmd.arg("seqtk").arg("fqchk").arg(&input_path);
        }
        "fastqc" => {
            cmd.arg("fastqc")
                .arg("-q")
                .arg("-o")
                .arg("/data/output")
                .arg(&input_path);
        }
        "fastqvalidator" => {
            cmd.arg("fastq-validator").arg("--file").arg(&input_path);
        }
        "fqtools" => {
            cmd.arg("fqtools").arg("count").arg(&input_path);
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    }

    let output = cmd.output().context("run docker")?;
    if !output.status.success() {
        return Err(anyhow!("docker run failed for {tool}"));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err(anyhow!("missing container id for {tool}"));
    }
    let exit_code = docker_wait(&id)?;
    let stdout = docker_logs(&id, LogStream::Stdout)?;
    let stderr = docker_logs(&id, LogStream::Stderr)?;
    Ok(ExecutionOutput {
        exit_code,
        stdout,
        stderr,
        output_fastq: out_dir.join("unused"),
    })
}

fn docker_wait(container_id: &str) -> Result<i32> {
    let output = Command::new("docker")
        .arg("wait")
        .arg(container_id)
        .output()
        .context("docker wait")?;
    if !output.status.success() {
        return Err(anyhow!("docker wait failed"));
    }
    let code = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .context("parse exit code")?;
    Ok(code)
}

#[derive(Clone, Copy)]
enum LogStream {
    Stdout,
    Stderr,
}

fn docker_logs(container_id: &str, stream: LogStream) -> Result<String> {
    let mut cmd = Command::new("docker");
    cmd.arg("logs");
    match stream {
        LogStream::Stdout => {
            cmd.arg("--stdout");
        }
        LogStream::Stderr => {
            cmd.arg("--stderr");
        }
    }
    let output = cmd.arg(container_id).output().context("docker logs")?;
    if !output.status.success() {
        return Err(anyhow!("docker logs failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn docker_stats_mb(container_id: &str) -> Result<f64> {
    let output = Command::new("docker")
        .arg("stats")
        .arg("--no-stream")
        .arg("--format")
        .arg("{{.MemUsage}}")
        .arg(container_id)
        .output()
        .context("docker stats")?;
    if !output.status.success() {
        return Err(anyhow!("docker stats failed"));
    }
    let usage = String::from_utf8_lossy(&output.stdout)
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    parse_mem_to_mb(&usage)
}

fn parse_mem_to_mb(value: &str) -> Result<f64> {
    let value = value.replace(' ', "");
    if value.is_empty() {
        return Err(anyhow!("empty memory value"));
    }
    let (num, unit) = value
        .chars()
        .position(|c| !c.is_ascii_digit() && c != '.')
        .map(|idx| value.split_at(idx))
        .ok_or_else(|| anyhow!("invalid memory format"))?;
    let number: f64 = num.parse().context("parse memory number")?;
    let mb = match unit.to_lowercase().as_str() {
        "b" => number / 1_000_000.0,
        "kb" => number / 1_000.0,
        "kib" => number / 1024.0,
        "mb" => number,
        "mib" => number * 1.048_576,
        "gb" => number * 1000.0,
        "gib" => number * 1024.0,
        _ => return Err(anyhow!("unknown memory unit {unit}")),
    };
    Ok(mb)
}

fn docker_rm(container_id: &str) -> Result<()> {
    let output = Command::new("docker")
        .arg("rm")
        .arg("-f")
        .arg(container_id)
        .output()
        .context("docker rm")?;
    if !output.status.success() {
        return Err(anyhow!("docker rm failed"));
    }
    Ok(())
}

#[allow(clippy::struct_field_names)]
struct SeqkitMetrics {
    reads: u64,
    bases: u64,
    mean_q: f64,
}

fn seqkit_stats(image: &ResolvedImage, mount_dir: &Path, fastq: &Path) -> Result<SeqkitMetrics> {
    let output_mount = format!("{}:/data/input:ro", mount_dir.display());
    let out_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("missing input filename"))?
        .to_string_lossy()
        .to_string();
    let output_path = format!("/data/input/{out_name}");

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(output_mount)
        .arg(&image.full_name)
        .arg("seqkit")
        .arg("stats")
        .arg("-a")
        .arg("-T")
        .arg(&output_path)
        .output()
        .context("run seqkit stats")?;
    if !output.status.success() {
        return Err(anyhow!("seqkit stats failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_seqkit_stats(&stdout)
}

fn parse_seqkit_stats(output: &str) -> Result<SeqkitMetrics> {
    let mut lines = output.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("missing seqkit header"))?;
    let data = lines.next().ok_or_else(|| anyhow!("missing seqkit data"))?;

    let headers: Vec<&str> = header.split('\t').collect();
    let values: Vec<&str> = data.split('\t').collect();
    if headers.len() != values.len() {
        return Err(anyhow!("seqkit header/data mismatch"));
    }

    let mut reads = None;
    let mut bases = None;
    let mut mean_q = None;

    for (idx, name) in headers.iter().enumerate() {
        let key = name.to_lowercase();
        match key.as_str() {
            "num_seqs" | "num_seq" => {
                reads = Some(values[idx].parse::<u64>().context("reads")?);
            }
            "sum_len" | "sum_len(bp)" => {
                bases = Some(values[idx].parse::<u64>().context("bases")?);
            }
            "avg_qual" | "avg_qual(phred)" | "avgqual" => {
                mean_q = Some(values[idx].parse::<f64>().context("mean_q")?);
            }
            _ => {}
        }
    }

    Ok(SeqkitMetrics {
        reads: reads.ok_or_else(|| anyhow!("missing num_seqs"))?,
        bases: bases.ok_or_else(|| anyhow!("missing sum_len"))?,
        mean_q: mean_q.ok_or_else(|| anyhow!("missing avg_qual"))?,
    })
}

fn write_report(base_dir: &Path, records: &[BenchmarkRecord<FastqTrimMetrics>]) -> Result<()> {
    let mut sorted = records.to_vec();
    sorted.sort_by(|a, b| {
        a.execution
            .runtime_s
            .partial_cmp(&b.execution.runtime_s)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut report = String::new();
    report.push_str(
        "tool\truntime_s\tmemory_mb\treads_in\treads_out\tbases_in\tbases_out\tmean_q_before\tmean_q_after\n",
    );
    for record in sorted {
        use std::fmt::Write as _;
        let _ = writeln!(
            report,
            "{}\t{:.3}\t{:.2}\t{}\t{}\t{}\t{}\t{:.2}\t{:.2}",
            record.context.tool,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.metrics.reads_in,
            record.metrics.reads_out,
            record.metrics.bases_in,
            record.metrics.bases_out,
            record.metrics.mean_q_before,
            record.metrics.mean_q_after
        );
    }
    fs::write(base_dir.join("report.md"), report).context("write report")?;
    Ok(())
}

fn write_validate_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Result<()> {
    let mut sorted = records.to_vec();
    sorted.sort_by(|a, b| {
        a.execution
            .runtime_s
            .partial_cmp(&b.execution.runtime_s)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut report = String::new();
    report.push_str("tool\truntime_s\tmemory_mb\treads\tbases\tmean_q\tformat_valid\n");
    for record in sorted {
        use std::fmt::Write as _;
        let _ = writeln!(
            report,
            "{}\t{:.3}\t{:.2}\t{}\t{}\t{:.2}\t{}",
            record.context.tool,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.metrics.reads,
            record.metrics.bases,
            record.metrics.mean_q,
            record.metrics.format_valid
        );
    }
    fs::write(base_dir.join("report.md"), report).context("write report")?;
    Ok(())
}

fn print_env_images(
    catalog: &HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
) -> Result<()> {
    let mut rows: Vec<(String, ResolvedImage)> = catalog
        .values()
        .map(|spec| {
            let resolved = resolve_image(spec, platform)
                .map_err(|err| anyhow!("image resolve failed for {}: {err}", spec.tool))?;
            Ok((spec.tool.clone(), resolved))
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let width = rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    for (name, resolved) in rows {
        println!(
            "{name:width$} -> {} ({})",
            resolved.full_name,
            resolved.runner,
            width = width
        );
    }
    Ok(())
}

fn print_env_info(catalog: &HashMap<String, ToolImageSpec>, platform: &PlatformSpec) {
    let mut detected = available_runners().unwrap_or_default();
    detected.sort_by_key(std::string::ToString::to_string);
    let chosen = select_best_runner(platform.runner, &detected).unwrap_or(platform.runner);
    println!("platform: {}", platform.name);
    println!("runner: {}", platform.runner);
    println!("detected_runners: {}", display_runners(&detected));
    println!("chosen_runner: {chosen}");
    println!("image_count: {}", catalog.len());
}

fn env_doctor(catalog: &HashMap<String, ToolImageSpec>, platform: &PlatformSpec) {
    let mut detected = available_runners().unwrap_or_default();
    detected.sort_by_key(std::string::ToString::to_string);
    let chosen = select_best_runner(platform.runner, &detected).unwrap_or(platform.runner);

    print_check("runners_available", !detected.is_empty());
    print_check("platform_valid", true);

    let images_resolvable = catalog
        .values()
        .all(|spec| resolve_image(spec, platform).is_ok());
    print_check("images_resolvable", images_resolvable);

    let cache_ok = ensure_cache_writable(platform.runner);
    print_check("cache_writable", cache_ok);

    let missing_images = catalog
        .values()
        .filter_map(|spec| resolve_image(spec, platform).ok())
        .any(|image| match chosen {
            RunnerKind::Docker => !docker_image_exists(&image),
            _ => false,
        });
    print_check("docker_images_present", !missing_images);
}

fn ensure_cache_writable(runner: RunnerKind) -> bool {
    let dir = cache_dir(runner);
    if fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let probe = dir.join(".bijux_write_test");
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn print_check(name: &str, ok: bool) {
    if ok {
        println!("{name}: PASS");
    } else {
        println!("{name}: FAIL");
    }
}

fn display_runners(runners: &[RunnerKind]) -> String {
    if runners.is_empty() {
        "none".to_string()
    } else {
        runners
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn resolve_stage_tool(command: &Commands) -> (StageId, ToolId, CommonArgs) {
    match command {
        Commands::Fastq { command } => {
            let (tool, common) = match command {
                FastqCommand::Filter(common) => ("filter", common.clone()),
                FastqCommand::Merge(common) => ("merge", common.clone()),
                FastqCommand::Trim(args) => ("fastp", args.common.clone()),
                FastqCommand::Contam(common) => ("contam", common.clone()),
                FastqCommand::Preprocess(common) => ("preprocess", common.clone()),
                FastqCommand::Umi(common) => ("umi", common.clone()),
                FastqCommand::ErrorCorrect(common) => ("error-correct", common.clone()),
                FastqCommand::Qc(common) => ("qc", common.clone()),
                FastqCommand::Validate(args) => ("seqtk", args.common.clone()),
                FastqCommand::Align(common) => ("align", common.clone()),
            };
            (
                StageId(
                    match command {
                        FastqCommand::Trim(_) => "fastq.trim",
                        FastqCommand::Validate(_) => "fastq.validate",
                        _ => "fastq",
                    }
                    .to_string(),
                ),
                ToolId(tool.to_string()),
                common,
            )
        }
        Commands::ValidateManifests
        | Commands::Platform
        | Commands::Env { .. }
        | Commands::Bench { .. } => (
            StageId("unknown".to_string()),
            ToolId("unknown".to_string()),
            CommonArgs::default(),
        ),
    }
}

fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> Result<PathBuf> {
    let run_base_dir = if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    };
    let file_name = run_base_dir.file_name().and_then(|s| s.to_str());
    if file_name == Some("runs") {
        run_base_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("run base dir has no parent"))
    } else {
        Ok(run_base_dir)
    }
}

fn is_bench_requested_trim(args: &FastqTrimArgs) -> bool {
    args.sample_id.is_some() || args.r1.is_some() || args.out.is_some() || !args.tools.is_empty()
}

fn bench_args_from_trim(args: &FastqTrimArgs) -> Result<BenchFastqTrimArgs> {
    let sample_id = args
        .sample_id
        .clone()
        .ok_or_else(|| anyhow!("--sample-id is required for benchmarking"))?;
    let r1 = args
        .r1
        .clone()
        .ok_or_else(|| anyhow!("--r1 is required for benchmarking"))?;
    let out = args
        .out
        .clone()
        .ok_or_else(|| anyhow!("--out is required for benchmarking"))?;
    if args.tools.is_empty() {
        return Err(anyhow!("--tools is required for benchmarking"));
    }
    Ok(BenchFastqTrimArgs {
        sample_id,
        r1,
        out,
        tools: args.tools.clone(),
    })
}

fn is_bench_requested_validate(args: &FastqValidateArgs) -> bool {
    args.sample_id.is_some() || args.r1.is_some() || args.out.is_some() || !args.tools.is_empty()
}

fn bench_args_from_validate(args: &FastqValidateArgs) -> Result<BenchFastqValidateArgs> {
    let sample_id = args
        .sample_id
        .clone()
        .ok_or_else(|| anyhow!("--sample-id is required for benchmarking"))?;
    let r1 = args
        .r1
        .clone()
        .ok_or_else(|| anyhow!("--r1 is required for benchmarking"))?;
    let out = args
        .out
        .clone()
        .ok_or_else(|| anyhow!("--out is required for benchmarking"))?;
    if args.tools.is_empty() {
        return Err(anyhow!("--tools is required for benchmarking"));
    }
    Ok(BenchFastqValidateArgs {
        sample_id,
        r1,
        out,
        tools: args.tools.clone(),
    })
}

fn parse_runner_override(env: Option<&str>) -> Result<Option<RunnerKind>> {
    let Some(value) = env else {
        return Ok(None);
    };
    let runner = RunnerKind::from_str(value)
        .map_err(|_| anyhow!("invalid runner: {value} (expected docker/apptainer/singularity)"))?;
    Ok(Some(runner))
}

fn bench_base_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    match out.file_name().and_then(|name| name.to_str()) {
        Some("artifacts") => out.join(stage).join("benchmark").join(sample_id),
        _ => out
            .join("artifacts")
            .join(stage)
            .join("benchmark")
            .join(sample_id),
    }
}

fn bench_tools_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    match out.file_name().and_then(|name| name.to_str()) {
        Some("artifacts") => out.join(stage).join("tools").join(sample_id),
        _ => out
            .join("artifacts")
            .join(stage)
            .join("tools")
            .join(sample_id),
    }
}

fn init_logging(log_path: &Path) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .context("open log file")?;
    let (file_writer, guard) = tracing_appender::non_blocking(file);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer().with_target(false);
    let file_layer = fmt::layer().with_writer(file_writer).with_target(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
