use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{fetch_fastq_trim_v2, insert_fastq_trim_v2};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqTrimMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_domain_fastq::TrimReadsReportV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::select_trim_tools;
use bijux_dna_planner_fastq::stage_api::fastq::trim_reads::{
    plan_with_options, validate_trim_toolset_support, TrimPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, inspect_headers, log_header_warnings,
    polyx_bank_context, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use super::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, json_string,
    observe_fastq_stats, prepare_trim_bench, TrimBenchInputs,
};
use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_READS,
};

mod policy;

use self::policy::{
    adapter_bank_requested, adapter_policy_uses_bank, benchmark_query_context,
    contaminant_policy_uses_bank, normalized_adapter_policy, normalized_contaminant_policy,
    normalized_polyx_policy, polyx_policy_uses_bank,
};

fn apply_thread_override(
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    threads: Option<u32>,
) -> bijux_dna_core::prelude::ToolExecutionSpecV1 {
    let mut spec = tool_spec.clone();
    if let Some(threads) = threads {
        spec.resources.threads = threads.max(1);
    }
    spec
}

fn load_governed_trim_report(report_path: &std::path::Path) -> Result<TrimReadsReportV1> {
    let raw = std::fs::read_to_string(report_path)
        .with_context(|| format!("read governed trim report {}", report_path.display()))?;
    bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw)
        .with_context(|| format!("parse governed trim report {}", report_path.display()))
}

fn write_governed_trim_report(
    report_path: &std::path::Path,
    report: &TrimReadsReportV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report)
        .with_context(|| format!("write governed trim report {}", report_path.display()))
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<FastqTrimMetrics>> {
    let tools = select_trim_benchmark_tools(args)?;
    let setup = prepare_trim_benchmark_setup(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_trim_benchmark_explain(&setup)?;
    }
    ensure_trim_benchmark_qa(catalog, platform, &setup.tools)?;

    let policy = resolve_trim_policy_context(args)?;
    validate_trim_toolset_support(&setup.tools, args.r2.is_some(), &policy.trim_options)?;

    let sqlite_path = setup.bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = setup.bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqTrimMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in setup.tools {
        let out_dir = setup.bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_TRIM_READS.as_str(),
            &tool,
            &setup.registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let tool_spec = apply_thread_override(&tool_spec, args.threads);
        let plan = plan_with_options(
            &tool_spec,
            &setup.bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            policy.adapter_context.as_ref(),
            policy.polyx_context.as_ref(),
            policy.contaminant_context.as_ref(),
            &policy.trim_options,
        )?;
        let bench_params = benchmark_query_context(
            policy.adapter_context.as_ref(),
            policy.polyx_context.as_ref(),
            policy.contaminant_context.as_ref(),
        )?
        .embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_trim_v2(
            &conn,
            &tool,
            &tool_spec.tool_version,
            &image_digest,
            &setup.bench_inputs.runner.to_string(),
            &platform.name,
            &setup.input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }

        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            setup.bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;

        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_TRIM_READS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let output_r1 = plan.io.outputs[0].path.clone();
        let output_stats_r1 =
            observe_fastq_stats(catalog, platform, setup.bench_inputs.runner, &output_r1)?;
        let output_stats_r2 = if args.r2.is_some() {
            Some(observe_fastq_stats(
                catalog,
                platform,
                setup.bench_inputs.runner,
                &plan.io.outputs[1].path,
            )?)
        } else {
            None
        };
        let before_stats =
            combine_seqkit_metrics(&setup.bench_inputs.input_stats, setup.input_stats_r2.as_ref());
        let after_stats = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
        let report_path = out_dir.join("trim_report.json");
        let mut governed_report = load_governed_trim_report(&report_path)?;
        governed_report.reads_in = Some(before_stats.reads);
        governed_report.reads_out = Some(after_stats.reads);
        governed_report.bases_in = Some(before_stats.bases);
        governed_report.bases_out = Some(after_stats.bases);
        governed_report.pairs_in = setup
            .input_stats_r2
            .as_ref()
            .map(|stats| setup.bench_inputs.input_stats.reads.min(stats.reads));
        governed_report.pairs_out =
            output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads));
        governed_report.mean_q_before = Some(before_stats.mean_q);
        governed_report.mean_q_after = Some(after_stats.mean_q);
        governed_report.runtime_s = Some(execution.runtime_s);
        governed_report.memory_mb = Some(execution.memory_mb);
        write_governed_trim_report(&report_path, &governed_report)?;
        let metrics = FastqTrimMetrics {
            reads_in: before_stats.reads,
            reads_out: after_stats.reads,
            bases_in: before_stats.bases,
            bases_out: after_stats.bases,
            pairs_in: setup
                .input_stats_r2
                .as_ref()
                .map(|stats| setup.bench_inputs.input_stats.reads.min(stats.reads)),
            pairs_out: output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads)),
            mean_q_before: before_stats.mean_q,
            mean_q_after: after_stats.mean_q,
            delta_metrics: derive_trim_delta(&before_stats, &after_stats),
            paired_mode: Some(
                match governed_report.paired_mode {
                    bijux_dna_domain_fastq::PairedMode::SingleEnd => "single_end",
                    bijux_dna_domain_fastq::PairedMode::PairedEnd => "paired_end",
                    bijux_dna_domain_fastq::PairedMode::Unknown => "not_declared",
                }
                .to_string(),
            ),
            adapter_policy: Some(governed_report.adapter_policy.clone()),
            polyx_policy: governed_report.polyx_policy.clone(),
            n_policy: governed_report.n_policy.clone(),
            contaminant_policy: governed_report.contaminant_policy.clone(),
            raw_backend_report_format: governed_report.raw_backend_report_format.clone(),
            adapter_preset: governed_report.adapter_preset.clone(),
            adapter_bank_id: governed_report.adapter_bank_id.clone(),
            adapter_bank_hash: governed_report.adapter_bank_hash.clone(),
            adapter_overrides: governed_report.adapter_overrides.clone().map(Into::into),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        let metrics_path = out_dir.join("metrics.json");
        bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_json)
            .context("write trim metrics")?;
        prune_trim_tool_payload(&out_dir, &report_path, &metrics_path, &governed_report)?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            setup.bench_inputs.runner,
            platform,
            setup.input_hash.clone(),
            bench_params.clone(),
        );
        let record = BenchmarkRecord {
            context,
            execution: ExecutionMetrics {
                runtime_s: execution.runtime_s,
                memory_mb: execution.memory_mb,
                exit_code: execution.exit_code,
            },
            metrics: metric_set,
        };
        record.validate()?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_trim_v2(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: setup.bench_inputs.bench_dir,
        explain: args.explain,
    })
}

