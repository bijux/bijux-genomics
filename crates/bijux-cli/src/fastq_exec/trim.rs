use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_analyze::{
    append_jsonl, fetch_fastq_trim_v2, insert_fastq_trim_v2, metric_set, AdapterTrimmingSummary,
    BenchmarkContext, BenchmarkRecord, FastqDeltaMetrics, FastqTrimMetrics,
};
use bijux_core::measure::ExecutionMetrics;
use bijux_engine::api::{ensure_bench_runner, load_registry};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};
use uuid::Uuid;

use bijux_domain_fastq::{
    adapter_bank_path, adapter_presets_path, contract_for_stage, inspect_headers,
    load_adapter_bank, load_adapter_presets, log_header_warnings, normalize_outputs,
    preflight_stage, resolve_adapter_preset, FastqArtifact,
};
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{bench_base_dir, bench_tools_dir};
use bijux_engine::api::{cleanup_execution, execution_memory_mb, run_tool_execution};
use bijux_engine::api::{hash_file_sha256, input_fastq_stats, output_fastq_stats};
use bijux_environment::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};

use crate::fastq_exec::helpers::{
    compute_run_id, normalize_tool_list, params_hash, prepare_tool_run_dirs, resolve_image_for_run,
    write_adapter_bank_ref, write_adapter_trimming_report, write_effective_adapters,
    write_execution_logs, write_explain_md, write_explain_plan_json, write_metrics_json,
    write_retention_report_placeholder, write_run_manifest, ExecutionManifest, RunArtifactInput,
};
use crate::fastq_exec::helpers::{filter_tools_by_role, BenchOutcome};
use bijux_domain_fastq::RawFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
pub struct TrimPlan {
    pub tool: String,
    pub input: std::path::PathBuf,
    pub output: std::path::PathBuf,
}

#[cfg(test)]
fn trim_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "cutadapt" => Some("cutadapt.fastq.gz"),
        "atropos" => Some("atropos.fastq.gz"),
        "bbduk" => Some("bbduk.fastq.gz"),
        "adapterremoval" => Some("adapterremoval.fastq.gz"),
        "trimmomatic" => Some("trimmomatic.fastq.gz"),
        "trim_galore" => Some("trimmed_trimmed.fq.gz"),
        "seqpurge" => Some("seqpurge.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        _ => None,
    }
}

/// Build a trim command plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
#[cfg(test)]
pub fn plan_trim(tool: &str, r1: &std::path::Path, out_dir: &std::path::Path) -> Result<TrimPlan> {
    let output_name =
        trim_output_name(tool).ok_or_else(|| anyhow!("unsupported trim tool: {tool}"))?;
    Ok(TrimPlan {
        tool: tool.to_string(),
        input: r1.to_path_buf(),
        output: out_dir.join(output_name),
    })
}

