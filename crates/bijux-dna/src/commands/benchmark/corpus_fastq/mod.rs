#![allow(clippy::too_many_arguments, clippy::too_many_lines)]

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::commands::benchmark_corpus_metadata::{
    corpus_expected_sample_total, discover_normalized_samples, load_corpus_spec,
    select_paired_samples, validate_corpus_contract, CorpusNormalizedSample,
};
use crate::commands::benchmark_workspace::{
    benchmark_publication_contract, benchmark_runtime_corpus_dir_name,
    benchmark_stage_run_relative_root, benchmark_workspace_value, load_benchmark_config,
    load_benchmark_workspace_config, BenchmarkConfig,
};
use crate::commands::cli::{BenchCorpusFastqArgs, BenchWorkspaceValueArgs, Cli};
use anyhow::{anyhow, Context, Result};

mod artifact_bundle;
mod models;
mod report_qc_support;
mod runtime_support;
mod sortmerna_support;
mod stage_preparation;

const REPORT_QC_INPUTS_SCHEMA_VERSION: &str = "bijux.fastq.report_qc.inputs.v1";
use self::artifact_bundle::artifact_bundle_manifest_fields;
pub(crate) use self::artifact_bundle::{
    artifact_bundle_size_bytes, sha256_artifact_bundle, sha256_file_hex,
};
use self::models::{CorpusRunManifest, PendingSampleRun, PostSuccessAction, SampleRunRecord};
use self::report_qc_support::{
    prepare_report_qc_sample, report_qc_contributor_tool_ids, report_qc_upstream_stage_ids,
};
#[cfg(test)]
use self::runtime_support::workspace_cache_root_for_output;
use self::runtime_support::{
    absolutize, benchmark_runtime_env, benchmark_sample_root, current_timestamp_utc, path_display,
    reset_sample_payload, sample_report_is_resume_ready,
};
use self::sortmerna_support::{
    prepare_sortmerna_sample_workdir, promote_sortmerna_sample_index_cache,
    prune_sortmerna_sample_payload, resolve_deplete_rrna_stage_options, sortmerna_shared_index_dir,
    sortmerna_shared_index_seeded, warm_sortmerna_shared_index_cache,
};
use self::stage_preparation::{stage_command_spec, StageCommandSpec, StageSamplePreparation};

pub(crate) fn print_benchmark_workspace_value(
    cwd: &Path,
    args: &BenchWorkspaceValueArgs,
) -> Result<()> {
    println!("{}", benchmark_workspace_value(cwd, args.config.as_deref(), &args.key)?);
    Ok(())
}