fn select_trim_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<Vec<String>> {
    let tools = select_trim_tools(&args.tools, false)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_TRIM_READS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_READS.as_str(), &header);
    Ok(tools)
}

struct TrimBenchmarkSetup {
    registry: ToolRegistry,
    tools: Vec<String>,
    excluded_tools: Vec<String>,
    bench_inputs: TrimBenchInputs,
    input_hash: String,
    input_stats_r2: Option<SeqkitMetrics>,
}

struct TrimPolicyContext {
    trim_options: TrimPlanOptions,
    adapter_context: Option<serde_json::Value>,
    polyx_context: Option<serde_json::Value>,
    contaminant_context: Option<serde_json::Value>,
}

fn resolve_trim_policy_context(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> Result<TrimPolicyContext> {
    let adapter_policy =
        normalized_adapter_policy(args.adapter_policy.as_deref(), adapter_bank_requested(args))?;
    let adapter_context = if adapter_policy_uses_bank(adapter_policy.as_deref()) {
        adapter_bank_context(
            args.adapter_bank_preset.as_deref(),
            args.adapter_bank.as_deref(),
            args.adapter_bank_file.as_deref(),
            &args.enable_adapters,
            &args.disable_adapters,
        )?
    } else {
        None
    };
    let polyx_policy =
        normalized_polyx_policy(args.polyx_policy.as_deref(), args.polyx_preset.is_some())?;
    let polyx_context = if polyx_policy_uses_bank(polyx_policy.as_deref()) {
        polyx_bank_context(args.polyx_preset.as_deref())?
    } else {
        None
    };
    let contaminant_policy = normalized_contaminant_policy(
        args.contaminant_policy.as_deref(),
        args.contaminant_preset.is_some(),
    )?;
    let contaminant_context = if contaminant_policy_uses_bank(contaminant_policy.as_deref()) {
        contaminant_bank_context(args.contaminant_preset.as_deref())?
    } else {
        None
    };
    let trim_options = TrimPlanOptions {
        threads: args.threads,
        min_length: args.min_length,
        quality_cutoff: args.quality_cutoff,
        n_policy: args.n_policy.clone(),
        adapter_policy,
        polyx_policy,
        contaminant_policy,
    };
    Ok(TrimPolicyContext { trim_options, adapter_context, polyx_context, contaminant_context })
}

fn prepare_trim_benchmark_setup<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
    tools: &[String],
) -> Result<TrimBenchmarkSetup> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_TRIM_READS.as_str(), tools, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_READS,
    )?;
    let input_hash = if let Some(r2) = args.r2.as_deref() {
        format!("{}+{}", bench_inputs.input_hash, bijux_dna_infra::hash_file_sha256(r2)?)
    } else {
        bench_inputs.input_hash.clone()
    };
    let input_stats_r2 = if let Some(r2) = args.r2.as_deref() {
        Some(observe_fastq_stats(catalog, platform, bench_inputs.runner, r2)?)
    } else {
        None
    };
    let excluded_tools = excluded_trim_tools(&registry, &tools);
    Ok(TrimBenchmarkSetup {
        registry,
        tools,
        excluded_tools,
        bench_inputs,
        input_hash,
        input_stats_r2,
    })
}

