use std::collections::HashMap;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::query_shared::{
    fetch_fastq_trim_polyg_v1, insert_fastq_trim_polyg_v1,
};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqTrimPolygMetrics};
use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_domain_fastq::observer::{parse_bbduk_reads_removed, parse_fastp_metrics};
use bijux_dna_domain_fastq::TrimPolygReportV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, polyx_bank_context, preflight_stage, FastqArtifactKind,
    RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use super::trim_bench_common::{
    benchmark_image_identity, build_benchmark_context, derive_trim_delta, observe_fastq_stats,
    prepare_trim_bench,
};
use crate::internal::handlers::fastq::jobs::{bench_jobs, execute_plans_with_jobs};
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_TRIM_POLYG_TAILS,
};
use std::path::{Path, PathBuf};

fn load_governed_trim_polyg_report(report_path: &Path) -> Result<TrimPolygReportV1> {
    let raw = std::fs::read_to_string(report_path)
        .with_context(|| format!("read governed trim-polyg report {}", report_path.display()))?;
    bijux_dna_domain_fastq::observer::parse_trim_polyg_report(&raw)
        .with_context(|| format!("parse governed trim-polyg report {}", report_path.display()))
}

fn write_governed_trim_polyg_report(report_path: &Path, report: &TrimPolygReportV1) -> Result<()> {
    bijux_dna_infra::atomic_write_json(report_path, report)
        .with_context(|| format!("write governed trim-polyg report {}", report_path.display()))
}

fn resolve_requested_tools(raw: &[String]) -> Vec<String> {
    if raw.is_empty() || (raw.len() == 1 && raw[0] == "auto") {
        return admitted_stage_tools();
    }
    if raw.len() == 1 && raw[0] == "all" {
        return admitted_stage_tools();
    }
    raw.to_vec()
}

fn admitted_stage_tools() -> Vec<String> {
    bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&StageId::new(
        STAGE_TRIM_POLYG_TAILS.as_str(),
    ))
    .into_iter()
    .map(|tool_id| tool_id.to_string())
    .collect()
}

