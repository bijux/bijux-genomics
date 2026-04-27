use std::collections::HashMap;
use std::path::Path;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::{fetch_fastq_screen_v1, insert_fastq_screen_v1};
use bijux_dna_analyze::{append_jsonl, metric_set, BenchmarkRecord, FastqScreenMetrics};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::{
    ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_screen_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::screen_taxonomy::{
    plan_screen_with_options, ScreenPlanOptions,
};
use bijux_dna_planner_fastq::stage_api::observer::{
    input_fastq_stats, parse_screen_summary_tsv, parse_seqkit_stats,
};
use bijux_dna_planner_fastq::stage_api::{
    inspect_headers, log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_SCREEN_TAXONOMY,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

use super::trim_bench_common::{benchmark_image_identity, build_benchmark_context};

/// # Errors
/// Returns an error if planning or execution fails.
#[allow(clippy::too_many_lines)]
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqScreenArgs,
) -> Result<BenchOutcome<FastqScreenMetrics>> {
    let tools = select_screen_benchmark_tools(args)?;
    let bench_inputs = prepare_screen_bench(catalog, platform, runner_override, args, &tools)?;

    if args.explain {
        write_screen_benchmark_explain(&bench_inputs)?;
    }

    ensure_image_qa_passed(STAGE_SCREEN_TAXONOMY.as_str(), &bench_inputs.tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_SCREEN_TAXONOMY.as_str(), &bench_inputs.tools, platform, catalog)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqScreenMetrics>>::new();
    for tool in &bench_inputs.tools {
        let out_dir = bench_inputs.tools_root.join(&tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec(
            STAGE_SCREEN_TAXONOMY.as_str(),
            &tool,
            &bench_inputs.registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_screen_with_options(
            &tool_spec,
            &bench_inputs.r1,
            bench_inputs.r2.as_deref(),
            &out_dir,
            &ScreenPlanOptions { database_root: args.database_root.clone(), threads: args.threads },
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = benchmark_image_identity(&tool_spec);
        if let Ok(Some(record)) = fetch_fastq_screen_v1(
            &conn,
            &tool,
            &tool_spec.tool_version,
            &image_digest,
            &bench_inputs.runner.to_string(),
            &platform.name,
            &bench_inputs.input_hash,
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
                stage: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool `{tool}` failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
            continue;
        }
        let record = build_screen_record(
            platform,
            &bench_inputs,
            &tool,
            &tool_spec,
            &plan,
            &out_dir,
            &execution,
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_screen_v1(&conn, &record).context("insert bench sqlite")?;
        records.push(record);
    }

    Ok(BenchOutcome { records, failures, bench_dir: bench_inputs.bench_dir, explain: args.explain })
}

fn select_screen_benchmark_tools(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqScreenArgs,
) -> Result<Vec<String>> {
    let tools = select_screen_tools(&args.tools)?;
    let artifact_kind =
        if args.r2.is_some() { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    preflight_stage(STAGE_SCREEN_TAXONOMY.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_SCREEN_TAXONOMY.as_str(), &header);
    Ok(tools)
}

fn write_screen_benchmark_explain(bench_inputs: &ScreenBenchInputs) -> Result<()> {
    write_explain_md(
        &bench_inputs.bench_dir,
        STAGE_SCREEN_TAXONOMY.as_str(),
        &bench_inputs.tools,
        &bench_inputs.excluded_tools,
        None,
    )?;
    write_explain_plan_json(
        &bench_inputs.bench_dir,
        STAGE_SCREEN_TAXONOMY.as_str(),
        &bench_inputs.tools,
        &bench_inputs.registry,
        None,
    )
}

#[derive(Debug, Clone)]
struct ScreenBenchInputs {
    registry: ToolRegistry,
    tools: Vec<String>,
    excluded_tools: Vec<String>,
    runner: RuntimeKind,
    r1: std::path::PathBuf,
    r2: Option<std::path::PathBuf>,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: std::path::PathBuf,
    tools_root: std::path::PathBuf,
}

fn prepare_screen_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqScreenArgs,
    selected_tools: &[String],
) -> Result<ScreenBenchInputs> {
    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools =
        filter_tools_by_role(STAGE_SCREEN_TAXONOMY.as_str(), selected_tools, &registry, false)?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_SCREEN_TAXONOMY.as_str());
    let all_tools: Vec<String> =
        registry.tools_for_stage(&stage_id).iter().map(|tool| tool.tool_id.to_string()).collect();
    let excluded_tools: Vec<String> =
        all_tools.into_iter().filter(|tool| !tools.contains(tool)).collect();
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_SCREEN_TAXONOMY)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_SCREEN_TAXONOMY.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1.parent().ok_or_else(|| anyhow!("r1 has no parent"))?.to_path_buf();

    let seqkit_tool = catalog
        .get(bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT)
        .ok_or_else(|| anyhow!("seqkit missing from images catalog"))?;
    let seqkit_image = resolve_image_for_run(seqkit_tool, platform)?;
    let stats_spec = input_fastq_stats(&r1_dir, &r1)?;
    let stats_output = execute_observer_command(
        &seqkit_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!("seqkit screen observer failed: {}", stats_output.stderr));
    }

    let (r2, input_stats_r2) = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2.parent().ok_or_else(|| anyhow!("r2 has no parent"))?.to_path_buf();
        let stats_spec = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output = execute_observer_command(
            &seqkit_image.full_name,
            stats_spec.mount_dir.as_path(),
            &stats_spec.args,
            runner,
        )?;
        if stats_output.exit_code != 0 {
            return Err(anyhow!("seqkit screen observer failed for r2: {}", stats_output.stderr));
        }
        (Some(r2), Some(parse_seqkit_stats(&stats_output.stdout)?))
    } else {
        (None, None)
    };

    let input_hash = if let Some(r2) = r2.as_ref() {
        format!(
            "{}+{}",
            hash_file_sha256(&args.r1).context("hash screen input r1")?,
            hash_file_sha256(r2).context("hash screen input r2")?
        )
    } else {
        hash_file_sha256(&args.r1).context("hash screen input")?
    };

    Ok(ScreenBenchInputs {
        registry,
        tools,
        excluded_tools,
        runner,
        r1,
        r2,
        input_hash,
        input_stats: parse_seqkit_stats(&stats_output.stdout)?,
        input_stats_r2,
        bench_dir,
        tools_root,
    })
}

#[allow(clippy::too_many_lines)]
fn build_screen_record(
    platform: &PlatformSpec,
    bench_inputs: &ScreenBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    out_dir: &Path,
    execution: &StageResultV1,
) -> Result<BenchmarkRecord<FastqScreenMetrics>> {
    let effective_params: ScreenEffectiveParams =
        serde_json::from_value(plan.effective_params.clone())
            .context("decode screen taxonomy effective params")?;
    let report_path = plan
        .params
        .get("report")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| out_dir.join("screen_report.tsv"), std::path::PathBuf::from);
    let classification_report_path = plan
        .params
        .get("assignments")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| out_dir.join("classification_report.json"), std::path::PathBuf::from);
    let reads_in = bench_inputs.input_stats.reads
        + bench_inputs.input_stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = bench_inputs.input_stats.bases
        + bench_inputs.input_stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs = bench_inputs
        .input_stats_r2
        .as_ref()
        .map_or(0, |stats| bench_inputs.input_stats.reads.min(stats.reads));
    let summary_entries = load_screen_summary_entries(&report_path)?;
    let unclassified_fraction = find_unclassified_fraction(&summary_entries);
    let classified_fraction = unclassified_fraction.map(|value| (1.0 - value).max(0.0));
    let contamination_rate = classified_fraction.unwrap_or(0.0);
    let top_taxa = top_taxa_entries(&summary_entries, 5);
    let governed_report = ScreenTaxonomyReportV1 {
        schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        stage_id: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        tool_id: tool.to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        classifier: effective_params.classifier.clone(),
        report_format: effective_params.report_format.clone(),
        assignment_format: effective_params.assignment_format.clone(),
        database_catalog_id: effective_params.database_catalog_id.clone(),
        database_artifact_id: effective_params.database_artifact_id.clone(),
        database_build_id: effective_params.database_build_id.clone(),
        database_digest: effective_params.database_digest.clone(),
        database_namespace: effective_params.database_namespace.clone(),
        database_scope: effective_params.database_scope.clone(),
        minimum_confidence: effective_params.minimum_confidence,
        emit_unclassified: effective_params.emit_unclassified,
        input_r1: bench_inputs.r1.display().to_string(),
        input_r2: bench_inputs.r2.as_ref().map(|path| path.display().to_string()),
        screen_report_tsv: report_path.display().to_string(),
        classification_report_json: classification_report_path.display().to_string(),
        reads_in: Some(reads_in),
        reads_out: Some(reads_in),
        bases_in: Some(bases_in),
        bases_out: Some(bases_in),
        pairs_in: Some(pairs),
        pairs_out: Some(pairs),
        contamination_rate: Some(contamination_rate),
        classified_fraction,
        unclassified_fraction,
        summary_entries: summary_entries.clone(),
        top_taxa: top_taxa.clone(),
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
    };
    bijux_dna_infra::atomic_write_json(&classification_report_path, &governed_report)
        .context("write governed screen taxonomy report")?;
    let metrics = FastqScreenMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: pairs,
        pairs_out: pairs,
        contamination_rate,
        classified_fraction,
        unclassified_fraction,
        classifier: Some(enum_json_name(&effective_params.classifier)?),
        report_format: Some(enum_json_name(&effective_params.report_format)?),
        database_catalog_id: Some(effective_params.database_catalog_id.clone()),
        database_artifact_id: Some(effective_params.database_artifact_id.clone()),
        minimum_confidence: effective_params.minimum_confidence.map(f64::from),
        emit_unclassified: Some(effective_params.emit_unclassified),
        contamination_summary: serde_json::to_value(&summary_entries)
            .context("serialize taxonomy summary entries")?
            .into(),
        top_taxa: serde_json::to_value(&top_taxa).context("serialize top taxonomy entries")?.into(),
    };
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write screen taxonomy metrics")?;

    let context = build_benchmark_context(
        tool,
        tool_spec.tool_version.clone(),
        benchmark_image_identity(tool_spec),
        bench_inputs.runner,
        platform,
        bench_inputs.input_hash.clone(),
        plan.params.clone(),
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
    Ok(record)
}