fn excluded_trim_tools(registry: &ToolRegistry, selected_tools: &[String]) -> Vec<String> {
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_READS.as_str());
    registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .filter(|tool| !selected_tools.contains(tool))
        .collect()
}

fn write_trim_benchmark_explain(setup: &TrimBenchmarkSetup) -> Result<()> {
    write_explain_md(
        &setup.bench_inputs.bench_dir,
        STAGE_TRIM_READS.as_str(),
        &setup.tools,
        &setup.excluded_tools,
        None,
    )?;
    write_explain_plan_json(
        &setup.bench_inputs.bench_dir,
        STAGE_TRIM_READS.as_str(),
        &setup.tools,
        &setup.registry,
        None,
    )
}

fn ensure_trim_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_TRIM_READS.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM_READS.as_str(), tools, platform, catalog)
}

fn combine_seqkit_metrics(
    primary: &SeqkitMetrics,
    secondary: Option<&SeqkitMetrics>,
) -> SeqkitMetrics {
    let secondary_reads = secondary.map_or(0, |stats| stats.reads);
    let secondary_bases = secondary.map_or(0, |stats| stats.bases);
    let total_bases = primary.bases + secondary_bases;
    let weighted_mean_q = if total_bases == 0 {
        0.0
    } else {
        ((primary.mean_q * u64_to_f64(primary.bases))
            + secondary.map_or(0.0, |stats| stats.mean_q * u64_to_f64(stats.bases)))
            / u64_to_f64(total_bases)
    };
    let weighted_gc = if total_bases == 0 {
        0.0
    } else {
        ((primary.gc_percent * u64_to_f64(primary.bases))
            + secondary.map_or(0.0, |stats| stats.gc_percent * u64_to_f64(stats.bases)))
            / u64_to_f64(total_bases)
    };
    SeqkitMetrics {
        reads: primary.reads + secondary_reads,
        bases: total_bases,
        mean_q: weighted_mean_q,
        gc_percent: weighted_gc,
    }
}

