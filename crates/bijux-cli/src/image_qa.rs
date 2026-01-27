use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_image_qa_jsonl, image_qa_input_hashes_from_records, image_qa_inputs, image_qa_passed,
    insert_image_qa_input_v1, insert_image_qa_v1, open_sqlite, ImageQaOutcome, ImageQaRecord,
};
use bijux_environment::api::{
    docker_image_exists, load_image_catalog, load_platform, resolve_image,
};
use bijux_environment::api::{PlatformSpec, ResolvedImage, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use crate::utils::{
    docker_rm, hash_file_sha256, image_qa_jsonl_path, image_qa_sqlite_path, input_fastq_stats,
    output_fastq_stats, run_merge_container_with_timeout, run_tool_container_with_timeout,
    run_validate_container_with_timeout, SeqkitMetrics,
};

const TOOLS_TRIM: [&str; 6] = [
    "fastp",
    "cutadapt",
    "bbduk",
    "adapterremoval",
    "trimmomatic",
    "trim_galore",
];
const TOOLS_VALIDATE: [&str; 4] = ["seqtk", "fastqc", "fastqvalidator", "fqtools"];
const TOOLS_FILTER: [&str; 3] = ["prinseq", "fastp", "seqkit"];
const TOOLS_MERGE: [&str; 4] = ["pear", "vsearch", "bbmerge", "flash2"];
const QA_TIMEOUT_SECS: u64 = 300;
const QA_MERGE_TIMEOUT_SECS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QaStage {
    Trim,
    Validate,
    Filter,
    Merge,
}

impl QaStage {
    fn stage_id(self) -> &'static str {
        match self {
            QaStage::Trim => "fastq.trim",
            QaStage::Validate => "fastq.validate",
            QaStage::Filter => "fastq.filter",
            QaStage::Merge => "fastq.merge",
        }
    }

    fn tools(self) -> &'static [&'static str] {
        match self {
            QaStage::Trim => &TOOLS_TRIM,
            QaStage::Validate => &TOOLS_VALIDATE,
            QaStage::Filter => &TOOLS_FILTER,
            QaStage::Merge => &TOOLS_MERGE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Debug,
}

struct StdoutLogger {
    level: LogLevel,
    quiet: bool,
}

impl StdoutLogger {
    fn new(level: LogLevel, quiet: bool) -> Self {
        Self { level, quiet }
    }

    fn info(&self, line: &str) {
        if !self.quiet {
            println!("{line}");
        }
    }

    fn debug(&self, line: &str) {
        if !self.quiet && matches!(self.level, LogLevel::Debug) {
            println!("{line}");
        }
    }
}

pub fn run_image_qa(platform_name: Option<&str>) -> Result<()> {
    let debug = std::env::var("DEBUG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let quiet = std::env::var("QUIET")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let logger = StdoutLogger::new(
        if debug {
            LogLevel::Debug
        } else {
            LogLevel::Info
        },
        quiet,
    );
    let platform =
        load_platform(platform_name).map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog = load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
    run_image_qa_with(&platform, &catalog, &logger)
}

fn run_image_qa_with(
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    logger: &StdoutLogger,
) -> Result<()> {
    if platform.runner != RunnerKind::Docker {
        return Err(anyhow!("image QA supports docker only for now"));
    }
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let qa_dir = image_qa_sqlite_path(&cwd, &platform.name)
        .parent()
        .ok_or_else(|| anyhow!("qa sqlite path missing parent"))?
        .to_path_buf();
    fs::create_dir_all(&qa_dir).context("create image-qa output dir")?;

    let qa_jsonl = image_qa_jsonl_path(&cwd, &platform.name);
    let qa_sqlite = image_qa_sqlite_path(&cwd, &platform.name);
    let conn = open_sqlite(&qa_sqlite).context("open qa sqlite")?;

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;
    let mut datasets = discover_qa_datasets()?;
    hydrate_datasets(&mut datasets, &seqkit_image)?;

    log_header(logger, &platform.name, platform.runner, &datasets);

    let mut pass = 0;
    let mut fail = 0;

    for stage in [
        QaStage::Trim,
        QaStage::Validate,
        QaStage::Filter,
        QaStage::Merge,
    ] {
        log_stage_header(logger, stage);
        for dataset in datasets_for_stage(stage, &datasets) {
            let input_hash = dataset_input_hash(stage, &dataset);
            log_dataset(logger, &dataset);
            insert_image_qa_input_v1(
                &conn,
                stage.stage_id(),
                &input_hash,
                &platform.name,
                &platform.runner.to_string(),
            )
            .context("write qa inputs sqlite")?;
            for &tool in stage.tools() {
                if stage != QaStage::Trim
                    && qa_already_passed(&conn, stage, tool, platform, catalog, &input_hash)
                        .unwrap_or(false)
                {
                    log_tool_skip(logger, stage, tool, &dataset);
                    continue;
                }
                log_tool(logger, stage, tool);
                let mut outcome =
                    run_stage_qa(stage, tool, platform, catalog, &dataset, &seqkit_image);
                let record = match build_qa_record(
                    stage,
                    tool,
                    platform,
                    catalog,
                    &input_hash,
                    outcome.clone(),
                ) {
                    Ok(record) => record,
                    Err(err) => {
                        outcome = ImageQaOutcome::Fail(err.to_string());
                        ImageQaRecord {
                            tool: tool.to_string(),
                            stage: stage.stage_id().to_string(),
                            tool_version: "unknown".to_string(),
                            image_digest: "unknown".to_string(),
                            runner: platform.runner.to_string(),
                            platform: platform.name.clone(),
                            input_hash: input_hash.clone(),
                            outcome: outcome.clone(),
                        }
                    }
                };
                log_tool_result(logger, stage, tool, &dataset, &outcome);
                if matches!(outcome, ImageQaOutcome::Pass) {
                    pass += 1;
                } else {
                    fail += 1;
                }
                append_image_qa_jsonl(&qa_jsonl, &record).context("write qa jsonl")?;
                insert_image_qa_v1(&conn, &record).context("write qa sqlite")?;
            }
        }
    }

    println!("QA PASS: {pass}");
    println!("QA FAIL: {fail}");
    if fail > 0 {
        return Err(anyhow!("image QA failed for {fail} tools"));
    }
    Ok(())
}

fn run_stage_qa(
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> ImageQaOutcome {
    let result = match stage {
        QaStage::Trim => qa_trim_tool(
            tool,
            platform,
            catalog,
            &dataset.r1,
            &dataset.r1_dir,
            seqkit_image,
            &dataset.input_stats_r1,
        ),
        QaStage::Validate => {
            qa_validate_tool(tool, platform, catalog, &dataset.r1, &dataset.r1_dir)
        }
        QaStage::Filter => qa_filter_tool(
            tool,
            platform,
            catalog,
            &dataset.r1,
            &dataset.r1_dir,
            seqkit_image,
            &dataset.input_stats_r1,
        ),
        QaStage::Merge => {
            let Some(r2) = dataset.r2.as_ref() else {
                return ImageQaOutcome::Fail("merge dataset missing r2".to_string());
            };
            let Some(stats_r2) = dataset.input_stats_r2.as_ref() else {
                return ImageQaOutcome::Fail("merge dataset missing r2 stats".to_string());
            };
            qa_merge_tool(
                tool,
                platform,
                catalog,
                &dataset.r1,
                r2,
                &dataset.r1_dir,
                seqkit_image,
                &dataset.input_stats_r1,
                stats_r2,
            )
        }
    };
    match result {
        Ok(()) => ImageQaOutcome::Pass,
        Err(err) => ImageQaOutcome::Fail(err.to_string()),
    }
}

fn qa_trim_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    r1: &Path,
    r1_dir: &Path,
    seqkit_image: &ResolvedImage,
    input_stats: &SeqkitMetrics,
) -> Result<()> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("trim", tool)?;
    let container_name = format!("bijux-qa-trim-{}-{}", tool, Uuid::new_v4());
    let start = Instant::now();
    let timeout = std::time::Duration::from_secs(QA_MERGE_TIMEOUT_SECS);
    let execution = match run_tool_container_with_timeout(
        tool,
        &image,
        r1_dir,
        r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    let _runtime = start.elapsed();
    docker_rm(&container_name)?;

    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    let out_fastq = execution
        .output_fastq
        .ok_or_else(|| anyhow!("output FASTQ missing"))?;
    let out_fastq = if out_fastq.exists() {
        out_fastq
    } else {
        let alt = out_fastq.with_extension("");
        if alt.exists() {
            alt
        } else {
            return Err(anyhow!("output FASTQ not found: {}", out_fastq.display()));
        }
    };
    let output_stats = output_fastq_stats(seqkit_image, &out_dir, &out_fastq)?;
    if output_stats.reads > input_stats.reads {
        return Err(anyhow!(
            "reads_out {} exceeds reads_in {}",
            output_stats.reads,
            input_stats.reads
        ));
    }
    Ok(())
}

fn qa_validate_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    r1: &Path,
    r1_dir: &Path,
) -> Result<()> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("validate", tool)?;
    let container_name = format!("bijux-qa-validate-{}-{}", tool, Uuid::new_v4());
    let timeout = std::time::Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_validate_container_with_timeout(
        tool,
        &image,
        r1_dir,
        r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    match tool {
        "seqtk" => {
            if execution.stdout.trim().is_empty() {
                return Err(anyhow!("seqtk produced empty output"));
            }
        }
        "fastqc" => {
            // FastQC sometimes writes output asynchronously or with differing suffixes.
            // Exit code is treated as sufficient for QA here.
        }
        "fastqvalidator" => {
            crate::utils::parse_fastqvalidator_count(&execution.stdout)?;
        }
        "fqtools" => {
            let count = execution
                .stdout
                .split_whitespace()
                .find_map(|part| part.parse::<u64>().ok());
            if count.is_none() {
                return Err(anyhow!("fqtools output missing count"));
            }
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    }
    Ok(())
}

fn qa_filter_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    r1: &Path,
    r1_dir: &Path,
    seqkit_image: &ResolvedImage,
    input_stats: &SeqkitMetrics,
) -> Result<()> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("filter", tool)?;
    let container_name = format!("bijux-qa-filter-{}-{}", tool, Uuid::new_v4());
    let timeout = std::time::Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_tool_container_with_timeout(
        tool,
        &image,
        r1_dir,
        r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    let out_fastq = execution
        .output_fastq
        .ok_or_else(|| anyhow!("output FASTQ missing"))?;
    let out_fastq = if out_fastq.exists() {
        out_fastq
    } else {
        let alt = out_fastq.with_extension("");
        if alt.exists() {
            alt
        } else {
            return Err(anyhow!("output FASTQ not found: {}", out_fastq.display()));
        }
    };
    let output_stats = output_fastq_stats(seqkit_image, &out_dir, &out_fastq)?;
    if output_stats.reads > input_stats.reads {
        return Err(anyhow!(
            "reads_out {} exceeds reads_in {}",
            output_stats.reads,
            input_stats.reads
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn qa_merge_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    r1: &Path,
    r2: &Path,
    r1_dir: &Path,
    seqkit_image: &ResolvedImage,
    input_stats_r1: &SeqkitMetrics,
    input_stats_r2: &SeqkitMetrics,
) -> Result<()> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("merge", tool)?;
    let container_name = format!("bijux-qa-merge-{}-{}", tool, Uuid::new_v4());
    let timeout = std::time::Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_merge_container_with_timeout(
        tool,
        &image,
        r1_dir,
        r1,
        r2,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }

    let merged_exists = execution.merged_fastq.exists();
    let unmerged_exists = execution.unmerged_r1.exists() && execution.unmerged_r2.exists();
    if !merged_exists && !unmerged_exists {
        return Err(anyhow!("merge outputs missing"));
    }

    let merged_reads = if merged_exists {
        let stats = output_fastq_stats(seqkit_image, &out_dir, &execution.merged_fastq)?;
        stats.reads
    } else {
        0
    };
    let min_reads = input_stats_r1.reads.min(input_stats_r2.reads);
    if merged_reads > min_reads {
        return Err(anyhow!(
            "merged_reads {merged_reads} exceeds min(reads_r1, reads_r2) {min_reads}"
        ));
    }
    Ok(())
}

fn temp_out_dir(stage: &str, tool: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir().join("bijux-image-qa").join(stage);
    fs::create_dir_all(&base).context("create qa temp dir")?;
    let path = base.join(format!("{tool}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).context("create qa tool dir")?;
    Ok(path)
}

fn build_qa_record(
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    input_hash: &str,
    outcome: ImageQaOutcome,
) -> Result<ImageQaRecord> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image_digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
        .to_string();
    Ok(ImageQaRecord {
        tool: tool.to_string(),
        stage: stage.stage_id().to_string(),
        tool_version: spec.version.clone(),
        image_digest,
        runner: platform.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: input_hash.to_string(),
        outcome,
    })
}

fn qa_already_passed(
    conn: &rusqlite::Connection,
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    input_hash: &str,
) -> Result<bool> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let digest = spec
        .digest
        .as_ref()
        .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?;
    let runner = platform.runner.to_string();
    image_qa_passed(
        conn,
        tool,
        stage.stage_id(),
        digest,
        &platform.name,
        &runner,
        input_hash,
    )
    .map_err(|err| anyhow!(err))
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
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
}

#[allow(dead_code)]
pub fn ensure_image_qa_passed(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let qa_sqlite = image_qa_sqlite_path(&cwd, &platform.name);
    if !qa_sqlite.exists() {
        return Err(anyhow!(
            "image QA results missing; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    let conn = open_sqlite(&qa_sqlite).context("open qa sqlite")?;
    let runner = platform.runner.to_string();
    let mut expected_inputs = image_qa_inputs(&conn, stage, &platform.name, &runner)?;
    if expected_inputs.is_empty() {
        expected_inputs = image_qa_input_hashes_from_records(&conn, stage, &platform.name, &runner)
            .map_err(|err| anyhow!(err))?;
    }
    if expected_inputs.is_empty() {
        return Err(anyhow!(
            "image QA inputs missing for {stage}; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    for tool in tools {
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let digest = spec
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?;
        for input_hash in &expected_inputs {
            let passed = image_qa_passed(
                &conn,
                tool,
                stage,
                digest,
                &platform.name,
                &runner,
                input_hash,
            )?;
            if !passed {
                return Err(anyhow!(
                    "image QA failed or missing for {tool} ({stage}); run `bijux image-qa --platform {}`",
                    platform.name
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct QaDataset {
    name: String,
    r1: PathBuf,
    r2: Option<PathBuf>,
    r1_dir: PathBuf,
    input_hash_r1: String,
    input_hash_r2: Option<String>,
    input_stats_r1: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
}

fn discover_qa_datasets() -> Result<Vec<QaDataset>> {
    let base = Path::new("tests/data/fastq");
    if !base.exists() {
        return Err(anyhow!("missing tests/data/fastq"));
    }
    let mut datasets = Vec::new();
    for entry in fs::read_dir(base).context("read tests/data/fastq")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let mut r1 = None;
            let mut r2 = None;
            for file in fs::read_dir(&path)? {
                let file = file?;
                let fpath = file.path();
                if !is_fastq_gz(&fpath) {
                    continue;
                }
                let name = fpath.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.ends_with("_1.fastq.gz") {
                    r1 = Some(fpath);
                } else if name.ends_with("_2.fastq.gz") {
                    r2 = Some(fpath);
                } else if name.ends_with(".fastq.gz") {
                    r1 = Some(fpath);
                }
            }
            if let Some(r1_path) = r1 {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("invalid dataset name"))?
                    .to_string();
                datasets.push(QaDataset {
                    name,
                    r1: r1_path,
                    r2,
                    r1_dir: path.clone(),
                    input_hash_r1: String::new(),
                    input_hash_r2: None,
                    input_stats_r1: SeqkitMetrics {
                        reads: 0,
                        bases: 0,
                        mean_q: 0.0,
                    },
                    input_stats_r2: None,
                });
            }
        } else if is_fastq_gz(&path) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("invalid dataset name"))?
                .to_string();
            let r1_dir = path.parent().ok_or_else(|| anyhow!("missing parent"))?;
            datasets.push(QaDataset {
                name,
                r1: path.clone(),
                r2: None,
                r1_dir: r1_dir.to_path_buf(),
                input_hash_r1: String::new(),
                input_hash_r2: None,
                input_stats_r1: SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                },
                input_stats_r2: None,
            });
        }
    }

    if datasets.is_empty() {
        return Err(anyhow!("no FASTQ files found in tests/data/fastq"));
    }
    Ok(datasets)
}

fn hydrate_datasets(datasets: &mut [QaDataset], seqkit_image: &ResolvedImage) -> Result<()> {
    for dataset in datasets {
        let r1 = dataset.r1.canonicalize().context("resolve r1 path")?;
        let r1_dir = r1
            .parent()
            .ok_or_else(|| anyhow!("r1 has no parent"))?
            .to_path_buf();
        dataset.r1 = r1;
        dataset.r1_dir.clone_from(&r1_dir);
        dataset.input_hash_r1 = hash_file_sha256(&dataset.r1)?;
        dataset.input_stats_r1 = input_fastq_stats(seqkit_image, &r1_dir, &dataset.r1)?;

        if let Some(r2) = dataset.r2.clone() {
            let r2 = r2.canonicalize().context("resolve r2 path")?;
            dataset.r2 = Some(r2.clone());
            let stats = input_fastq_stats(seqkit_image, &r1_dir, &r2)?;
            dataset.input_stats_r2 = Some(stats);
            let r2_hash = hash_file_sha256(&r2)?;
            dataset.input_hash_r2 = Some(r2_hash);
        }
    }
    Ok(())
}

fn datasets_for_stage(stage: QaStage, datasets: &[QaDataset]) -> Vec<QaDataset> {
    match stage {
        QaStage::Merge => datasets
            .iter()
            .filter(|dataset| dataset.r2.is_some())
            .cloned()
            .collect(),
        _ => datasets.to_vec(),
    }
}

fn dataset_input_hash(stage: QaStage, dataset: &QaDataset) -> String {
    match stage {
        QaStage::Merge => {
            let r1 = dataset.input_hash_r1.as_str();
            let r2 = dataset.input_hash_r2.as_deref().unwrap_or("missing");
            format!("{r1},{r2}")
        }
        _ => dataset.input_hash_r1.clone(),
    }
}

fn is_fastq_gz(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
}

fn log_header(logger: &StdoutLogger, platform: &str, runner: RunnerKind, datasets: &[QaDataset]) {
    logger.info(&format!("[bijux] Image QA started ({runner})"));
    logger.info(&format!(
        "[bijux] Platform: {platform} | Datasets: {}",
        datasets.len()
    ));
    if !datasets.is_empty() {
        logger.info("[bijux] QA datasets:");
        for dataset in datasets {
            logger.info(&format!("  - {}", dataset.name));
        }
    }
}

fn log_stage_header(logger: &StdoutLogger, stage: QaStage) {
    logger.info(&format!("[bijux][image-qa] Stage: {}", stage.stage_id()));
}

fn log_dataset(logger: &StdoutLogger, dataset: &QaDataset) {
    logger.debug(&format!("[bijux][image-qa][dataset] {}", dataset.name));
}

fn log_tool(logger: &StdoutLogger, stage: QaStage, tool: &str) {
    logger.debug(&format!(
        "[bijux][image-qa][run] {}::{tool}",
        stage.stage_id()
    ));
}

fn log_tool_result(
    logger: &StdoutLogger,
    stage: QaStage,
    tool: &str,
    dataset: &QaDataset,
    outcome: &ImageQaOutcome,
) {
    let status = match outcome {
        ImageQaOutcome::Pass => "pass",
        ImageQaOutcome::Fail(_) => "fail",
    };
    let mut line = format!(
        "[bijux][image-qa][{status}] {}::{tool} ({})",
        stage.stage_id(),
        dataset.name
    );
    if let ImageQaOutcome::Fail(reason) = outcome {
        let _ = write!(line, " - {reason}");
    }
    logger.info(&line);
}

fn log_tool_skip(logger: &StdoutLogger, stage: QaStage, tool: &str, dataset: &QaDataset) {
    logger.info(&format!(
        "[bijux][image-qa][skip] {}::{tool} ({})",
        stage.stage_id(),
        dataset.name
    ));
}