fn load_screen_summary_entries(path: &Path) -> Result<Vec<TaxonomyScreenSummaryEntryV1>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("screen report missing: {}", path.display()))?;
    parse_screen_summary_tsv(&raw)
}

fn find_unclassified_fraction(entries: &[TaxonomyScreenSummaryEntryV1]) -> Option<f64> {
    entries.iter().find_map(|entry| {
        let label = entry.label.to_ascii_lowercase();
        if label.contains("unclassified") || label.contains("unmapped") || label.contains("no hit")
        {
            Some((entry.percent / 100.0).clamp(0.0, 1.0))
        } else {
            None
        }
    })
}

fn top_taxa_entries(
    entries: &[TaxonomyScreenSummaryEntryV1],
    max_entries: usize,
) -> Vec<TaxonomyScreenSummaryEntryV1> {
    let mut top_taxa = entries
        .iter()
        .filter(|entry| {
            let label = entry.label.to_ascii_lowercase();
            !label.contains("unclassified")
                && !label.contains("unmapped")
                && !label.contains("no hit")
        })
        .cloned()
        .collect::<Vec<_>>();
    top_taxa.sort_by(|left, right| right.percent.total_cmp(&left.percent));
    top_taxa.truncate(max_entries);
    top_taxa
}

fn enum_json_name<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_value(value)?
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("screen taxonomy enum must serialize to a string"))
}

