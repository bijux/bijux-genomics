use std::collections::HashMap;
use std::path::Path;

use crate::internal::fastq::stages::record_identity::stable_params_hash;
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
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::PairedMode;
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
use bijux_dna_stage_contract::StagePlanV1;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_SCREEN_TAXONOMY,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;

use super::trim_bench_common::{benchmark_image_identity, build_benchmark_context};

/// # Errors
/// Returns an error if planning or execution fails.
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

    ensure_screen_benchmark_qa(catalog, platform, &bench_inputs.tools)?;

    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");
    let jobs = bench_jobs(args.jobs);
    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqScreenMetrics>>::new();
    for tool in &bench_inputs.tools {
        let tool_plan =
            prepare_screen_tool_plan(catalog, platform, args, &bench_inputs, jobs, tool)?;
        if let Ok(Some(record)) = fetch_fastq_screen_v1(
            &conn,
            &tool_plan.tool,
            &tool_plan.tool_spec.tool_version,
            &tool_plan.image_digest,
            &bench_inputs.runner.to_string(),
            &platform.name,
            &bench_inputs.input_hash,
            &tool_plan.params_hash,
        ) {
            records.push(record);
            continue;
        }
        let execution = execute_screen_tool(&tool_plan, bench_inputs.runner, jobs)?;
        if let Some(failure) = screen_tool_failure(&tool_plan, execution.exit_code) {
            failures.push(failure);
            continue;
        }
        let record = build_screen_record(&ScreenRecordInputs {
            platform,
            bench_inputs: &bench_inputs,
            tool: &tool_plan.tool,
            tool_spec: &tool_plan.tool_spec,
            plan: &tool_plan.plan,
            out_dir: &tool_plan.out_dir,
            execution: &execution,
        })?;
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

fn ensure_screen_benchmark_qa<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    tools: &[String],
) -> Result<()> {
    ensure_image_qa_passed(STAGE_SCREEN_TAXONOMY.as_str(), tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_SCREEN_TAXONOMY.as_str(), tools, platform, catalog)
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

struct ScreenToolPlan {
    tool: String,
    tool_spec: ToolExecutionSpecV1,
    plan: StagePlanV1,
    out_dir: std::path::PathBuf,
    params_hash: String,
    image_digest: String,
}

struct ScreenRecordInputs<'a> {
    platform: &'a PlatformSpec,
    bench_inputs: &'a ScreenBenchInputs,
    tool: &'a str,
    tool_spec: &'a ToolExecutionSpecV1,
    plan: &'a StagePlanV1,
    out_dir: &'a Path,
    execution: &'a StageResultV1,
}

struct ScreenReportPaths {
    summary_tsv: std::path::PathBuf,
    classification_json: std::path::PathBuf,
}

struct ScreenReadAccounting {
    reads_in: u64,
    bases_in: u64,
    pairs: u64,
}

struct ScreenClassificationSummary {
    entries: Vec<TaxonomyScreenSummaryEntryV1>,
    classified_fraction: Option<f64>,
    unclassified_fraction: Option<f64>,
    contamination_rate: f64,
    top_taxa: Vec<TaxonomyScreenSummaryEntryV1>,
}