pub(crate) fn run_benchmark_corpus_fastq(cli: &Cli, args: &BenchCorpusFastqArgs) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let benchmark_config = load_benchmark_config(&repo_root, args.config.as_deref())?;
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
    let contract = benchmark_publication_contract(
        &repo_root,
        args.publication_config.as_deref(),
        &args.corpus_id,
        &args.stage,
    )?;
    let stage_spec = stage_command_spec(&args.stage)?;
    let corpus_spec = load_corpus_spec(&repo_root, args.config.as_deref(), &args.corpus_id)?;
    if corpus_spec.corpus_id != args.corpus_id {
        return Err(anyhow!(
            "configured corpus spec drift: expected `{}`, found `{}`",
            args.corpus_id,
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

    let all_samples = discover_normalized_samples(
        &corpus_root,
        &corpus_spec.corpus_id,
        corpus_expected_sample_total(&corpus_spec),
    )?;
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
    let platform = resolve_benchmark_platform(cli.platform.as_deref())?;
    let program = std::env::current_exe().context("resolve bijux-dna executable")?;
    let mut runs = Vec::new();
    let mut pending = Vec::new();
    let runtime_env = benchmark_runtime_env(&out_root);
    let (resolved_stage_args, resolved_manifest_args) = merge_stage_args_from_config(
        &benchmark_config,
        &args.stage,
        &args.stage_args,
        &args.manifest_args,
    )?;
    let mut extra_manifest_fields =
        collect_stage_manifest_fields(&args.stage, &resolved_stage_args, &resolved_manifest_args)?;
    let deplete_rrna_options = if args.stage == "fastq.deplete_rrna" {
        Some(resolve_deplete_rrna_stage_options(&resolved_stage_args)?)
    } else {
        None
    };

    for sample in selected_samples {
        let sample_root =
            benchmark_sample_root(&out_root, stage_spec.report_dir, &sample.sample_id);
        let report_json = out_root
            .join("bench")
            .join(stage_spec.report_dir)
            .join(&sample.sample_id)
            .join("report.json");
        let prepared = prepare_stage_sample(
            &args.stage,
            &program,
            &repo_root,
            &workspace_config,
            &corpus_spec.corpus_id,
            &platform,
            &out_root,
            &sample,
            args.dry_run,
        )?;
        if args.resume {
            if stage_spec.strict_resume_report {
                if sample_root.is_dir() && !report_json.is_file() {
                    reset_sample_payload(&sample_root)?;
                }
                if report_json.is_file() {
                    if sample_report_is_resume_ready(&report_json) {
                        if let Some(options) = deplete_rrna_options.as_ref() {
                            if tools == ["sortmerna"] && !args.dry_run {
                                prune_sortmerna_sample_payload(&out_root, &sample.sample_id)?;
                                extra_manifest_fields.insert(
                                    "rrna_index_dir".to_string(),
                                    serde_json::Value::String(
                                        sortmerna_shared_index_dir(
                                            &out_root,
                                            &options.rrna_bundle_id,
                                        )
                                        .display()
                                        .to_string(),
                                    ),
                                );
                            }
                        }
                        runs.push(SampleRunRecord {
                            sample_id: sample.sample_id.clone(),
                            r1: sample.r1.display().to_string(),
                            r2: sample.r2.as_ref().map(|path| path.display().to_string()),
                            layout: sample.layout.clone(),
                            status: "skipped_existing_report".to_string(),
                            exit_code: 0,
                            command: Vec::new(),
                            report_json: report_json.display().to_string(),
                            extra_fields: prepared.run_extra_fields.clone(),
                        });
                        continue;
                    }
                    reset_sample_payload(&sample_root)?;
                }
            } else if report_json.is_file() {
                runs.push(SampleRunRecord {
                    sample_id: sample.sample_id.clone(),
                    r1: sample.r1.display().to_string(),
                    r2: sample.r2.as_ref().map(|path| path.display().to_string()),
                    layout: sample.layout.clone(),
                    status: "skipped_existing_report".to_string(),
                    exit_code: 0,
                    command: Vec::new(),
                    report_json: report_json.display().to_string(),
                    extra_fields: prepared.run_extra_fields.clone(),
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
            &resolved_stage_args,
            &prepared.extra_stage_args,
        );
        let command = std::iter::once(program.display().to_string())
            .chain(command_args.iter().cloned())
            .collect::<Vec<_>>();
        if args.dry_run {
            runs.push(SampleRunRecord {
                sample_id: sample.sample_id.clone(),
                r1: sample.r1.display().to_string(),
                r2: sample.r2.as_ref().map(|path| path.display().to_string()),
                layout: sample.layout.clone(),
                status: "dry_run".to_string(),
                exit_code: 0,
                command,
                report_json: report_json.display().to_string(),
                extra_fields: prepared.run_extra_fields,
            });
            continue;
        }
        pending.push(PendingSampleRun {
            sample,
            report_json,
            command_args,
            command,
            env_bindings: runtime_env.clone(),
            extra_fields: prepared.run_extra_fields,
            post_success_action: None,
        });
    }

    if !pending.is_empty() {
        if let Some(options) = deplete_rrna_options.as_ref() {
            if tools == ["sortmerna"] && !args.dry_run {
                let shared_idx_dir = sortmerna_shared_index_dir(&out_root, &options.rrna_bundle_id);
                warm_sortmerna_shared_index_cache(
                    &workspace_config,
                    &platform,
                    &options.rrna_db,
                    Path::new(&pending[0].sample.r1),
                    &shared_idx_dir,
                    args.threads,
                )?;
                if !sortmerna_shared_index_seeded(&shared_idx_dir) {
                    let mut first = pending.remove(0);
                    prepare_sortmerna_sample_workdir(
                        &out_root,
                        &first.sample.sample_id,
                        &options.rrna_bundle_id,
                    )?;
                    first.post_success_action =
                        Some(PostSuccessAction::PromoteAndPruneSortmernaCache {
                            out_root: out_root.clone(),
                            sample_id: first.sample.sample_id.clone(),
                            rrna_bundle_id: options.rrna_bundle_id.clone(),
                        });
                    runs.push(execute_sample(&program, &repo_root, first)?);
                }
                for row in &mut pending {
                    prepare_sortmerna_sample_workdir(
                        &out_root,
                        &row.sample.sample_id,
                        &options.rrna_bundle_id,
                    )?;
                    row.post_success_action =
                        Some(PostSuccessAction::PromoteAndPruneSortmernaCache {
                            out_root: out_root.clone(),
                            sample_id: row.sample.sample_id.clone(),
                            rrna_bundle_id: options.rrna_bundle_id.clone(),
                        });
                }
                extra_manifest_fields.insert(
                    "rrna_index_dir".to_string(),
                    serde_json::Value::String(shared_idx_dir.display().to_string()),
                );
                extra_manifest_fields.insert(
                    "rrna_index_seeded".to_string(),
                    serde_json::Value::Bool(sortmerna_shared_index_seeded(&shared_idx_dir)),
                );
            }
        }
        runs.extend(execute_pending_samples(&program, &repo_root, pending, args.sample_jobs)?);
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
        stage_args: resolved_stage_args.clone(),
        samples_total: runs.len(),
        samples_failed: failures,
        runs,
        extra_fields: extra_manifest_fields,
    };
    let manifest_path = out_root.join("run_manifest.json");
    fs::write(&manifest_path, format!("{}\n", serde_json::to_string_pretty(&manifest)?))
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

fn resolve_benchmark_platform(explicit_platform: Option<&str>) -> Result<String> {
    if let Some(platform) = explicit_platform {
        let trimmed = platform.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("benchmark platform must be a non-empty identifier"));
        }
        return Ok(trimmed.to_string());
    }
    std::env::var("BIJUX_PLATFORM")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow!("benchmark platform must be declared with --platform or BIJUX_PLATFORM")
        })
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
    let corpus_dir_name = benchmark_runtime_corpus_dir_name(workspace_config, corpus_id)?;
    Ok(PathBuf::from(results_root).join(benchmark_stage_run_relative_root(
        workspace_config,
        "remote",
        &corpus_dir_name,
        stage_id,
    )?))
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
    extra_stage_args: &[String],
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
        sample.r1.display().to_string(),
        "--out".to_string(),
        out_root.display().to_string(),
        "--tools".to_string(),
        tools.join(","),
        "--threads".to_string(),
        threads.to_string(),
    ];
    if let Some(r2) = sample.r2.as_ref() {
        command.push("--r2".to_string());
        command.push(r2.display().to_string());
    }
    if jobs > 1 {
        command.push("--jobs".to_string());
        command.push(jobs.to_string());
    }
    command.extend(stage_args.iter().cloned());
    command.extend(extra_stage_args.iter().cloned());
    command
}

fn execute_pending_samples(
    program: &Path,
    repo_root: &Path,
    pending: Vec<PendingSampleRun>,
    sample_jobs: usize,
) -> Result<Vec<SampleRunRecord>> {
    if sample_jobs <= 1 || pending.len() <= 1 {
        return pending.into_iter().map(|row| execute_sample(program, repo_root, row)).collect();
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
                        let mut locked =
                            queue.lock().map_err(|_| anyhow!("benchmark work queue poisoned"))?;
                        locked.pop_front()
                    };
                    let Some(row) = next else {
                        break;
                    };
                    let run = execute_sample(&program, &repo_root, row)?;
                    let mut locked =
                        results.lock().map_err(|_| anyhow!("benchmark result queue poisoned"))?;
                    locked.push(run);
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle.join().map_err(|_| anyhow!("corpus benchmark worker panicked"))??;
        }
        Ok(())
    })?;

    let mut locked = results.lock().map_err(|_| anyhow!("benchmark result queue poisoned"))?;
    Ok(std::mem::take(&mut *locked))
}