#[allow(clippy::too_many_lines)]
/// Run the FASTQ benchmark stage.
///
/// # Errors
/// Returns an error if planning, execution, or metric recording fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_domain_fastq::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<FastqTrimMetrics>> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.trim", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.trim", &header);
    let tools = normalize_tool_list(&args.tools)?;
    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.trim", &tools, &registry, false)?;
    let bench_dir = bench_base_dir(&args.out, "trim", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "trim", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    println!("planned tools: {}", tools.join(", "));
    let selected = tools.clone();
    let all_tools: Vec<String> = registry
        .tools_for_stage("fastq.trim")
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !selected.contains(tool))
        .collect();
    write_explain_md(&bench_dir, "fastq.trim", &selected, &excluded, None)?;
    write_explain_plan_json(&bench_dir, "fastq.trim", &selected, &registry, None)?;

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

    let adapter_bank_path = args.adapter_bank.clone().unwrap_or_else(adapter_bank_path);
    let adapter_bank = load_adapter_bank(&adapter_bank_path)?;
    let presets_path = adapter_presets_path();
    let adapter_presets = load_adapter_presets(&presets_path, &adapter_bank)?;
    let effective_adapters = resolve_adapter_preset(
        &adapter_bank,
        &adapter_presets,
        &args.adapter_preset,
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let adapter_bank_checksum = hash_file_sha256(&adapter_bank_path)?;
    let adapter_presets_checksum = hash_file_sha256(&presets_path)?;

    ensure_image_qa_passed("fastq.trim", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.trim", &tools, platform, catalog)?;

    let sqlite_path = bench_dir.join("bench.sqlite");
    let conn = bijux_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let mut records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();
    let mut new_records: Vec<BenchmarkRecord<FastqTrimMetrics>> = Vec::new();
    let mut failures: Vec<RawFailure> = Vec::new();

    for tool in tools {
        let record = (|| -> Result<BenchmarkRecord<FastqTrimMetrics>> {
            let spec = catalog
                .get(&tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
            let image = resolve_image_for_run(spec, platform)?;
            let image_digest = spec
                .digest
                .as_ref()
                .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
                .to_string();
            let cached =
                fetch_fastq_trim_v2(&conn, &tool, &spec.version, &image_digest, &input_hash);
            if let Ok(Some(record)) = cached {
                return Ok(record);
            }

            let params = serde_json::json!({
                "sample_id": args.sample_id,
                "r1": r1,
                "adapter_preset": args.adapter_preset,
                "adapter_bank": adapter_bank_path,
                "enable_adapters": args.enable_adapters,
                "disable_adapters": args.disable_adapters,
                "adapter_bank_checksum": adapter_bank_checksum,
                "adapter_presets_checksum": adapter_presets_checksum,
            });
            let param_hash = params_hash(&params).unwrap_or_else(|_| Uuid::new_v4().to_string());
            let run_id =
                compute_run_id("fastq.trim", &tool, &image_digest, &input_hash, &param_hash);
            let run_dirs = prepare_tool_run_dirs(&tools_root, &tool, &run_id)?;
            let out_dir = run_dirs.artifacts_dir.clone();
            let start = Instant::now();
            let container_name = format!("bijux-bench-{}-{}", args.sample_id, Uuid::new_v4());
            let execution =
                run_tool_execution(&tool, &image, &r1_dir, &r1, &out_dir, &container_name)?;
            let runtime_s = start.elapsed().as_secs_f64();
            let memory_mb = execution_memory_mb(&container_name)?;
            cleanup_execution(&container_name)?;

            let contract = contract_for_stage("fastq.trim")
                .ok_or_else(|| anyhow!("missing fastq.trim contract"))?;
            let normalized = normalize_outputs("fastq.trim", &out_dir, contract.output_kind)?;
            let out_fastq = normalized
                .r1
                .as_ref()
                .ok_or_else(|| anyhow!("output fastq missing"))?;
            let output_stats = output_fastq_stats(&seqkit_image, &out_dir, out_fastq)?;

            let tool_manifest = registry
                .tool_by_id("fastq.trim", &tool)
                .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
            validate_execution_outputs(&tool_manifest.execution_contract, &out_dir)?;

            let manifest = ExecutionManifest {
                run_id: run_id.clone(),
                stage: "fastq.trim".to_string(),
                tool: tool.clone(),
                tool_version: spec.version.clone(),
                image_digest: image_digest.clone(),
                command: execution.command.clone(),
                input_hashes: vec![input_hash.clone()],
                input_files: vec![r1.display().to_string()],
                output_dir: out_dir.display().to_string(),
                runner: runner.to_string(),
                platform: platform.name.clone(),
                arch: platform.arch.clone(),
            };
            fs::write(
                &run_dirs.manifest_path,
                serde_json::to_vec_pretty(&manifest)?,
            )
            .context("write execution manifest")?;
            write_execution_logs(&run_dirs, &execution.stdout, &execution.stderr)?;

            let context = BenchmarkContext {
                tool: tool.clone(),
                tool_version: spec.version.clone(),
                image_digest,
                runner: runner.to_string(),
                platform: platform.name.clone(),
                input_hash: input_hash.clone(),
                parameters: params.clone(),
            };
            let execution_metrics = ExecutionMetrics {
                runtime_s,
                memory_mb,
                exit_code: execution.exit_code,
            };
            let effective_adapters_path = write_effective_adapters(
                &run_dirs,
                &effective_adapters,
                &adapter_bank_checksum,
                &adapter_presets_checksum,
            )?;
            let adapter_bank_ref_path = write_adapter_bank_ref(
                &run_dirs,
                &adapter_bank,
                &adapter_bank_path,
                &presets_path,
                &adapter_bank_checksum,
                &adapter_presets_checksum,
                &effective_adapters,
            )?;
            let adapter_report_path = write_adapter_trimming_report(
                &run_dirs,
                &tool,
                &params,
                &effective_adapters.enabled_ids,
                input_stats.reads,
            )?;
            let summary = AdapterTrimmingSummary {
                reads_with_any_adapter: Some(0),
                total_reads: Some(input_stats.reads),
                bases_trimmed_total: Some(0),
                top_k_adapters: Vec::new(),
            };
            let delta = bijux_domain_fastq::compute_delta(input_stats, output_stats);
            let metric_set = metric_set(FastqTrimMetrics {
                reads_in: input_stats.reads,
                reads_out: output_stats.reads,
                bases_in: input_stats.bases,
                bases_out: output_stats.bases,
                mean_q_before: input_stats.mean_q,
                mean_q_after: output_stats.mean_q,
                delta_metrics: FastqDeltaMetrics {
                    read_retention: delta.read_retention,
                    base_retention: delta.base_retention,
                    mean_q_delta: delta.delta_mean_q,
                    gc_delta: delta.delta_gc,
                },
                adapter_preset: Some(effective_adapters.preset.clone()),
                adapter_bank_checksum: Some(adapter_bank_checksum.clone()),
                effective_adapters_path: Some(effective_adapters_path.display().to_string()),
                adapter_trimming_summary: Some(summary),
            });
            bijux_analyze::validate_metric_set(&metric_set)?;
            let envelope = &metric_set;
            write_metrics_json(&run_dirs, &execution_metrics, envelope)?;
            write_retention_report_placeholder(&run_dirs, "fastq.trim", &tool, &params)?;
            write_run_manifest(
                &run_dirs,
                "fastq.trim",
                &tool,
                &adapter_bank_path,
                &[
                    RunArtifactInput {
                        name: "effective_adapters",
                        path: effective_adapters_path.clone(),
                    },
                    RunArtifactInput {
                        name: "adapter_bank_ref",
                        path: adapter_bank_ref_path.clone(),
                    },
                    RunArtifactInput {
                        name: "adapter_trimming_report",
                        path: adapter_report_path.clone(),
                    },
                ],
            )?;
            let record = BenchmarkRecord {
                context,
                execution: execution_metrics,
                metrics: metric_set,
            };
            record.validate()?;
            Ok(record)
        })();
        match record {
            Ok(record) => new_records.push(record),
            Err(err) => failures.push(RawFailure {
                stage: "fastq.trim".to_string(),
                tool: tool.to_string(),
                reason: err.to_string(),
            }),
        }
    }

    records.extend(new_records.iter().cloned());

    let bench_path = bench_dir.join("bench.jsonl");
    for record in &new_records {
        append_jsonl(&bench_path, record).context("write bench.jsonl")?;
    }

    for record in &new_records {
        insert_fastq_trim_v2(&conn, record).context("insert bench sqlite")?;
    }

    check_fastq_trim_comparability(&records);
    Ok(BenchOutcome {
        records,
        failures,
        bench_dir,
        explain: args.explain,
    })
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::{plan_trim, trim_output_name};
    use anyhow::Result;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn trim_output_names_are_defined_for_known_tools() {
        assert_eq!(trim_output_name("fastp"), Some("fastp.fastq.gz"));
        assert_eq!(
            trim_output_name("trimmomatic"),
            Some("trimmomatic.fastq.gz")
        );
        assert_eq!(trim_output_name("unknown"), None);
    }

    #[test]
    fn plan_trim_builds_expected_paths() -> Result<()> {
        let plan = plan_trim("fastp", Path::new("reads.fastq.gz"), Path::new("out"))?;
        assert_eq!(plan.output.to_string_lossy(), "out/fastp.fastq.gz");
        Ok(())
    }

    #[test]
    fn plan_trim_rejects_unknown_tool() {
        match plan_trim("mystery", Path::new("reads.fastq.gz"), Path::new("out")) {
            Ok(_) => panic!("expected unsupported trim tool"),
            Err(err) => assert!(err.to_string().contains("unsupported trim tool")),
        }
    }

    #[test]
    fn default_adapter_preset_writes_effective_adapters() -> Result<()> {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
        let prev_dir = std::env::current_dir()?;
        std::env::set_current_dir(repo_root)?;
        let bank_path = bijux_domain_fastq::adapter_bank_path();
        let presets_path = bijux_domain_fastq::adapter_presets_path();
        let bank = bijux_domain_fastq::load_adapter_bank(&bank_path)?;
        let presets = bijux_domain_fastq::load_adapter_presets(&presets_path, &bank)?;
        let effective =
            bijux_domain_fastq::resolve_adapter_preset(&bank, &presets, "default_adna", &[], &[])?;
        let tmp = TempDir::new()?;
        let tools_root = tmp.path().join("tools");
        let run_dirs =
            crate::fastq_exec::helpers::prepare_tool_run_dirs(&tools_root, "fastp", "test-run")?;
        let path = crate::fastq_exec::helpers::write_effective_adapters(
            &run_dirs, &effective, "bank", "presets",
        )?;
        let payload = std::fs::read_to_string(&path)?;
        assert!(payload.contains("truseq_universal"));
        assert!(payload.contains("truseq_indexed"));
        std::env::set_current_dir(prev_dir)?;
        Ok(())
    }

    #[test]
    fn disabling_adapter_changes_params_hash() -> Result<()> {
        let base = serde_json::json!({
            "adapter_preset": "default_adna",
            "enable_adapters": [],
            "disable_adapters": []
        });
        let disabled = serde_json::json!({
            "adapter_preset": "default_adna",
            "enable_adapters": [],
            "disable_adapters": ["truseq_universal"]
        });
        let base_hash = crate::fastq_exec::helpers::params_hash(&base)?;
        let disabled_hash = crate::fastq_exec::helpers::params_hash(&disabled)?;
        assert_ne!(base_hash, disabled_hash);
        Ok(())
    }

    #[test]
    fn ssdna_preset_changes_params_hash() -> Result<()> {
        let base = serde_json::json!({
            "adapter_preset": "default_adna",
            "enable_adapters": [],
            "disable_adapters": []
        });
        let ssdna = serde_json::json!({
            "adapter_preset": "ssdna",
            "enable_adapters": [],
            "disable_adapters": []
        });
        let base_hash = crate::fastq_exec::helpers::params_hash(&base)?;
        let ssdna_hash = crate::fastq_exec::helpers::params_hash(&ssdna)?;
        assert_ne!(base_hash, ssdna_hash);
        Ok(())
    }
}

#[allow(clippy::cast_precision_loss)]
fn check_fastq_trim_comparability(records: &[BenchmarkRecord<FastqTrimMetrics>]) {
    if records.len() <= 1 {
        return;
    }
    let first = &records[0];
    let mut reads_in = first.metrics.metrics.reads_in;
    let mut bases_in = first.metrics.metrics.bases_in;
    let mut mean_q_before = first.metrics.metrics.mean_q_before;

    for record in records.iter().skip(1) {
        if record.metrics.metrics.reads_in != reads_in {
            tracing::warn!(
                tool = record.context.tool,
                reads_in = record.metrics.metrics.reads_in,
                "reads_in differs from baseline"
            );
            reads_in = record.metrics.metrics.reads_in;
        }
        if record.metrics.metrics.bases_in != bases_in {
            tracing::warn!(
                tool = record.context.tool,
                bases_in = record.metrics.metrics.bases_in,
                "bases_in differs from baseline"
            );
            bases_in = record.metrics.metrics.bases_in;
        }
        if (record.metrics.metrics.mean_q_before - mean_q_before).abs() > 1e-6 {
            tracing::warn!(
                tool = record.context.tool,
                mean_q_before = record.metrics.metrics.mean_q_before,
                "mean_q_before differs from baseline"
            );
            mean_q_before = record.metrics.metrics.mean_q_before;
        }
        if record.metrics.metrics.reads_in > 0 {
            let loss = 1.0
                - (record.metrics.metrics.reads_out as f64
                    / record.metrics.metrics.reads_in as f64);
            if loss < -1e-6 {
                tracing::warn!(
                    tool = record.context.tool,
                    reads_in = record.metrics.metrics.reads_in,
                    reads_out = record.metrics.metrics.reads_out,
                    "reads_out exceeds reads_in"
                );
            }
        }
    }
}