fn prepare_screen_tool_plan<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqScreenArgs,
    bench_inputs: &ScreenBenchInputs,
    jobs: usize,
    tool: &str,
) -> Result<ScreenToolPlan> {
    let out_dir = bench_inputs.tools_root.join(tool);
    bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
    let tool_spec = build_tool_execution_spec(
        STAGE_SCREEN_TAXONOMY.as_str(),
        tool,
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
    let params_hash = stable_params_hash(&plan.params);
    let image_digest = benchmark_image_identity(&tool_spec);
    Ok(ScreenToolPlan {
        tool: tool.to_string(),
        tool_spec,
        plan,
        out_dir,
        params_hash,
        image_digest,
    })
}

fn execute_screen_tool(
    tool_plan: &ScreenToolPlan,
    runner: RuntimeKind,
    jobs: usize,
) -> Result<StageResultV1> {
    execute_plans_with_jobs(
        vec![bijux_dna_stage_contract::execution_step_from_stage_plan(&tool_plan.plan)],
        runner,
        jobs,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("missing execution result for {}", tool_plan.tool))
}

fn screen_tool_failure(tool_plan: &ScreenToolPlan, exit_code: i32) -> Option<RawFailure> {
    if exit_code == 0 {
        return None;
    }
    Some(RawFailure {
        stage: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        tool: tool_plan.tool.clone(),
        reason: format!("tool `{}` failed with status {exit_code}", tool_plan.tool),
        category: ErrorCategory::ToolError,
    })
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

    let input_hash = screen_input_hash(&r1, r2.as_deref())?;

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

fn screen_input_hash(r1: &Path, r2: Option<&Path>) -> Result<String> {
    let r1_hash = hash_file_sha256(r1).context("hash screen input r1")?;
    if let Some(r2) = r2 {
        let r2_hash = hash_file_sha256(r2).context("hash screen input r2")?;
        return params_hash(&serde_json::json!({
            "r1": r1_hash,
            "r2": r2_hash,
        }))
        .context("combine screen paired input hashes");
    }
    Ok(r1_hash)
}

fn build_screen_record(
    inputs: &ScreenRecordInputs<'_>,
) -> Result<BenchmarkRecord<FastqScreenMetrics>> {
    let effective_params: ScreenEffectiveParams =
        serde_json::from_value(inputs.plan.effective_params.clone())
            .context("decode screen taxonomy effective params")?;
    let report_paths = screen_report_paths(inputs.plan)?;
    let read_accounting = screen_read_accounting(inputs.bench_inputs);
    let classification_summary =
        screen_classification_summary(load_screen_summary_entries(&report_paths.summary_tsv)?);
    let governed_report = ScreenTaxonomyReportV1 {
        schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        stage_id: STAGE_SCREEN_TAXONOMY.as_str().to_string(),
        tool_id: inputs.tool.to_string(),
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
        input_r1: inputs.bench_inputs.r1.display().to_string(),
        input_r2: inputs.bench_inputs.r2.as_ref().map(|path| path.display().to_string()),
        screen_report_tsv: report_paths.summary_tsv.display().to_string(),
        classification_report_json: report_paths.classification_json.display().to_string(),
        reads_in: Some(read_accounting.reads_in),
        reads_out: Some(read_accounting.reads_in),
        bases_in: Some(read_accounting.bases_in),
        bases_out: Some(read_accounting.bases_in),
        pairs_in: Some(read_accounting.pairs),
        pairs_out: Some(read_accounting.pairs),
        contamination_rate: Some(classification_summary.contamination_rate),
        classified_fraction: classification_summary.classified_fraction,
        unclassified_fraction: classification_summary.unclassified_fraction,
        summary_entries: classification_summary.entries.clone(),
        top_taxa: classification_summary.top_taxa.clone(),
        runtime_s: Some(inputs.execution.runtime_s),
        memory_mb: Some(inputs.execution.memory_mb),
    };
    validate_screen_report_identity(inputs.tool, &governed_report)?;
    validate_screen_report_execution(&governed_report, inputs.execution)?;
    validate_screen_report_paired_mode(inputs.bench_inputs.r2.is_some(), &governed_report)?;
    bijux_dna_infra::atomic_write_json(&report_paths.classification_json, &governed_report)
        .context("write governed screen taxonomy report")?;
    let metrics =
        screen_metrics_from_summary(&read_accounting, &classification_summary, &effective_params)?;
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&inputs.out_dir.join("metrics.json"), &metrics_json)
        .context("write screen taxonomy metrics")?;

    let context = build_benchmark_context(
        inputs.tool,
        inputs.tool_spec.tool_version.clone(),
        benchmark_image_identity(inputs.tool_spec),
        inputs.bench_inputs.runner,
        inputs.platform,
        inputs.bench_inputs.input_hash.clone(),
        inputs.plan.params.clone(),
    );
    let record = BenchmarkRecord {
        context,
        execution: ExecutionMetrics {
            runtime_s: inputs.execution.runtime_s,
            memory_mb: inputs.execution.memory_mb,
            exit_code: inputs.execution.exit_code,
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

fn screen_report_paths(plan: &StagePlanV1) -> Result<ScreenReportPaths> {
    Ok(ScreenReportPaths {
        summary_tsv: required_screen_output_path(plan, "screen_report_tsv")?,
        classification_json: required_screen_output_path(plan, "classification_report_json")?,
    })
}

fn artifact_output_path(plan: &StagePlanV1, artifact_id: &str) -> Option<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == artifact_id)
        .map(|artifact| artifact.path.clone())
}

fn required_screen_output_path(
    plan: &StagePlanV1,
    artifact_id: &str,
) -> Result<std::path::PathBuf> {
    artifact_output_path(plan, artifact_id)
        .ok_or_else(|| anyhow!("screen taxonomy plan missing output artifact {artifact_id}"))
}

fn screen_read_accounting(bench_inputs: &ScreenBenchInputs) -> ScreenReadAccounting {
    let reads_in = bench_inputs.input_stats.reads
        + bench_inputs.input_stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = bench_inputs.input_stats.bases
        + bench_inputs.input_stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs = bench_inputs
        .input_stats_r2
        .as_ref()
        .map_or(0, |stats| bench_inputs.input_stats.reads.min(stats.reads));
    ScreenReadAccounting { reads_in, bases_in, pairs }
}

fn screen_classification_summary(
    entries: Vec<TaxonomyScreenSummaryEntryV1>,
) -> ScreenClassificationSummary {
    let unclassified_fraction = find_unclassified_fraction(&entries);
    let classified_fraction = unclassified_fraction.map(|value| (1.0 - value).max(0.0));
    let contamination_rate = classified_fraction.unwrap_or(0.0);
    let top_taxa = top_taxa_entries(&entries, 5);
    ScreenClassificationSummary {
        entries,
        classified_fraction,
        unclassified_fraction,
        contamination_rate,
        top_taxa,
    }
}

fn screen_metrics_from_summary(
    read_accounting: &ScreenReadAccounting,
    classification_summary: &ScreenClassificationSummary,
    effective_params: &ScreenEffectiveParams,
) -> Result<FastqScreenMetrics> {
    Ok(FastqScreenMetrics {
        reads_in: read_accounting.reads_in,
        reads_out: read_accounting.reads_in,
        bases_in: read_accounting.bases_in,
        bases_out: read_accounting.bases_in,
        pairs_in: read_accounting.pairs,
        pairs_out: read_accounting.pairs,
        contamination_rate: classification_summary.contamination_rate,
        classified_fraction: classification_summary.classified_fraction,
        unclassified_fraction: classification_summary.unclassified_fraction,
        classifier: Some(enum_json_name(&effective_params.classifier)?),
        report_format: Some(enum_json_name(&effective_params.report_format)?),
        database_catalog_id: Some(effective_params.database_catalog_id.clone()),
        database_artifact_id: Some(effective_params.database_artifact_id.clone()),
        minimum_confidence: effective_params.minimum_confidence.map(f64::from),
        emit_unclassified: Some(effective_params.emit_unclassified),
        contamination_summary: serde_json::to_value(&classification_summary.entries)
            .context("serialize taxonomy summary entries")?
            .into(),
        top_taxa: serde_json::to_value(&classification_summary.top_taxa)
            .context("serialize top taxonomy entries")?
            .into(),
    })
}

fn validate_screen_report_identity(tool: &str, report: &ScreenTaxonomyReportV1) -> Result<()> {
    if report.schema_version != SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "screen taxonomy report schema mismatch: expected {}, observed {}",
            SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
            report.schema_version
        ));
    }
    if report.stage != STAGE_SCREEN_TAXONOMY.as_str()
        || report.stage_id != STAGE_SCREEN_TAXONOMY.as_str()
    {
        return Err(anyhow!(
            "screen taxonomy report stage mismatch: observed stage={} stage_id={}",
            report.stage,
            report.stage_id
        ));
    }
    if report.tool_id != tool {
        return Err(anyhow!(
            "screen taxonomy report tool mismatch: expected {}, observed {}",
            tool,
            report.tool_id
        ));
    }
    Ok(())
}

fn validate_screen_report_execution(
    report: &ScreenTaxonomyReportV1,
    execution: &StageResultV1,
) -> Result<()> {
    if report.runtime_s.is_none_or(|observed| (observed - execution.runtime_s).abs() > f64::EPSILON)
    {
        return Err(anyhow!(
            "screen taxonomy report runtime mismatch: expected {}, observed {:?}",
            execution.runtime_s,
            report.runtime_s
        ));
    }
    if report.memory_mb.is_none_or(|observed| (observed - execution.memory_mb).abs() > f64::EPSILON)
    {
        return Err(anyhow!(
            "screen taxonomy report memory mismatch: expected {}, observed {:?}",
            execution.memory_mb,
            report.memory_mb
        ));
    }
    Ok(())
}

fn validate_screen_report_paired_mode(has_r2: bool, report: &ScreenTaxonomyReportV1) -> Result<()> {
    let expected = if has_r2 { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    if report.paired_mode != expected {
        return Err(anyhow!(
            "screen taxonomy report paired mode mismatch: expected {:?}, observed {:?}",
            expected,
            report.paired_mode
        ));
    }
    Ok(())
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