fn execute_sample(
    program: &Path,
    repo_root: &Path,
    row: PendingSampleRun,
) -> Result<SampleRunRecord> {
    let command_name =
        program.to_str().ok_or_else(|| anyhow!("benchmark executable path is not valid UTF-8"))?;
    let output = bijux_dna_api::v1::api::run::run_command_with_context(
        command_name,
        &row.command_args,
        Some(repo_root),
        Some(&row.env_bindings),
    )
    .with_context(|| format!("run {}", row.command.join(" ")))?;
    let exit_code = output.exit_code;
    if exit_code == 0 {
        if let Some(action) = row.post_success_action.as_ref() {
            match action {
                PostSuccessAction::PromoteAndPruneSortmernaCache {
                    out_root,
                    sample_id,
                    rrna_bundle_id,
                } => {
                    promote_sortmerna_sample_index_cache(out_root, sample_id, rrna_bundle_id)?;
                    prune_sortmerna_sample_payload(out_root, sample_id)?;
                }
            }
        }
    }
    Ok(SampleRunRecord {
        sample_id: row.sample.sample_id,
        r1: row.sample.r1.display().to_string(),
        r2: row.sample.r2.map(|path| path.display().to_string()),
        layout: row.sample.layout,
        status: if exit_code == 0 { "completed".to_string() } else { "failed".to_string() },
        exit_code,
        command: row.command,
        report_json: row.report_json.display().to_string(),
        extra_fields: row.extra_fields,
    })
}