/// # Errors
/// Returns an error if planning, execution, metric derivation, or persistence fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_trim_polyg_tails<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimPolygArgs,
) -> Result<BenchOutcome<FastqTrimPolygMetrics>> {
    let requested = resolve_requested_tools(&args.tools);
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_TRIM_POLYG_TAILS.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_TRIM_POLYG_TAILS.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_TRIM_POLYG_TAILS.as_str(), &requested, &registry, false)?;
    let bench_inputs = prepare_trim_bench(
        catalog,
        platform,
        runner_override,
        &args.sample_id,
        &args.out,
        &args.r1,
        &STAGE_TRIM_POLYG_TAILS,
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

    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_TRIM_POLYG_TAILS.as_str());
    let all_tools: Vec<String> =
        registry.tools_for_stage(&stage_id).iter().map(|tool| tool.tool_id.to_string()).collect();
    let excluded: Vec<String> =
        all_tools.into_iter().filter(|tool| !tools.contains(tool)).collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_TRIM_POLYG_TAILS.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_TRIM_POLYG_TAILS.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_TRIM_POLYG_TAILS.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_TRIM_POLYG_TAILS.as_str(), &tools, platform, catalog)?;

    let polyx_context = polyx_bank_context(args.polyx_preset.as_deref())?;
    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut records = Vec::<BenchmarkRecord<FastqTrimPolygMetrics>>::new();
    let mut failures = Vec::<RawFailure>::new();

    for tool in tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_TRIM_POLYG_TAILS.as_str(),
            &tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = bijux_dna_planner_fastq::stage_api::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
            &tool_spec,
            &bench_inputs.r1,
            args.r2.as_deref(),
            &out_dir,
            &bijux_dna_planner_fastq::stage_api::fastq::trim_polyg_tails::TrimPolygPlanOptions {
                threads: args.threads,
                trim_polyg: args.trim_polyg.unwrap_or(true),
                min_polyg_run: args.min_polyg_run.unwrap_or(10),
            },
        )?;
        let bench_params =
            benchmark_query_context(polyx_context.as_ref())?.embed_in_parameters(&plan.params);
        let params_hash = stable_params_hash(&bench_params);
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_trim_polyg_v1(
            &conn,
            &tool,
            &tool_spec.tool_version,
            &image_digest,
            &bench_inputs.runner.to_string(),
            &platform.name,
            &input_hash,
            &params_hash,
        ) {
            records.push(record);
            continue;
        }

        let execution = execute_plans_with_jobs(
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&plan)],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;

        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_TRIM_POLYG_TAILS.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }

        let output_r1 = plan.io.outputs[0].path.clone();
        let output_stats_r1 =
            observe_fastq_stats(catalog, platform, bench_inputs.runner, &output_r1)?;
        let output_stats_r2 = if args.r2.is_some() {
            Some(observe_fastq_stats(
                catalog,
                platform,
                bench_inputs.runner,
                &plan.io.outputs[1].path,
            )?)
        } else {
            None
        };
        let before_stats =
            combine_seqkit_metrics(&bench_inputs.input_stats, input_stats_r2.as_ref());
        let after_stats = combine_seqkit_metrics(&output_stats_r1, output_stats_r2.as_ref());
        let (raw_report_path, raw_report_format) = raw_polyg_report_artifact(&tool, &out_dir)?;
        let backend_metrics = normalized_polyg_backend_metrics(&raw_report_path, raw_report_format)
            .context("normalize trim polyg backend report")?;
        let report_path = out_dir.join("trim_polyg_tails_report.json");
        let mut governed_report = load_governed_trim_polyg_report(&report_path)?;
        governed_report.reads_in = Some(before_stats.reads);
        governed_report.reads_out = Some(after_stats.reads);
        governed_report.bases_in = Some(before_stats.bases);
        governed_report.bases_out = Some(after_stats.bases);
        governed_report.pairs_in =
            input_stats_r2.as_ref().map(|stats| bench_inputs.input_stats.reads.min(stats.reads));
        governed_report.pairs_out =
            output_stats_r2.as_ref().map(|stats| output_stats_r1.reads.min(stats.reads));
        governed_report.mean_q_before = Some(before_stats.mean_q);
        governed_report.mean_q_after = Some(after_stats.mean_q);
        governed_report.bases_trimmed_polyg =
            Some(before_stats.bases.saturating_sub(after_stats.bases));
        governed_report.runtime_s = Some(execution.runtime_s);
        governed_report.memory_mb = Some(execution.memory_mb);
        governed_report.raw_backend_report = Some(raw_report_path.display().to_string());
        governed_report.raw_backend_report_format = Some(raw_report_format.to_string());
        governed_report.backend_metrics = Some(backend_metrics.clone());
        governed_report.polyx_bank_id = polyx_context
            .as_ref()
            .and_then(|value| value.get("bank_id"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        governed_report.polyx_bank_hash = polyx_context
            .as_ref()
            .and_then(|value| value.get("bank_hash"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        governed_report.polyx_preset = polyx_context
            .as_ref()
            .and_then(|value| value.get("preset"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        write_governed_trim_polyg_report(&report_path, &governed_report)?;
        let metrics = FastqTrimPolygMetrics {
            reads_in: before_stats.reads,
            reads_out: after_stats.reads,
            bases_in: before_stats.bases,
            bases_out: after_stats.bases,
            pairs_in: input_stats_r2
                .as_ref()
                .map(|stats| bench_inputs.input_stats.reads.min(stats.reads)),
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
            threads: Some(governed_report.threads),
            trim_polyg: Some(governed_report.trim_polyg),
            min_polyg_run: Some(governed_report.min_polyg_run),
            bases_trimmed_polyg: governed_report.bases_trimmed_polyg,
            raw_backend_report_format: governed_report.raw_backend_report_format.clone(),
            polyx_bank_id: governed_report.polyx_bank_id.clone(),
            polyx_bank_hash: governed_report.polyx_bank_hash.clone(),
            polyx_preset: governed_report.polyx_preset.clone(),
        };
        let metric_set = metric_set(metrics.clone());
        bijux_dna_analyze::validate_metric_set(&metric_set)?;
        let metrics_json = serde_json::to_value(&metric_set)?;
        bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
            .context("write trim polyg metrics")?;

        let context = build_benchmark_context(
            &tool,
            tool_spec.tool_version.clone(),
            image_digest,
            bench_inputs.runner,
            platform,
            input_hash.clone(),
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
        insert_fastq_trim_polyg_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: bench_inputs.bench_dir, explain: args.explain })
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

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn benchmark_query_context(
    polyx_context: Option<&serde_json::Value>,
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    let mut context = bijux_dna_domain_fastq::governed_stage_bench_query_context(
        STAGE_TRIM_POLYG_TAILS.as_str(),
    )?;
    if let Some(bank_hash) =
        polyx_context.and_then(|value| value.get("bank_hash")).and_then(serde_json::Value::as_str)
    {
        context = context.with_bank_hash("polyx_bank", bank_hash.to_string());
    }
    Ok(context)
}

fn raw_polyg_report_artifact(tool_id: &str, out_dir: &Path) -> Result<(PathBuf, &'static str)> {
    match tool_id {
        "fastp" => Ok((out_dir.join("trim_polyg_tails_report.fastp.json"), "fastp_json")),
        "bbduk" => Ok((out_dir.join("trim_polyg_tails_report.stats.txt"), "bbduk_stats")),
        _ => Err(anyhow!("unsupported trim_polyg_tails raw report artifact for tool {tool_id}")),
    }
}

fn normalized_polyg_backend_metrics(
    raw_report_path: &Path,
    raw_report_format: &str,
) -> Result<serde_json::Value> {
    let raw_backend_report =
        std::fs::read_to_string(raw_report_path).context("read trim polyg backend report")?;
    match raw_report_format {
        "fastp_json" => {
            let metrics = parse_fastp_metrics(&raw_backend_report)
                .context("parse fastp polyg backend metrics")?;
            Ok(serde_json::to_value(metrics).context("serialize fastp polyg backend metrics")?)
        }
        "bbduk_stats" => {
            let reads_removed = parse_bbduk_reads_removed(&raw_backend_report)
                .context("parse bbduk polyg backend metrics")?;
            Ok(serde_json::json!({
                "schema_version": "bijux.bbduk.trim_polyg.metrics.v1",
                "reads_removed": reads_removed,
            }))
        }
        _ => Err(anyhow!("unsupported trim_polyg_tails raw report format {raw_report_format}")),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{
        admitted_stage_tools, benchmark_query_context, load_governed_trim_polyg_report,
        normalized_polyg_backend_metrics, raw_polyg_report_artifact, resolve_requested_tools,
        write_governed_trim_polyg_report,
    };
    use bijux_dna_domain_fastq::params::PairedMode;
    use bijux_dna_domain_fastq::{TrimPolygReportV1, TRIM_POLYG_REPORT_SCHEMA_VERSION};
    use std::path::Path;

    #[test]
    fn resolve_requested_tools_uses_execution_support_for_auto_and_all() {
        let expected = admitted_stage_tools();
        assert_eq!(resolve_requested_tools(&[]), expected);
        assert_eq!(resolve_requested_tools(&["auto".to_string()]), expected);
        assert_eq!(resolve_requested_tools(&["all".to_string()]), expected);
    }

    #[test]
    fn benchmark_query_context_keeps_governed_polyg_bank_hash() {
        let polyx_context = serde_json::json!({"bank_hash": "polyx-hash"});
        let context = benchmark_query_context(Some(&polyx_context)).expect("query context");

        assert!(context.stage_contract_hash.is_some());
        assert_eq!(context.bank_hashes.get("polyx_bank").map(String::as_str), Some("polyx-hash"));
    }

    #[test]
    fn raw_polyg_report_artifact_uses_backend_specific_native_outputs() {
        let out_dir = Path::new("out");

        let (fastp_path, fastp_format) =
            raw_polyg_report_artifact("fastp", out_dir).expect("fastp raw report");
        assert_eq!(fastp_path, Path::new("out").join("trim_polyg_tails_report.fastp.json"));
        assert_eq!(fastp_format, "fastp_json");

        let (bbduk_path, bbduk_format) =
            raw_polyg_report_artifact("bbduk", out_dir).expect("bbduk raw report");
        assert_eq!(bbduk_path, Path::new("out").join("trim_polyg_tails_report.stats.txt"));
        assert_eq!(bbduk_format, "bbduk_stats");
    }

    #[test]
    fn normalized_polyg_backend_metrics_parses_fastp_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        let raw_report_path = temp.path().join("trim_polyg.fastp.json");
        bijux_dna_infra::write_bytes(
            &raw_report_path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": 960_u64,
                    "low_quality_reads": 18_u64,
                    "too_many_N_reads": 4_u64,
                    "too_short_reads": 12_u64
                }
            })
            .to_string(),
        )
        .expect("write fastp report");

        let metrics =
            normalized_polyg_backend_metrics(&raw_report_path, "fastp_json").expect("metrics");

        assert_eq!(metrics["passed_filter_reads"], serde_json::json!(960_u64));
        assert_eq!(metrics["too_short_reads"], serde_json::json!(12_u64));
    }

    #[test]
    fn normalized_polyg_backend_metrics_parses_bbduk_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        let raw_report_path = temp.path().join("trim_polyg.stats.txt");
        bijux_dna_infra::write_bytes(&raw_report_path, "Reads Removed: 137\n")
            .expect("write bbduk report");

        let metrics =
            normalized_polyg_backend_metrics(&raw_report_path, "bbduk_stats").expect("metrics");

        assert_eq!(metrics["reads_removed"], serde_json::json!(137_u64));
    }

    #[test]
    fn governed_trim_polyg_report_round_trips_with_backend_metrics() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_polyg_tails_report.json");
        let report = TrimPolygReportV1 {
            schema_version: TRIM_POLYG_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_polyg_tails".to_string(),
            stage_id: "fastq.trim_polyg_tails".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 6,
            trim_polyg: true,
            min_polyg_run: 12,
            input_r1: "reads.fastq.gz".to_string(),
            input_r2: None,
            output_r1: "trimmed.fastq.gz".to_string(),
            output_r2: None,
            reads_in: Some(100),
            reads_out: Some(97),
            bases_in: Some(1000),
            bases_out: Some(910),
            pairs_in: None,
            pairs_out: None,
            mean_q_before: Some(28.0),
            mean_q_after: Some(29.0),
            bases_trimmed_polyg: Some(90),
            polyx_bank_id: Some("polyx".to_string()),
            polyx_bank_hash: Some("sha256:polyx".to_string()),
            polyx_preset: Some("illumina_twocolor".to_string()),
            runtime_s: Some(3.5),
            memory_mb: Some(42.0),
            raw_backend_report: Some("trim.fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
            backend_metrics: Some(serde_json::json!({
                "schema_version": "bijux.fastp.metrics.v1",
                "passed_filter_reads": 97_u64,
            })),
        };

        write_governed_trim_polyg_report(&report_path, &report).expect("write report");
        let decoded = load_governed_trim_polyg_report(&report_path).expect("load report");

        assert_eq!(decoded.threads, 6);
        assert_eq!(decoded.min_polyg_run, 12);
        assert_eq!(decoded.bases_trimmed_polyg, Some(90));
        assert_eq!(decoded.polyx_preset.as_deref(), Some("illumina_twocolor"));
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastp_json"));
        assert_eq!(
            decoded
                .backend_metrics
                .as_ref()
                .and_then(|metrics| metrics.get("passed_filter_reads"))
                .and_then(serde_json::Value::as_u64),
            Some(97)
        );
    }
}