fn prune_trim_tool_payload(
    out_dir: &Path,
    report_path: &Path,
    metrics_path: &Path,
    report: &TrimReadsReportV1,
) -> Result<()> {
    let run_artifacts_dir = out_dir.join("run_artifacts");
    let mut keep = HashSet::new();
    keep.insert(report_path.to_path_buf());
    keep.insert(metrics_path.to_path_buf());
    if let Some(raw_backend_report) = report.raw_backend_report.as_ref() {
        keep.insert(Path::new(raw_backend_report).to_path_buf());
    }

    let mut dirs = vec![out_dir.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in
            fs::read_dir(&dir).with_context(|| format!("read trim tool dir {}", dir.display()))?
        {
            let path = entry.with_context(|| format!("read entry in {}", dir.display()))?.path();
            if path == run_artifacts_dir || path.starts_with(&run_artifacts_dir) {
                continue;
            }
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if keep.contains(&path) {
                continue;
            }
            fs::remove_file(&path)
                .with_context(|| format!("prune trim payload {}", path.display()))?;
        }
    }

    Ok(())
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        adapter_bank_requested, adapter_policy_uses_bank, apply_thread_override,
        benchmark_query_context, contaminant_policy_uses_bank, normalized_adapter_policy,
        normalized_contaminant_policy, normalized_polyx_policy, polyx_policy_uses_bank,
        prune_trim_tool_payload, write_governed_trim_report,
    };
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use bijux_dna_domain_fastq::{PairedMode, TrimReadsReportV1, TRIM_READS_REPORT_SCHEMA_VERSION};
    use std::fs;

    fn dummy_tool(tool_id: &'static str, threads: u32) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static(tool_id),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 { image: format!("{tool_id}:latest"), digest: None },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads,
            },
        }
    }

    #[test]
    fn benchmark_query_context_captures_governed_trim_bank_hashes() {
        let adapter_context = serde_json::json!({"bank_hash": "adapter-hash"});
        let polyx_context = serde_json::json!({"bank_hash": "polyx-hash"});
        let contaminant_context = serde_json::json!({"bank_hash": "contaminant-hash"});

        let context = benchmark_query_context(
            Some(&adapter_context),
            Some(&polyx_context),
            Some(&contaminant_context),
        )
        .unwrap_or_else(|err| panic!("query context: {err}"));

        assert!(context.stage_contract_hash.is_some());
        assert_eq!(
            context.bank_hashes.get("adapter_bank").map(String::as_str),
            Some("adapter-hash")
        );
        assert_eq!(context.bank_hashes.get("polyx_bank").map(String::as_str), Some("polyx-hash"));
        assert_eq!(
            context.bank_hashes.get("contaminant_bank").map(String::as_str),
            Some("contaminant-hash")
        );
    }

    #[test]
    fn implicit_trim_banks_stay_disabled_without_policy_or_explicit_selection() {
        assert_eq!(
            normalized_adapter_policy(None, false).unwrap_or_else(|err| panic!("{err}")),
            None
        );
        assert_eq!(
            normalized_polyx_policy(None, false).unwrap_or_else(|err| panic!("{err}")),
            None
        );
        assert_eq!(
            normalized_contaminant_policy(None, false).unwrap_or_else(|err| panic!("{err}")),
            None
        );
        assert!(!adapter_policy_uses_bank(None));
        assert!(!polyx_policy_uses_bank(None));
        assert!(!contaminant_policy_uses_bank(None));
    }

    #[test]
    fn explicit_trim_bank_selection_promotes_missing_policy_to_bank() {
        assert_eq!(
            normalized_adapter_policy(None, true).unwrap_or_else(|err| panic!("{err}")).as_deref(),
            Some("bank")
        );
        assert_eq!(
            normalized_polyx_policy(None, true).unwrap_or_else(|err| panic!("{err}")).as_deref(),
            Some("bank")
        );
        assert_eq!(
            normalized_contaminant_policy(None, true)
                .unwrap_or_else(|err| panic!("{err}"))
                .as_deref(),
            Some("bank")
        );
    }

    #[test]
    fn adapter_policy_supports_ancient_strict_without_forcing_explicit_flags() {
        assert_eq!(
            normalized_adapter_policy(Some("ancient_strict"), false)
                .unwrap_or_else(|err| panic!("{err}"))
                .as_deref(),
            Some("ancient_strict")
        );
        assert!(adapter_policy_uses_bank(Some("ancient_strict")));
    }

    #[test]
    fn adapter_bank_requested_detects_any_explicit_adapter_selection() {
        let args = bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs {
            sample_id: "sample".to_string(),
            r1: "reads_R1.fastq.gz".into(),
            r2: None,
            out: "out".into(),
            tools: vec!["fastp".to_string()],
            explain: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            threads: None,
            adapter_bank_preset: Some("illumina-default".to_string()),
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            min_length: None,
            quality_cutoff: None,
            n_policy: None,
            adapter_policy: None,
            polyx_policy: None,
            contaminant_policy: None,
        };

        assert!(adapter_bank_requested(&args));
    }

    #[test]
    fn write_governed_trim_report_preserves_contract_shape() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let report_path = temp.path().join("trim_report.json");
        let report = TrimReadsReportV1 {
            schema_version: TRIM_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 4,
            input_r1: "reads.fastq.gz".to_string(),
            input_r2: None,
            output_r1: "trimmed.fastq.gz".to_string(),
            output_r2: None,
            min_length: 30,
            quality_cutoff: None,
            adapter_policy: "none".to_string(),
            polyx_policy: Some("none".to_string()),
            n_policy: Some("retain".to_string()),
            contaminant_policy: Some("none".to_string()),
            adapter_bank_id: None,
            adapter_bank_hash: None,
            adapter_preset: None,
            adapter_overrides: Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"],
            })),
            polyx_bank_id: None,
            polyx_bank_hash: None,
            polyx_preset: None,
            contaminant_bank_id: None,
            contaminant_bank_hash: None,
            contaminant_preset: None,
            reads_in: Some(100),
            reads_out: Some(95),
            bases_in: Some(1000),
            bases_out: Some(900),
            pairs_in: None,
            pairs_out: None,
            mean_q_before: Some(28.0),
            mean_q_after: Some(30.0),
            runtime_s: Some(1.5),
            memory_mb: Some(64.0),
            raw_backend_report: Some("trim.fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
        };

        write_governed_trim_report(&report_path, &report)
            .unwrap_or_else(|err| panic!("write report: {err}"));
        let raw = std::fs::read_to_string(&report_path)
            .unwrap_or_else(|err| panic!("read report: {err}"));
        let decoded: TrimReadsReportV1 =
            serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse report: {err}"));
        assert_eq!(decoded.tool_id, "fastp");
        assert_eq!(decoded.threads, 4);
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastp_json"));
        assert_eq!(
            decoded.adapter_overrides,
            Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"],
            }))
        );
    }

    #[test]
    fn thread_override_replaces_governed_trim_threads() {
        let tool = dummy_tool("fastp", 2);
        let overridden = apply_thread_override(&tool, Some(8));
        assert_eq!(overridden.resources.threads, 8);
    }

    #[test]
    fn prune_trim_tool_payload_keeps_reports_and_run_artifacts() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out_dir = temp.path().join("fastp");
        let run_artifacts = out_dir.join("run_artifacts");
        fs::create_dir_all(&run_artifacts).expect("mkdir");

        let report_path = out_dir.join("trim_report.json");
        let metrics_path = out_dir.join("metrics.json");
        let raw_backend_report = out_dir.join("trim.fastp.json");
        let trimmed_r1 = out_dir.join("reads_R1.fastq.gz");
        let trimmed_r2 = out_dir.join("reads_R2.fastq.gz");
        let stage_report = run_artifacts.join("stage_report.json");

        fs::write(&report_path, "{}").expect("write report");
        fs::write(&metrics_path, "{}").expect("write metrics");
        fs::write(&raw_backend_report, "{}").expect("write backend report");
        fs::write(&trimmed_r1, "trimmed").expect("write r1");
        fs::write(&trimmed_r2, "trimmed").expect("write r2");
        fs::write(&stage_report, "{}").expect("write run artifact");

        let report = TrimReadsReportV1 {
            schema_version: TRIM_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: trimmed_r1.display().to_string(),
            output_r2: Some(trimmed_r2.display().to_string()),
            min_length: 30,
            quality_cutoff: None,
            adapter_policy: "none".to_string(),
            polyx_policy: Some("none".to_string()),
            n_policy: Some("retain".to_string()),
            contaminant_policy: Some("none".to_string()),
            adapter_bank_id: None,
            adapter_bank_hash: None,
            adapter_preset: None,
            adapter_overrides: None,
            polyx_bank_id: None,
            polyx_bank_hash: None,
            polyx_preset: None,
            contaminant_bank_id: None,
            contaminant_bank_hash: None,
            contaminant_preset: None,
            reads_in: Some(100),
            reads_out: Some(90),
            bases_in: Some(1000),
            bases_out: Some(900),
            pairs_in: Some(50),
            pairs_out: Some(45),
            mean_q_before: Some(28.0),
            mean_q_after: Some(30.0),
            runtime_s: Some(1.0),
            memory_mb: Some(64.0),
            raw_backend_report: Some(raw_backend_report.display().to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
        };

        prune_trim_tool_payload(&out_dir, &report_path, &metrics_path, &report)
            .unwrap_or_else(|err| panic!("prune payload: {err}"));

        assert!(report_path.is_file());
        assert!(metrics_path.is_file());
        assert!(raw_backend_report.is_file());
        assert!(stage_report.is_file());
        assert!(!trimmed_r1.exists());
        assert!(!trimmed_r2.exists());
    }
}