fn merge_stage_args_from_config(
    benchmark_config: &BenchmarkConfig,
    stage_id: &str,
    stage_args: &[String],
    manifest_args: &[String],
) -> Result<(Vec<String>, Vec<String>)> {
    let stage_values = parse_cli_arg_pairs("stage-arg", stage_args)?;
    let manifest_values = parse_cli_arg_pairs("manifest-arg", manifest_args)?;
    let mut merged_stage_args = stage_args.to_vec();
    let mut merged_manifest_args = manifest_args.to_vec();

    match stage_id {
        "fastq.deplete_rrna" => {
            let config = &benchmark_config.stage_inputs.fastq_deplete_rrna;
            append_missing_arg(
                &mut merged_stage_args,
                &stage_values,
                "rrna_db",
                config.rrna_db.as_deref(),
            );
            append_missing_arg(
                &mut merged_stage_args,
                &stage_values,
                "rrna_bundle_id",
                config.rrna_bundle_id.as_deref(),
            );
            append_missing_arg(
                &mut merged_stage_args,
                &stage_values,
                "min_identity",
                config.min_identity.as_deref(),
            );
        }
        "fastq.deplete_host" => {
            let config = &benchmark_config.stage_inputs.fastq_deplete_host;
            append_missing_arg(
                &mut merged_stage_args,
                &stage_values,
                "reference_index",
                config.reference_index.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "reference_catalog_id",
                config.reference_catalog_id.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "reference_index_backend",
                config.reference_index_backend.as_deref(),
            );
        }
        "fastq.deplete_reference_contaminants" => {
            let config = &benchmark_config.stage_inputs.fastq_deplete_reference_contaminants;
            append_missing_arg(
                &mut merged_stage_args,
                &stage_values,
                "reference_index",
                config.reference_index.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "reference_catalog_id",
                config.reference_catalog_id.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "reference_index_backend",
                config.reference_index_backend.as_deref(),
            );
        }
        "fastq.screen_taxonomy" => {
            let config = &benchmark_config.stage_inputs.fastq_screen_taxonomy;
            append_missing_arg(
                &mut merged_stage_args,
                &stage_values,
                "database_root",
                config.database_root.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "database_catalog_id",
                config.database_catalog_id.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "database_artifact_id",
                config.database_artifact_id.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "database_namespace",
                config.database_namespace.as_deref(),
            );
            append_missing_arg(
                &mut merged_manifest_args,
                &manifest_values,
                "database_scope",
                config.database_scope.as_deref(),
            );
        }
        _ => {}
    }

    Ok((merged_stage_args, merged_manifest_args))
}

