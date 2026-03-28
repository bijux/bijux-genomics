use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest as _;

use crate::commands::benchmark_workspace::{
    benchmark_workspace_value, corpus_01_publication_contract, load_benchmark_workspace_config,
};
use crate::commands::cli::{BenchCorpusFastqArgs, BenchWorkspaceValueArgs, Cli};

#[derive(Debug, Clone, Deserialize)]
struct CorpusSpec {
    corpus_id: String,
    #[serde(default)]
    target_ancient_se: usize,
    #[serde(default)]
    target_ancient_pe: usize,
    #[serde(default)]
    target_modern_se: usize,
    #[serde(default)]
    target_modern_pe: usize,
    #[serde(default)]
    samples: Vec<CorpusSpecSample>,
}

#[derive(Debug, Clone, Deserialize)]
struct CorpusSpecSample {
    accession: String,
    era: String,
    layout: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CorpusManifest {
    #[serde(default)]
    files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct CorpusNormalizedSample {
    sample_id: String,
    r1: String,
    r2: Option<String>,
    layout: String,
}

#[derive(Debug, Clone)]
struct PendingSampleRun {
    sample: CorpusNormalizedSample,
    report_json: PathBuf,
    command_args: Vec<String>,
    command: Vec<String>,
    env_overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct SampleRunRecord {
    sample_id: String,
    r1: String,
    r2: Option<String>,
    layout: String,
    status: String,
    exit_code: i32,
    command: Vec<String>,
    report_json: String,
}

#[derive(Debug, Serialize)]
struct CorpusRunManifest {
    schema_version: String,
    generated_at_utc: String,
    corpus_id: String,
    stage_id: String,
    scenario_id: String,
    sample_scope: String,
    tool_kind: String,
    platform: String,
    tools: Vec<String>,
    threads: u32,
    jobs: u32,
    sample_jobs: usize,
    sample_limit: Option<usize>,
    dry_run: bool,
    config_path: String,
    publication_config_path: String,
    repo_root: String,
    corpus_root: String,
    out_root: String,
    stage_args: Vec<String>,
    samples_total: usize,
    samples_failed: usize,
    runs: Vec<SampleRunRecord>,
    #[serde(flatten)]
    extra_fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
struct StageCommandSpec {
    bench_subcommand: &'static str,
    report_dir: &'static str,
    strict_resume_report: bool,
}

pub(crate) fn print_benchmark_workspace_value(
    cwd: &Path,
    args: &BenchWorkspaceValueArgs,
) -> Result<()> {
    println!(
        "{}",
        benchmark_workspace_value(cwd, args.config.as_deref(), &args.key)?
    );
    Ok(())
}

pub(crate) fn run_benchmark_corpus_fastq(cli: &Cli, args: &BenchCorpusFastqArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current working directory")?;
    let workspace_config = load_benchmark_workspace_config(&repo_root, args.config.as_deref())?;
    let workspace_config_path =
        crate::commands::benchmark_workspace::benchmark_workspace_config_path(
            &repo_root,
            args.config.as_deref(),
        );
    let publication_config_path =
        crate::commands::benchmark_workspace::benchmark_publication_config_path(
            &repo_root,
            args.publication_config.as_deref(),
        );
    let contract = corpus_01_publication_contract(
        &repo_root,
        args.publication_config.as_deref(),
        &args.stage,
    )?;
    let stage_spec = stage_command_spec(&args.stage)?;
    let corpus_spec = load_corpus_spec(&repo_root)?;
    if corpus_spec.corpus_id != "corpus-01" {
        return Err(anyhow!(
            "unsupported corpus benchmark spec `{}`; expected corpus-01",
            corpus_spec.corpus_id
        ));
    }

    let corpus_root = if let Some(path) = args.corpus_root.as_ref() {
        absolutize(&repo_root, path)
    } else {
        workspace_config
            .remote
            .as_ref()
            .and_then(|row| row.corpus_root.as_deref())
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("workspace config is missing remote.corpus_root"))?
    };
    let out_root = if let Some(path) = args.out_root.as_ref() {
        absolutize(&repo_root, path)
    } else {
        default_stage_out_root(&workspace_config, &corpus_spec.corpus_id, &args.stage)?
    };

    validate_benchmark_layout(&corpus_root, &out_root)?;
    fs::create_dir_all(&out_root).with_context(|| format!("create {}", out_root.display()))?;

    let all_samples =
        discover_normalized_samples(&corpus_root, corpus_expected_sample_total(&corpus_spec))?;
    let metadata_by_sample = validate_corpus_contract(&corpus_root, &corpus_spec, &all_samples)?;
    let mut selected_samples = match contract.sample_scope.as_str() {
        "full" => all_samples,
        "paired" => select_paired_samples(&corpus_spec, &all_samples, &metadata_by_sample)?,
        other => {
            return Err(anyhow!(
                "unsupported corpus benchmark sample scope `{other}` for {}",
                contract.stage_id
            ))
        }
    };
    if args.sample_limit > 0 && args.sample_limit < selected_samples.len() {
        selected_samples.truncate(args.sample_limit);
    }

    let tools = resolve_tools(&contract.tools, &args.tools)?;
    let platform = cli
        .platform
        .clone()
        .or_else(|| std::env::var("BIJUX_PLATFORM").ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "apptainer-amd64".to_string());
    let program = std::env::current_exe().context("resolve bijux-dna executable")?;
    let mut runs = Vec::new();
    let mut pending = Vec::new();
    let runtime_env = benchmark_runtime_env(&out_root);
    let extra_manifest_fields =
        collect_stage_manifest_fields(&args.stage, &args.stage_args, &args.manifest_args)?;

    for sample in selected_samples {
        let sample_root = benchmark_sample_root(&out_root, stage_spec.report_dir, &sample.sample_id);
        let report_json = out_root
            .join("bench")
            .join(stage_spec.report_dir)
            .join(&sample.sample_id)
            .join("report.json");
        if args.resume {
            if stage_spec.strict_resume_report {
                if sample_root.is_dir() && !report_json.is_file() {
                    reset_sample_payload(&sample_root)?;
                }
                if report_json.is_file() {
                    if sample_report_is_resume_ready(&report_json) {
                        runs.push(SampleRunRecord {
                            sample_id: sample.sample_id.clone(),
                            r1: sample.r1.clone(),
                            r2: sample.r2.clone(),
                            layout: sample.layout.clone(),
                            status: "skipped_existing_report".to_string(),
                            exit_code: 0,
                            command: Vec::new(),
                            report_json: report_json.display().to_string(),
                        });
                        continue;
                    }
                    reset_sample_payload(&sample_root)?;
                }
            } else if report_json.is_file() {
                runs.push(SampleRunRecord {
                    sample_id: sample.sample_id.clone(),
                    r1: sample.r1.clone(),
                    r2: sample.r2.clone(),
                    layout: sample.layout.clone(),
                    status: "skipped_existing_report".to_string(),
                    exit_code: 0,
                    command: Vec::new(),
                    report_json: report_json.display().to_string(),
                });
                continue;
            }
        }

        let command_args = build_stage_command_args(
            stage_spec,
            &platform,
            &sample,
            &out_root,
            &tools,
            args.threads,
            args.jobs,
            &args.stage_args,
        );
        let command = std::iter::once(program.display().to_string())
            .chain(command_args.iter().cloned())
            .collect::<Vec<_>>();
        if args.dry_run {
            runs.push(SampleRunRecord {
                sample_id: sample.sample_id.clone(),
                r1: sample.r1.clone(),
                r2: sample.r2.clone(),
                layout: sample.layout.clone(),
                status: "dry_run".to_string(),
                exit_code: 0,
                command,
                report_json: report_json.display().to_string(),
            });
            continue;
        }
        pending.push(PendingSampleRun {
            sample,
            report_json,
            command_args,
            command,
            env_overrides: runtime_env.clone(),
        });
    }

    if !pending.is_empty() {
        runs.extend(execute_pending_samples(
            &program,
            &repo_root,
            pending,
            args.sample_jobs,
        )?);
    }

    runs.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    let failures = runs.iter().filter(|row| row.exit_code != 0).count();
    let manifest = CorpusRunManifest {
        schema_version: "bijux.fastq.corpus_benchmark_run.v1".to_string(),
        generated_at_utc: current_timestamp_utc()?,
        corpus_id: corpus_spec.corpus_id.clone(),
        stage_id: contract.stage_id.clone(),
        scenario_id: contract.scenario_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        tool_kind: "benchmark".to_string(),
        platform,
        tools,
        threads: args.threads,
        jobs: args.jobs,
        sample_jobs: args.sample_jobs.max(1),
        sample_limit: (args.sample_limit > 0).then_some(args.sample_limit),
        dry_run: args.dry_run,
        config_path: workspace_config_path.display().to_string(),
        publication_config_path: publication_config_path.display().to_string(),
        repo_root: repo_root.display().to_string(),
        corpus_root: corpus_root.display().to_string(),
        out_root: out_root.display().to_string(),
        stage_args: args.stage_args.clone(),
        samples_total: runs.len(),
        samples_failed: failures,
        runs,
        extra_fields: extra_manifest_fields,
    };
    let manifest_path = out_root.join("run_manifest.json");
    fs::write(
        &manifest_path,
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )
    .with_context(|| format!("write {}", manifest_path.display()))?;

    println!("stage_id={}", contract.stage_id);
    println!("corpus_root={}", corpus_root.display());
    println!("out_root={}", out_root.display());
    println!("manifest={}", manifest_path.display());
    println!("samples_total={}", manifest.samples_total);
    println!("samples_failed={}", manifest.samples_failed);
    if manifest.samples_failed > 0 {
        return Err(anyhow!(
            "corpus benchmark failures: {} samples failed",
            manifest.samples_failed
        ));
    }
    Ok(())
}

fn default_stage_out_root(
    workspace_config: &crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig,
    corpus_id: &str,
    stage_id: &str,
) -> Result<PathBuf> {
    let results_root = workspace_config
        .remote
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_root"))?;
    let stage_template = workspace_config
        .layout
        .as_ref()
        .and_then(|row| row.stage_runs.as_ref())
        .and_then(|row| row.remote_results_template.as_deref())
        .unwrap_or("{corpus_id}/{stage_id}/lunarc");
    let rel = stage_template
        .replace("{corpus_id}", corpus_id)
        .replace("{stage_id}", stage_id);
    Ok(PathBuf::from(results_root).join(rel))
}

fn stage_command_spec(stage_id: &str) -> Result<StageCommandSpec> {
    let spec = match stage_id {
        "fastq.validate_reads" => StageCommandSpec {
            bench_subcommand: "validate-reads",
            report_dir: "validate_reads",
            strict_resume_report: false,
        },
        "fastq.trim_polyg_tails" => StageCommandSpec {
            bench_subcommand: "trim-polyg-tails",
            report_dir: "trim_polyg_tails",
            strict_resume_report: false,
        },
        "fastq.trim_reads" => StageCommandSpec {
            bench_subcommand: "trim-reads",
            report_dir: "trim_reads",
            strict_resume_report: true,
        },
        "fastq.trim_terminal_damage" => StageCommandSpec {
            bench_subcommand: "trim-terminal-damage",
            report_dir: "trim_terminal_damage",
            strict_resume_report: false,
        },
        "fastq.detect_adapters" => StageCommandSpec {
            bench_subcommand: "detect-adapters",
            report_dir: "detect_adapters",
            strict_resume_report: false,
        },
        "fastq.profile_reads" => StageCommandSpec {
            bench_subcommand: "profile-reads",
            report_dir: "profile_reads",
            strict_resume_report: false,
        },
        "fastq.profile_read_lengths" => StageCommandSpec {
            bench_subcommand: "profile-read-lengths",
            report_dir: "profile_read_lengths",
            strict_resume_report: false,
        },
        "fastq.profile_overrepresented_sequences" => StageCommandSpec {
            bench_subcommand: "profile-overrepresented-sequences",
            report_dir: "profile_overrepresented_sequences",
            strict_resume_report: false,
        },
        "fastq.filter_low_complexity" => StageCommandSpec {
            bench_subcommand: "filter-low-complexity",
            report_dir: "filter_low_complexity",
            strict_resume_report: false,
        },
        "fastq.filter_reads" => StageCommandSpec {
            bench_subcommand: "filter",
            report_dir: "filter",
            strict_resume_report: true,
        },
        "fastq.merge_pairs" => StageCommandSpec {
            bench_subcommand: "merge",
            report_dir: "merge_pairs",
            strict_resume_report: false,
        },
        "fastq.report_qc" => StageCommandSpec {
            bench_subcommand: "report-qc",
            report_dir: "report_qc",
            strict_resume_report: false,
        },
        "fastq.remove_duplicates" => StageCommandSpec {
            bench_subcommand: "remove-duplicates",
            report_dir: "remove_duplicates",
            strict_resume_report: false,
        },
        "fastq.normalize_primers" => StageCommandSpec {
            bench_subcommand: "normalize-primers",
            report_dir: "normalize_primers",
            strict_resume_report: false,
        },
        "fastq.deplete_rrna" => StageCommandSpec {
            bench_subcommand: "deplete-rrna",
            report_dir: "deplete_rrna",
            strict_resume_report: true,
        },
        "fastq.deplete_host" => StageCommandSpec {
            bench_subcommand: "deplete-host",
            report_dir: "deplete_host",
            strict_resume_report: true,
        },
        "fastq.deplete_reference_contaminants" => StageCommandSpec {
            bench_subcommand: "deplete-reference-contaminants",
            report_dir: "deplete_reference_contaminants",
            strict_resume_report: true,
        },
        "fastq.screen_taxonomy" => StageCommandSpec {
            bench_subcommand: "screen-taxonomy",
            report_dir: "screen_taxonomy",
            strict_resume_report: true,
        },
        "fastq.correct_errors" => StageCommandSpec {
            bench_subcommand: "correct",
            report_dir: "correct_errors",
            strict_resume_report: true,
        },
        "fastq.extract_umis" => StageCommandSpec {
            bench_subcommand: "umi",
            report_dir: "extract_umis",
            strict_resume_report: true,
        },
        other => return Err(anyhow!("unsupported corpus benchmark stage `{other}`")),
    };
    Ok(spec)
}

fn build_stage_command_args(
    stage_spec: StageCommandSpec,
    platform: &str,
    sample: &CorpusNormalizedSample,
    out_root: &Path,
    tools: &[String],
    threads: u32,
    jobs: u32,
    stage_args: &[String],
) -> Vec<String> {
    let mut command = vec![
        "--platform".to_string(),
        platform.to_string(),
        "bench".to_string(),
        "fastq".to_string(),
        stage_spec.bench_subcommand.to_string(),
        "--sample-id".to_string(),
        sample.sample_id.clone(),
        "--r1".to_string(),
        sample.r1.clone(),
        "--out".to_string(),
        out_root.display().to_string(),
        "--tools".to_string(),
        tools.join(","),
        "--threads".to_string(),
        threads.to_string(),
    ];
    if let Some(r2) = sample.r2.as_ref() {
        command.push("--r2".to_string());
        command.push(r2.clone());
    }
    if jobs > 1 {
        command.push("--jobs".to_string());
        command.push(jobs.to_string());
    }
    command.extend(stage_args.iter().cloned());
    command
}

fn execute_pending_samples(
    program: &Path,
    repo_root: &Path,
    pending: Vec<PendingSampleRun>,
    sample_jobs: usize,
) -> Result<Vec<SampleRunRecord>> {
    if sample_jobs <= 1 || pending.len() <= 1 {
        return pending
            .into_iter()
            .map(|row| execute_sample(program, repo_root, row))
            .collect();
    }

    let queue = Arc::new(Mutex::new(VecDeque::from(pending)));
    let results = Arc::new(Mutex::new(Vec::<SampleRunRecord>::new()));
    let worker_count = sample_jobs.max(1);

    std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::new();
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let program = program.to_path_buf();
            let repo_root = repo_root.to_path_buf();
            handles.push(scope.spawn(move || -> Result<()> {
                loop {
                    let next = {
                        let mut locked = queue
                            .lock()
                            .map_err(|_| anyhow!("benchmark work queue poisoned"))?;
                        locked.pop_front()
                    };
                    let Some(row) = next else {
                        break;
                    };
                    let run = execute_sample(&program, &repo_root, row)?;
                    let mut locked = results
                        .lock()
                        .map_err(|_| anyhow!("benchmark result queue poisoned"))?;
                    locked.push(run);
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow!("corpus benchmark worker panicked"))??;
        }
        Ok(())
    })?;

    let mut locked = results
        .lock()
        .map_err(|_| anyhow!("benchmark result queue poisoned"))?;
    Ok(std::mem::take(&mut *locked))
}

fn execute_sample(
    program: &Path,
    repo_root: &Path,
    row: PendingSampleRun,
) -> Result<SampleRunRecord> {
    let mut command = Command::new(program);
    command
        .args(&row.command_args)
        .current_dir(repo_root)
        .envs(&row.env_overrides);
    let status = command
        .status()
        .with_context(|| format!("run {}", row.command.join(" ")))?;
    let exit_code = status.code().unwrap_or(1);
    Ok(SampleRunRecord {
        sample_id: row.sample.sample_id,
        r1: row.sample.r1,
        r2: row.sample.r2,
        layout: row.sample.layout,
        status: if exit_code == 0 {
            "completed".to_string()
        } else {
            "failed".to_string()
        },
        exit_code,
        command: row.command,
        report_json: row.report_json.display().to_string(),
    })
}

fn load_corpus_spec(repo_root: &Path) -> Result<CorpusSpec> {
    let path = repo_root.join("configs/runtime/corpora/corpus-01.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn corpus_expected_sample_total(spec: &CorpusSpec) -> usize {
    spec.target_ancient_se + spec.target_ancient_pe + spec.target_modern_se + spec.target_modern_pe
}

fn discover_normalized_samples(
    corpus_root: &Path,
    expected_total: usize,
) -> Result<Vec<CorpusNormalizedSample>> {
    let normalized = corpus_root.join("normalized");
    if !normalized.is_dir() {
        return Err(anyhow!(
            "missing normalized corpus directory: {}",
            normalized.display()
        ));
    }

    let mut sample_ids = std::collections::BTreeSet::new();
    for entry in
        fs::read_dir(&normalized).with_context(|| format!("read {}", normalized.display()))?
    {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|row| row.to_str()) else {
            continue;
        };
        if let Some(sample_id) = name.strip_suffix("_R1.fastq.gz") {
            sample_ids.insert(sample_id.to_string());
        }
        if let Some(sample_id) = name.strip_suffix("_R2.fastq.gz") {
            sample_ids.insert(sample_id.to_string());
        }
    }

    let mut samples = Vec::new();
    for sample_id in sample_ids {
        let r1 = normalized.join(format!("{sample_id}_R1.fastq.gz"));
        let r2 = normalized.join(format!("{sample_id}_R2.fastq.gz"));
        if !r1.is_file() {
            return Err(anyhow!(
                "missing R1 for sample {sample_id}: {}",
                r1.display()
            ));
        }
        let r2_value = r2.is_file().then(|| r2.display().to_string());
        samples.push(CorpusNormalizedSample {
            sample_id,
            r1: r1.display().to_string(),
            r2: r2_value.clone(),
            layout: if r2_value.is_some() {
                "pe".to_string()
            } else {
                "se".to_string()
            },
        });
    }

    if samples.len() != expected_total {
        return Err(anyhow!(
            "expected {expected_total} normalized samples for corpus-01, found {}",
            samples.len()
        ));
    }
    Ok(samples)
}

fn validate_corpus_contract(
    corpus_root: &Path,
    spec: &CorpusSpec,
    samples: &[CorpusNormalizedSample],
) -> Result<BTreeMap<String, CorpusSpecSample>> {
    let manifest_path = corpus_root.join("MANIFEST.json");
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: CorpusManifest =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;

    let mut hash_to_accessions = BTreeMap::<String, Vec<String>>::new();
    for (relative_path, digest) in &manifest.files {
        let path = Path::new(relative_path);
        let parts = path.iter().collect::<Vec<_>>();
        if parts.len() >= 2 && parts[0].to_str() == Some("raw") {
            let accession = parts[1].to_string_lossy().to_string();
            hash_to_accessions
                .entry(digest.clone())
                .or_default()
                .push(accession);
        }
    }
    let spec_by_accession = spec
        .samples
        .iter()
        .cloned()
        .map(|row| (row.accession.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut metadata_by_sample = BTreeMap::<String, CorpusSpecSample>::new();
    for (relative_path, digest) in &manifest.files {
        let path = Path::new(relative_path);
        let parts = path.iter().collect::<Vec<_>>();
        if parts.len() != 2 || parts[0].to_str() != Some("normalized") {
            continue;
        }
        let file_name = parts[1].to_string_lossy();
        let sample_id = if let Some(sample_id) = file_name.strip_suffix("_R1.fastq.gz") {
            sample_id.to_string()
        } else if let Some(sample_id) = file_name.strip_suffix("_R2.fastq.gz") {
            sample_id.to_string()
        } else {
            continue;
        };
        let accessions = hash_to_accessions
            .get(digest)
            .ok_or_else(|| anyhow!("missing accession for {}", relative_path))?;
        if accessions.len() != 1 {
            return Err(anyhow!(
                "expected one accession for {}, found {}",
                relative_path,
                accessions.join(",")
            ));
        }
        let accession = &accessions[0];
        let metadata = spec_by_accession
            .get(accession)
            .cloned()
            .ok_or_else(|| anyhow!("missing curated metadata for accession {accession}"))?;
        metadata_by_sample.insert(sample_id, metadata);
    }

    let mut actual_counts = BTreeMap::<String, usize>::new();
    for sample in samples {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing accession metadata for {}", sample.sample_id))?;
        *actual_counts
            .entry(format!("{}_{}", metadata.era, metadata.layout))
            .or_default() += 1;
    }

    let expected_counts = BTreeMap::from([
        ("ancient_pe".to_string(), spec.target_ancient_pe),
        ("ancient_se".to_string(), spec.target_ancient_se),
        ("modern_pe".to_string(), spec.target_modern_pe),
        ("modern_se".to_string(), spec.target_modern_se),
    ]);
    if actual_counts != expected_counts {
        return Err(anyhow!(
            "corpus-01 cohort contract drift: expected {:?}, found {:?}",
            expected_counts,
            actual_counts
        ));
    }
    Ok(metadata_by_sample)
}

fn select_paired_samples(
    spec: &CorpusSpec,
    samples: &[CorpusNormalizedSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
) -> Result<Vec<CorpusNormalizedSample>> {
    let paired = samples
        .iter()
        .filter(|row| {
            metadata_by_sample
                .get(&row.sample_id)
                .is_some_and(|meta| meta.layout == "pe")
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut actual_counts = BTreeMap::<String, usize>::new();
    for sample in &paired {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing paired metadata for {}", sample.sample_id))?;
        *actual_counts
            .entry(format!("{}_{}", metadata.era, metadata.layout))
            .or_default() += 1;
    }
    let expected_counts = BTreeMap::from([
        ("ancient_pe".to_string(), spec.target_ancient_pe),
        ("modern_pe".to_string(), spec.target_modern_pe),
    ]);
    if actual_counts != expected_counts {
        return Err(anyhow!(
            "paired corpus contract drift: expected {:?}, found {:?}",
            expected_counts,
            actual_counts
        ));
    }
    Ok(paired)
}

fn resolve_tools(default_tools: &[String], requested_tools: &[String]) -> Result<Vec<String>> {
    if requested_tools.is_empty() {
        return Ok(default_tools.to_vec());
    }
    let mut expected = default_tools.to_vec();
    expected.sort();
    expected.dedup();
    let mut requested = requested_tools
        .iter()
        .filter_map(|row| {
            let trimmed = row.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect::<Vec<_>>();
    requested.sort();
    requested.dedup();
    if requested != expected {
        return Err(anyhow!(
            "tool roster drift for corpus benchmark: expected {:?}, found {:?}",
            expected,
            requested
        ));
    }
    Ok(default_tools.to_vec())
}

fn validate_benchmark_layout(corpus_root: &Path, out_root: &Path) -> Result<()> {
    if out_root.starts_with(corpus_root) {
        return Err(anyhow!(
            "benchmark output root must not live under the corpus data tree: {}",
            out_root.display()
        ));
    }
    let legacy_root = corpus_root.join("benchmarks");
    if legacy_root.exists() {
        return Err(anyhow!(
            "legacy benchmark outputs still exist under the corpus data tree: {}",
            legacy_root.display()
        ));
    }
    Ok(())
}

fn collect_stage_manifest_fields(
    stage_id: &str,
    stage_args: &[String],
    manifest_args: &[String],
) -> Result<BTreeMap<String, serde_json::Value>> {
    let stage_values = parse_cli_arg_pairs("stage-arg", stage_args)?;
    let manifest_values = parse_cli_arg_pairs("manifest-arg", manifest_args)?;
    let mut fields = BTreeMap::new();

    match stage_id {
        "fastq.normalize_primers" => {
            insert_string_field_from(&mut fields, &stage_values, "primer_set_id");
            insert_string_field_from(&mut fields, &stage_values, "orientation_policy");
            insert_f64_field_from(&mut fields, &stage_values, "max_mismatch_rate")?;
            insert_u64_field_from(&mut fields, &stage_values, "min_overlap_bp")?;
            insert_bool_field_from(&mut fields, &stage_values, "strict_5p_anchor")?;
            insert_bool_field_from(&mut fields, &stage_values, "allow_iupac_codes")?;
        }
        "fastq.remove_duplicates" => {
            insert_string_field_from(&mut fields, &stage_values, "dedup_mode");
            insert_bool_field_from(&mut fields, &stage_values, "keep_order")?;
        }
        "fastq.deplete_host" => {
            if let Some(reference_index) = stage_values.get("reference_index") {
                fields.extend(artifact_bundle_manifest_fields(
                    "reference_index",
                    "reference_index",
                    Path::new(reference_index),
                )?);
            }
            insert_string_field_from(&mut fields, &manifest_values, "reference_catalog_id");
            insert_string_field_from(&mut fields, &manifest_values, "reference_index_backend");
            insert_f64_field_from(&mut fields, &stage_values, "host_identity_threshold")?;
            insert_bool_field_from(&mut fields, &stage_values, "retain_unmapped_only")?;
        }
        "fastq.deplete_reference_contaminants" => {
            if let Some(reference_index) = stage_values.get("reference_index") {
                fields.extend(artifact_bundle_manifest_fields(
                    "reference_index",
                    "reference_index",
                    Path::new(reference_index),
                )?);
            }
            insert_string_field_from(&mut fields, &manifest_values, "reference_catalog_id");
            insert_string_field_from(&mut fields, &manifest_values, "reference_index_backend");
            insert_string_field_from(&mut fields, &stage_values, "decoy_mode");
        }
        "fastq.screen_taxonomy" => {
            if let Some(database_root) = stage_values.get("database_root") {
                fields.extend(artifact_bundle_manifest_fields(
                    "database_root",
                    "database",
                    Path::new(database_root),
                )?);
            }
            insert_string_field_from(&mut fields, &manifest_values, "database_catalog_id");
            insert_string_field_from(&mut fields, &manifest_values, "database_artifact_id");
            insert_string_field_from(&mut fields, &manifest_values, "database_namespace");
            insert_string_field_from(&mut fields, &manifest_values, "database_scope");
        }
        _ => {}
    }

    Ok(fields)
}

fn parse_cli_arg_pairs(label: &str, args: &[String]) -> Result<BTreeMap<String, String>> {
    if args.len() % 2 != 0 {
        return Err(anyhow!(
            "{label} expects flag/value pairs, found odd-length input: {:?}",
            args
        ));
    }
    let mut values = BTreeMap::new();
    for chunk in args.chunks(2) {
        let flag = chunk[0]
            .strip_prefix("--")
            .ok_or_else(|| anyhow!("{label} entry must start with `--`: {}", chunk[0]))?;
        values.insert(flag.replace('-', "_"), chunk[1].clone());
    }
    Ok(values)
}

fn insert_string_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) {
    if let Some(value) = values.get(key) {
        fields.insert(key.to_string(), serde_json::Value::String(value.clone()));
    }
}

fn insert_bool_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) -> Result<()> {
    let Some(value) = values.get(key) else {
        return Ok(());
    };
    fields.insert(
        key.to_string(),
        serde_json::Value::Bool(parse_bool_literal(value)?),
    );
    Ok(())
}

fn insert_f64_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) -> Result<()> {
    let Some(value) = values.get(key) else {
        return Ok(());
    };
    let parsed = value
        .parse::<f64>()
        .with_context(|| format!("parse {key} from {value:?}"))?;
    fields.insert(key.to_string(), serde_json::json!(parsed));
    Ok(())
}

fn insert_u64_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) -> Result<()> {
    let Some(value) = values.get(key) else {
        return Ok(());
    };
    let parsed = value
        .parse::<u64>()
        .with_context(|| format!("parse {key} from {value:?}"))?;
    fields.insert(
        key.to_string(),
        serde_json::Value::Number(serde_json::Number::from(parsed)),
    );
    Ok(())
}

fn parse_bool_literal(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(anyhow!("invalid boolean literal: {value}")),
    }
}

fn artifact_bundle_manifest_fields(
    path_key: &str,
    digest_prefix: &str,
    path: &Path,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut fields = BTreeMap::new();
    fields.insert(
        path_key.to_string(),
        serde_json::Value::String(path.display().to_string()),
    );
    fields.insert(
        format!("{digest_prefix}_digest"),
        serde_json::Value::String(sha256_artifact_bundle(path)?),
    );
    fields.insert(
        format!("{digest_prefix}_size_bytes"),
        serde_json::Value::Number(serde_json::Number::from(artifact_bundle_size_bytes(path)?)),
    );
    if let Some(lineage_json) = resolve_artifact_lineage_json(path) {
        fields.insert(
            format!("{digest_prefix}_lineage_json"),
            serde_json::Value::String(lineage_json.display().to_string()),
        );
        fields.insert(
            format!("{digest_prefix}_lineage_digest"),
            serde_json::Value::String(sha256_file_hex(&lineage_json)?),
        );
    }
    Ok(fields)
}

fn artifact_bundle_members(path: &Path) -> Result<Vec<PathBuf>> {
    if path.exists() {
        return Ok(vec![path.to_path_buf()]);
    }
    let Some(parent) = path.parent() else {
        return Ok(Vec::new());
    };
    if !parent.is_dir() {
        return Ok(Vec::new());
    }
    let Some(prefix) = path.file_name().and_then(|row| row.to_str()) else {
        return Ok(Vec::new());
    };
    let mut members = fs::read_dir(parent)
        .with_context(|| format!("read {}", parent.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;
            (name.starts_with(prefix) && (path.is_file() || path.is_dir())).then_some(path)
        })
        .collect::<Vec<_>>();
    members.sort();
    Ok(members)
}

fn artifact_bundle_size_bytes(path: &Path) -> Result<u64> {
    let mut total = 0_u64;
    for member in artifact_bundle_members(path)? {
        if member.is_file() {
            total += member
                .metadata()
                .with_context(|| format!("stat {}", member.display()))?
                .len();
            continue;
        }
        let mut nested = member
            .read_dir()
            .with_context(|| format!("read {}", member.display()))?
            .filter_map(|entry| entry.ok().map(|row| row.path()))
            .collect::<Vec<_>>();
        while let Some(candidate) = nested.pop() {
            if candidate.is_dir() {
                let children = candidate
                    .read_dir()
                    .with_context(|| format!("read {}", candidate.display()))?
                    .filter_map(|entry| entry.ok().map(|row| row.path()))
                    .collect::<Vec<_>>();
                nested.extend(children);
                continue;
            }
            total += candidate
                .metadata()
                .with_context(|| format!("stat {}", candidate.display()))?
                .len();
        }
    }
    Ok(total)
}

fn sha256_artifact_bundle(path: &Path) -> Result<String> {
    let members = artifact_bundle_members(path)?;
    if members.is_empty() {
        return Err(anyhow!("missing artifact bundle: {}", path.display()));
    }
    let mut digest = sha2::Sha256::new();
    for member in members {
        if member.is_file() {
            let name = member
                .file_name()
                .and_then(|row| row.to_str())
                .ok_or_else(|| anyhow!("invalid artifact bundle member {}", member.display()))?;
            digest.update(name.as_bytes());
            digest.update(b"\0file\0");
            digest.update(sha256_file_hex(&member)?.as_bytes());
            continue;
        }
        let parent = member
            .parent()
            .ok_or_else(|| anyhow!("artifact bundle member missing parent {}", member.display()))?;
        let mut nested = collect_sorted_paths(&member)?;
        for path in nested.drain(..) {
            if path == member {
                continue;
            }
            let relative = path
                .strip_prefix(parent)
                .with_context(|| format!("strip prefix {} from {}", parent.display(), path.display()))?;
            digest.update(path_display(relative).as_bytes());
            if path.is_dir() {
                digest.update(b"\0dir\0");
                continue;
            }
            digest.update(b"\0file\0");
            digest.update(sha256_file_hex(&path)?.as_bytes());
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn collect_sorted_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut all = vec![root.to_path_buf()];
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.is_dir() {
            continue;
        }
        let mut children = fs::read_dir(&path)
            .with_context(|| format!("read {}", path.display()))?
            .filter_map(|entry| entry.ok().map(|row| row.path()))
            .collect::<Vec<_>>();
        children.sort();
        for child in &children {
            all.push(child.to_path_buf());
        }
        children.reverse();
        stack.extend(children);
    }
    all.sort();
    Ok(all)
}

fn resolve_artifact_lineage_json(path: &Path) -> Option<PathBuf> {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let candidate = if resolved.is_dir() {
        resolved.join("lineage.json")
    } else {
        resolved.parent()?.join("lineage.json")
    };
    candidate.is_file().then_some(candidate)
}

fn sha256_file_hex(path: &Path) -> Result<String> {
    let mut handle = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = sha2::Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        use std::io::Read as _;
        let read = handle
            .read(&mut buffer)
            .with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn path_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn benchmark_sample_root(out_root: &Path, report_dir: &str, sample_id: &str) -> PathBuf {
    out_root.join("bench").join(report_dir).join(sample_id)
}

fn reset_sample_payload(sample_root: &Path) -> Result<()> {
    if sample_root.is_dir() {
        fs::remove_dir_all(sample_root)
            .with_context(|| format!("remove stale sample payload {}", sample_root.display()))?;
    }
    Ok(())
}

fn sample_report_is_resume_ready(sample_report: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(sample_report) else {
        return false;
    };
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    if payload
        .get("failures")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|row| !row.is_empty())
    {
        return false;
    }
    if payload
        .get("gate")
        .and_then(|row| row.get("passes"))
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return false;
    }
    payload
        .get("records")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|row| !row.is_empty())
}

fn benchmark_runtime_env(out_root: &Path) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    let Some(cache_root) = workspace_cache_root_for_output(out_root) else {
        return env;
    };
    env.insert(
        "BIJUX_CACHE_ROOT".to_string(),
        cache_root.display().to_string(),
    );
    env.insert("XDG_CACHE_HOME".to_string(), cache_root.display().to_string());
    if cache_root.file_name().and_then(|row| row.to_str()) == Some(".cache") {
        if let Some(parent) = cache_root.parent() {
            env.insert("BIJUX_HPC_ROOT".to_string(), parent.display().to_string());
        }
    }
    env
}

fn workspace_cache_root_for_output(out_root: &Path) -> Option<PathBuf> {
    let resolved = out_root
        .canonicalize()
        .unwrap_or_else(|_| out_root.to_path_buf());
    for candidate in resolved.ancestors() {
        if candidate.file_name().and_then(|row| row.to_str()) == Some(".cache") {
            return Some(candidate.to_path_buf());
        }
    }
    None
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn current_timestamp_utc() -> Result<String> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("resolve benchmark timestamp")?;
    Ok(format!("unix:{}", elapsed.as_secs()))
}

#[cfg(test)]
mod tests {
    use super::{
        benchmark_runtime_env, default_stage_out_root, resolve_tools, sample_report_is_resume_ready,
        stage_command_spec, workspace_cache_root_for_output,
    };
    use crate::commands::benchmark_workspace::{
        BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout, BenchmarkWorkspaceRemote,
        BenchmarkWorkspaceStageRuns,
    };
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn stage_mapping_preserves_filter_report_dir() {
        let spec = stage_command_spec("fastq.filter_reads").expect("stage spec");
        assert_eq!(spec.bench_subcommand, "filter");
        assert_eq!(spec.report_dir, "filter");
        assert!(spec.strict_resume_report);
    }

    #[test]
    fn tool_resolution_rejects_contract_drift() {
        let error = resolve_tools(
            &["fastqc".to_string(), "seqtk".to_string()],
            &["fastqc".to_string()],
        )
        .expect_err("tool drift should fail");
        assert!(error.to_string().contains("tool roster drift"));
    }

    #[test]
    fn default_out_root_uses_workspace_template() {
        let workspace = BenchmarkWorkspaceConfig {
            remote: Some(BenchmarkWorkspaceRemote {
                results_root: Some("/srv/cache/results".to_string()),
                ..BenchmarkWorkspaceRemote::default()
            }),
            layout: Some(BenchmarkWorkspaceLayout {
                stage_runs: Some(BenchmarkWorkspaceStageRuns {
                    remote_results_template: Some("{corpus_id}/{stage_id}/cluster".to_string()),
                    ..BenchmarkWorkspaceStageRuns::default()
                }),
            }),
            ..BenchmarkWorkspaceConfig::default()
        };
        let out_root =
            default_stage_out_root(&workspace, "corpus-01", "fastq.validate_reads").expect("root");
        assert_eq!(
            out_root,
            PathBuf::from("/srv/cache/results/corpus-01/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn resume_ready_requires_successful_gate_and_records() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report = temp.path().join("report.json");
        fs::write(
            &report,
            serde_json::json!({
                "failures": [],
                "gate": {"passes": true},
                "records": [{"tool": "fastp"}],
            })
            .to_string(),
        )
        .expect("write report");
        assert!(sample_report_is_resume_ready(&report));

        fs::write(
            &report,
            serde_json::json!({
                "failures": [{"kind": "tool_exit"}],
                "gate": {"passes": false},
                "records": [],
            })
            .to_string(),
        )
        .expect("write failed report");
        assert!(!sample_report_is_resume_ready(&report));
    }

    #[test]
    fn benchmark_runtime_env_follows_cache_root_ancestor() {
        let out_root =
            PathBuf::from("/tmp/workspace/.cache/results/corpus_01/fastq.trim_reads/lunarc");
        let env = benchmark_runtime_env(&out_root);
        assert_eq!(
            env.get("BIJUX_CACHE_ROOT"),
            Some(&"/tmp/workspace/.cache".to_string())
        );
        assert_eq!(
            env.get("XDG_CACHE_HOME"),
            Some(&"/tmp/workspace/.cache".to_string())
        );
        assert_eq!(
            env.get("BIJUX_HPC_ROOT"),
            Some(&"/tmp/workspace".to_string())
        );
        assert_eq!(
            workspace_cache_root_for_output(&out_root),
            Some(PathBuf::from("/tmp/workspace/.cache"))
        );
    }
}