#[cfg(test)]
mod tests {
    use super::{find_unclassified_fraction, top_taxa_entries};
    use bijux_dna_domain_fastq::TaxonomyScreenSummaryEntryV1;

    #[test]
    fn unclassified_fraction_is_derived_from_summary_entries() {
        let entries = vec![
            TaxonomyScreenSummaryEntryV1 { label: "unclassified".to_string(), percent: 17.5 },
            TaxonomyScreenSummaryEntryV1 { label: "bacteria".to_string(), percent: 82.5 },
        ];

        assert_eq!(find_unclassified_fraction(&entries), Some(0.175));
    }

    #[test]
    fn top_taxa_entries_exclude_unclassified_labels() {
        let entries = vec![
            TaxonomyScreenSummaryEntryV1 { label: "unclassified".to_string(), percent: 40.0 },
            TaxonomyScreenSummaryEntryV1 { label: "viruses".to_string(), percent: 35.0 },
            TaxonomyScreenSummaryEntryV1 { label: "bacteria".to_string(), percent: 25.0 },
        ];

        let top_taxa = top_taxa_entries(&entries, 2);
        assert_eq!(top_taxa.len(), 2);
        assert_eq!(top_taxa[0].label, "viruses");
        assert_eq!(top_taxa[1].label, "bacteria");
    }
}