fn append_missing_arg(
    target: &mut Vec<String>,
    existing: &BTreeMap<String, String>,
    key: &str,
    value: Option<&str>,
) {
    if existing.contains_key(key) {
        return;
    }
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    target.push(format!("--{}", key.replace('_', "-")));
    target.push(value.to_string());
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
            "tool roster drift for corpus benchmark: expected {expected:?}, found {requested:?}"
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

fn prepare_stage_sample(
    stage_id: &str,
    program: &Path,
    repo_root: &Path,
    workspace_config: &crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig,
    corpus_id: &str,
    platform: &str,
    out_root: &Path,
    sample: &CorpusNormalizedSample,
    dry_run: bool,
) -> Result<StageSamplePreparation> {
    match stage_id {
        "fastq.report_qc" => prepare_report_qc_sample(
            program,
            repo_root,
            workspace_config,
            corpus_id,
            platform,
            out_root,
            sample,
            dry_run,
        ),
        _ => Ok(StageSamplePreparation::default()),
    }
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
        "fastq.deplete_rrna" => {
            let options = resolve_deplete_rrna_stage_options(stage_args)?;
            fields.insert(
                "rrna_db".to_string(),
                serde_json::Value::String(options.rrna_db.display().to_string()),
            );
            fields.insert(
                "rrna_bundle_id".to_string(),
                serde_json::Value::String(options.rrna_bundle_id),
            );
            fields.insert(
                "rrna_bundle_digest".to_string(),
                serde_json::Value::String(sha256_artifact_bundle(&options.rrna_db)?),
            );
            fields.insert(
                "rrna_bundle_size_bytes".to_string(),
                serde_json::Value::Number(serde_json::Number::from(artifact_bundle_size_bytes(
                    &options.rrna_db,
                )?)),
            );
            fields.insert("min_identity".to_string(), serde_json::json!(options.min_identity));
        }
        "fastq.report_qc" => {
            fields.insert(
                "aggregation_engine".to_string(),
                serde_json::Value::String("multiqc".to_string()),
            );
            fields.insert(
                "aggregation_scope".to_string(),
                serde_json::Value::String("governed_qc_artifacts".to_string()),
            );
            fields.insert("report_only".to_string(), serde_json::Value::Bool(true));
            fields.insert("mutates_fastq".to_string(), serde_json::Value::Bool(false));
            fields.insert("may_change_read_count".to_string(), serde_json::Value::Bool(false));
            fields.insert(
                "governed_contributor_stage_ids".to_string(),
                serde_json::Value::Array(
                    report_qc_upstream_stage_ids()
                        .iter()
                        .map(|row| serde_json::Value::String(row.clone()))
                        .collect(),
                ),
            );
            fields.insert(
                "governed_contributor_tool_ids".to_string(),
                serde_json::Value::Array(
                    report_qc_contributor_tool_ids()
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }
        _ => {}
    }

    Ok(fields)
}

fn parse_cli_arg_pairs(label: &str, args: &[String]) -> Result<BTreeMap<String, String>> {
    if args.len() % 2 != 0 {
        return Err(anyhow!("{label} expects flag/value pairs, found odd-length input: {args:?}"));
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
    fields.insert(key.to_string(), serde_json::Value::Bool(parse_bool_literal(value)?));
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
    let parsed = value.parse::<f64>().with_context(|| format!("parse {key} from {value:?}"))?;
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
    let parsed = value.parse::<u64>().with_context(|| format!("parse {key} from {value:?}"))?;
    fields.insert(key.to_string(), serde_json::Value::Number(serde_json::Number::from(parsed)));
    Ok(())
}

fn parse_bool_literal(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(anyhow!("invalid boolean literal: {value}")),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::sortmerna_support::supports_apptainer_warmup;
    use super::{
        benchmark_runtime_env, default_stage_out_root, prepare_report_qc_sample,
        prepare_sortmerna_sample_workdir, promote_sortmerna_sample_index_cache,
        resolve_benchmark_platform, resolve_deplete_rrna_stage_options, resolve_tools,
        sample_report_is_resume_ready, stage_command_spec, workspace_cache_root_for_output,
        CorpusNormalizedSample,
    };
    use crate::commands::benchmark_workspace::{
        BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout, BenchmarkWorkspaceRemote,
        BenchmarkWorkspaceStageRuns,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    #[test]
    fn stage_mapping_preserves_filter_report_dir() {
        let spec = stage_command_spec("fastq.filter_reads").expect("stage spec");
        assert_eq!(spec.bench_subcommand, "filter");
        assert_eq!(spec.report_dir, "filter");
        assert!(spec.strict_resume_report);
    }

    #[test]
    fn tool_resolution_rejects_contract_drift() {
        let error =
            resolve_tools(&["fastqc".to_string(), "seqtk".to_string()], &["fastqc".to_string()])
                .expect_err("tool drift should fail");
        assert!(error.to_string().contains("tool roster drift"));
    }

    #[test]
    fn default_out_root_uses_workspace_template() {
        let workspace = BenchmarkWorkspaceConfig {
            remote: Some(BenchmarkWorkspaceRemote {
                corpus_root: Some("/bench/remote/corpora/benchmark_corpus".to_string()),
                results_root: Some("/bench/remote/cache/results".to_string()),
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
            default_stage_out_root(&workspace, "benchmark-corpus", "fastq.validate_reads")
                .expect("root");
        assert_eq!(
            out_root,
            PathBuf::from(
                "/bench/remote/cache/results/benchmark_corpus/fastq.validate_reads/cluster",
            )
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
        let out_root = PathBuf::from(
            "/bench/workspace/.cache/results/benchmark_corpus/fastq.trim_reads/cluster-apptainer",
        );
        let env = benchmark_runtime_env(&out_root);
        assert_eq!(env.get("BIJUX_CACHE_ROOT"), Some(&"/bench/workspace/.cache".to_string()));
        assert_eq!(env.get("XDG_CACHE_HOME"), Some(&"/bench/workspace/.cache".to_string()));
        assert!(!env.contains_key("BIJUX_HPC_ROOT"));
        assert_eq!(
            workspace_cache_root_for_output(&out_root),
            Some(PathBuf::from("/bench/workspace/.cache"))
        );
    }

    #[test]
    fn report_qc_preparation_writes_governed_manifest_during_dry_run() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let out_root = temp.path().join("results").join("corpus-01").join("fastq.report_qc");
        fs::create_dir_all(&repo_root).expect("repo root");
        fs::create_dir_all(&out_root).expect("out root");
        let workspace = BenchmarkWorkspaceConfig {
            remote: Some(BenchmarkWorkspaceRemote {
                corpus_root: Some(
                    temp.path().join("remote-corpora").join("corpus-01").display().to_string(),
                ),
                results_root: Some(temp.path().join("remote-results").display().to_string()),
                ..BenchmarkWorkspaceRemote::default()
            }),
            layout: Some(BenchmarkWorkspaceLayout {
                stage_runs: Some(BenchmarkWorkspaceStageRuns {
                    remote_results_template: Some(
                        "{corpus_id}/{stage_id}/cluster-apptainer".to_string(),
                    ),
                    ..BenchmarkWorkspaceStageRuns::default()
                }),
            }),
            ..BenchmarkWorkspaceConfig::default()
        };
        let corpus_root = temp.path().join("corpus");
        let sample = CorpusNormalizedSample {
            sample_id: "sample_0001".to_string(),
            r1: corpus_root.join("sample_0001_R1.fastq.gz"),
            r2: Some(corpus_root.join("sample_0001_R2.fastq.gz")),
            layout: "pe".to_string(),
        };

        let prepared = prepare_report_qc_sample(
            Path::new("/bin/true"),
            &repo_root,
            &workspace,
            "corpus-01",
            "cluster-apptainer",
            &out_root,
            &sample,
            true,
        )
        .expect("prepare report qc sample");

        assert_eq!(
            prepared.extra_stage_args,
            vec![
                "--aggregation-engine".to_string(),
                "multiqc".to_string(),
                "--aggregation-scope".to_string(),
                "governed_qc_artifacts".to_string(),
                "--governed-qc-manifest".to_string(),
                out_root
                    .join("bench")
                    .join("report_qc")
                    .join("sample_0001")
                    .join("governed_qc_inputs_manifest.json")
                    .display()
                    .to_string(),
            ]
        );
        assert_eq!(
            prepared.run_extra_fields.get("governed_qc_input_count"),
            Some(&serde_json::Value::Number(serde_json::Number::from(6_u64)))
        );
        let governed_manifest = out_root
            .join("bench")
            .join("report_qc")
            .join("sample_0001")
            .join("governed_qc_inputs_manifest.json");
        let expected_raw_fastqc_dir = temp
            .path()
            .join("remote-results")
            .join("corpus-01")
            .join("fastq.detect_adapters")
            .join("cluster-apptainer")
            .join("bench")
            .join("detect_adapters")
            .join("sample_0001")
            .join("tools")
            .join("fastqc")
            .join("fastqc")
            .display()
            .to_string();
        let payload: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&governed_manifest).expect("read governed manifest"),
        )
        .expect("parse governed manifest");
        assert_eq!(
            payload.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.fastq.report_qc.inputs.v1")
        );
        assert_eq!(
            payload.get("qc_inputs").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
            Some(6)
        );
        assert_eq!(
            payload.get("raw_fastqc_dir").and_then(serde_json::Value::as_str),
            Some(expected_raw_fastqc_dir.as_str())
        );
    }

    #[test]
    fn report_qc_preparation_reuses_existing_validation_backend_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let out_root = temp.path().join("results").join("corpus-01").join("fastq.report_qc");
        let remote_results = temp.path().join("remote-results");
        fs::create_dir_all(&repo_root).expect("repo root");
        fs::create_dir_all(&out_root).expect("out root");

        let workspace = BenchmarkWorkspaceConfig {
            remote: Some(BenchmarkWorkspaceRemote {
                corpus_root: Some(
                    temp.path().join("remote-corpora").join("corpus-01").display().to_string(),
                ),
                results_root: Some(remote_results.display().to_string()),
                ..BenchmarkWorkspaceRemote::default()
            }),
            layout: Some(BenchmarkWorkspaceLayout {
                stage_runs: Some(BenchmarkWorkspaceStageRuns {
                    remote_results_template: Some(
                        "{corpus_id}/{stage_id}/cluster-apptainer".to_string(),
                    ),
                    ..BenchmarkWorkspaceStageRuns::default()
                }),
            }),
            ..BenchmarkWorkspaceConfig::default()
        };
        let corpus_root = temp.path().join("corpus");
        let sample = CorpusNormalizedSample {
            sample_id: "sample_0001".to_string(),
            r1: corpus_root.join("sample_0001_R1.fastq.gz"),
            r2: Some(corpus_root.join("sample_0001_R2.fastq.gz")),
            layout: "pe".to_string(),
        };

        let fastqc_root = remote_results
            .join("corpus-01")
            .join("fastq.validate_reads")
            .join("cluster-apptainer")
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("tools")
            .join("fastqc");
        fs::create_dir_all(&fastqc_root).expect("create fastqc validation root");
        fs::write(fastqc_root.join("validation.json"), "{\n  \"tool_id\": \"fastqc\"\n}\n")
            .expect("write fastqc validation report");
        fs::write(
            fastqc_root.join("validated_reads_manifest.json"),
            "{\n  \"tool_id\": \"fastqc\"\n}\n",
        )
        .expect("write fastqc validated reads manifest");

        let prepared = prepare_report_qc_sample(
            Path::new("/bin/true"),
            &repo_root,
            &workspace,
            "corpus-01",
            "cluster-apptainer",
            &out_root,
            &sample,
            true,
        )
        .expect("prepare report qc sample");
        assert_eq!(
            prepared.run_extra_fields.get("governed_qc_input_count"),
            Some(&serde_json::Value::Number(serde_json::Number::from(6_u64)))
        );

        let governed_manifest = out_root
            .join("bench")
            .join("report_qc")
            .join("sample_0001")
            .join("governed_qc_inputs_manifest.json");
        let payload: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&governed_manifest).expect("read governed manifest"),
        )
        .expect("parse governed manifest");
        let qc_inputs = payload
            .get("qc_inputs")
            .and_then(serde_json::Value::as_array)
            .expect("qc inputs array");
        assert!(
            qc_inputs.iter().any(|entry| {
                entry.get("name").and_then(serde_json::Value::as_str)
                    == Some("fastq.validate_reads.tool.fastqc.validation_report")
                    && entry.get("path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                        path.ends_with(
                            "/fastq.validate_reads/cluster-apptainer/bench/validate_reads/sample_0001/tools/fastqc/validation.json",
                        )
                    })
            }),
            "report_qc must reuse an existing fastqc validation report when it is already present"
        );
        assert!(
            qc_inputs.iter().any(|entry| {
                entry.get("name").and_then(serde_json::Value::as_str)
                    == Some("fastq.validate_reads.tool.fastqc.validated_reads_manifest")
                    && entry.get("path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                        path.ends_with(
                            "/fastq.validate_reads/cluster-apptainer/bench/validate_reads/sample_0001/tools/fastqc/validated_reads_manifest.json",
                        )
                    })
            }),
            "report_qc must reuse an existing fastqc lineage manifest when it is already present"
        );
    }

    #[test]
    fn report_qc_manifest_fields_advertise_all_validation_backends() {
        let fields = super::collect_stage_manifest_fields("fastq.report_qc", &[], &[])
            .expect("report_qc fields");
        let tool_ids = fields
            .get("governed_contributor_tool_ids")
            .and_then(serde_json::Value::as_array)
            .expect("governed contributor tool ids");
        for tool_id in
            ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqkit_stats", "seqtk"]
        {
            assert!(
                tool_ids.iter().any(|entry| entry.as_str() == Some(tool_id)),
                "report_qc contributor metadata must advertise {tool_id}"
            );
        }
    }

    #[test]
    fn supports_apptainer_warmup_accepts_apptainer_platform_names() {
        assert!(supports_apptainer_warmup("apptainer-amd64"));
        assert!(supports_apptainer_warmup("cluster-apptainer"));
        assert!(!supports_apptainer_warmup("docker-amd64"));
    }

    #[test]
    fn resolve_benchmark_platform_rejects_empty_identifier() {
        let error =
            resolve_benchmark_platform(Some("   ")).expect_err("empty platform contract must fail");
        assert!(error.to_string().contains("benchmark platform must be a non-empty identifier"));
    }

    #[test]
    fn deplete_rrna_stage_options_require_existing_rrna_bundle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let rrna_db = temp.path().join("sortmerna_v4_3_default_db.fasta");
        fs::write(&rrna_db, ">rrna\nACGT\n").expect("write rrna fasta");

        let options = resolve_deplete_rrna_stage_options(&[
            "--rrna-db".to_string(),
            rrna_db.display().to_string(),
            "--rrna-bundle-id".to_string(),
            "sortmerna_v4_3_default_db".to_string(),
            "--min-identity".to_string(),
            "0.95".to_string(),
        ])
        .expect("resolve rrna stage options");
        assert_eq!(options.rrna_db, rrna_db);
        assert_eq!(options.rrna_bundle_id, "sortmerna_v4_3_default_db");
        assert!((options.min_identity - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn deplete_rrna_stage_options_require_declared_bundle_id() {
        let temp = tempfile::tempdir().expect("tempdir");
        let rrna_db = temp.path().join("sortmerna_v4_3_default_db.fasta");
        fs::write(&rrna_db, ">rrna\nACGT\n").expect("write rrna fasta");

        let error = resolve_deplete_rrna_stage_options(&[
            "--rrna-db".to_string(),
            rrna_db.display().to_string(),
        ])
        .expect_err("missing rrna bundle id must fail");
        assert!(error
            .to_string()
            .contains("fastq.deplete_rrna requires --rrna-bundle-id in stage arguments"));
    }

    #[test]
    fn sortmerna_workdir_preparation_clones_and_promotes_shared_cache() {
        let temp = tempfile::tempdir().expect("tempdir");
        let out_root = temp.path().join("results");

        let shared_idx_dir =
            prepare_sortmerna_sample_workdir(&out_root, "sample_0001", "sortmerna_v4_3_default_db")
                .expect("prepare empty workdir");
        assert!(shared_idx_dir.is_dir());

        let sample_one_idx = out_root
            .join("bench")
            .join("deplete_rrna")
            .join("sample_0001")
            .join("tools")
            .join("sortmerna")
            .join("sortmerna_workdir")
            .join("idx")
            .join("reference.stats");
        fs::write(&sample_one_idx, "seed").expect("write sample idx seed");
        let promoted = promote_sortmerna_sample_index_cache(
            &out_root,
            "sample_0001",
            "sortmerna_v4_3_default_db",
        )
        .expect("promote shared idx");
        assert!(promoted.join("reference.stats").is_file());

        prepare_sortmerna_sample_workdir(&out_root, "sample_0002", "sortmerna_v4_3_default_db")
            .expect("prepare seeded workdir");
        let sample_two_idx = out_root
            .join("bench")
            .join("deplete_rrna")
            .join("sample_0002")
            .join("tools")
            .join("sortmerna")
            .join("sortmerna_workdir")
            .join("idx")
            .join("reference.stats");
        assert!(sample_two_idx.is_file());
    }
}
