use bijux_dna_core::prelude::ContainerImageRefV1;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench::fetch_fastq_qc_post_v1;
use bijux_dna_analyze::quality::insert_fastq_qc_post_v1;
use bijux_dna_analyze::{
    append_jsonl, metric_set, BenchmarkContext, BenchmarkRecord, FastqQcPostMetrics,
};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::measure::{ExecutionMetrics, SeqkitMetrics};
use bijux_dna_core::prelude::params_hash;
use bijux_dna_core::prelude::{ArtifactId, ArtifactRef};
use bijux_dna_domain_fastq::params::{qc_post::QcAggregationScope, PairedMode};
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::select_qc_post_tools;
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::fastq::report_qc::{
    aux_tool_ids, plan_qc_post_with_qc_inputs,
};
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_planner_fastq::stage_api::{
    governed_qc_producer_stage_ids, inspect_headers, log_header_warnings, preflight_stage,
    FastqArtifactKind, RawFailure,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::{execute_observer_command, StageResultV1};
use uuid::Uuid;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::jobs::execute_plans_with_jobs;
use crate::internal::handlers::fastq::{
    write_explain_md, write_explain_plan_json, BenchOutcome, STAGE_REPORT_QC,
};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_stage_contract::StagePlanV1;

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_qc_post<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_dna_analyze::FastqQcPostMetrics>> {
    let tools = select_qc_post_tools(&args.tools)?;
    let artifact_kind = if args.r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    preflight_stage(STAGE_REPORT_QC.as_str(), artifact_kind)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings(STAGE_REPORT_QC.as_str(), &header);

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role(STAGE_REPORT_QC.as_str(), &tools, &registry, false)?;
    let bench_inputs = prepare_qc_post_bench(catalog, platform, runner_override, args)?;
    let stage_id = bijux_dna_core::ids::StageId::new(STAGE_REPORT_QC.as_str());
    let all_tools: Vec<String> = registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| tool.tool_id.to_string())
        .collect();
    let excluded: Vec<String> = all_tools
        .into_iter()
        .filter(|tool| !tools.contains(tool))
        .collect();

    if args.explain {
        write_explain_md(
            &bench_inputs.bench_dir,
            STAGE_REPORT_QC.as_str(),
            &tools,
            &excluded,
            None,
        )?;
        write_explain_plan_json(
            &bench_inputs.bench_dir,
            STAGE_REPORT_QC.as_str(),
            &tools,
            &registry,
            None,
        )?;
    }

    ensure_image_qa_passed(STAGE_REPORT_QC.as_str(), &tools, platform, catalog)?;
    ensure_tool_qa_passed(STAGE_REPORT_QC.as_str(), &tools, platform, catalog)?;
    let sqlite_path = bench_inputs.bench_dir.join("bench.sqlite");
    let conn = bijux_dna_analyze::open_sqlite(&sqlite_path).context("open bench sqlite")?;
    let bench_path = bench_inputs.bench_dir.join("bench.jsonl");

    let jobs = bench_jobs(args.jobs);
    let mut aux_tools = std::collections::BTreeMap::new();
    for aux_tool in aux_tool_ids() {
        let spec = catalog
            .get(aux_tool.as_str())
            .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.toml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        aux_tools.insert(
            aux_tool,
            ContainerImageRefV1 {
                image: image.full_name,
                digest: spec.digest.clone(),
            },
        );
    }

    let mut failures = Vec::<RawFailure>::new();
    let mut records = Vec::<BenchmarkRecord<FastqQcPostMetrics>>::new();
    for tool in &tools {
        let out_dir = bench_inputs.tools_root.join(tool);
        bijux_dna_infra::ensure_dir(&out_dir).context("create tool output dir")?;
        let governed_qc = prepare_governed_qc_inputs(
            catalog,
            platform,
            &registry,
            bench_inputs.runner,
            jobs,
            &bench_inputs,
            &out_dir,
        )?;
        let tool_spec = build_tool_execution_spec(
            STAGE_REPORT_QC.as_str(),
            tool,
            &registry,
            catalog,
            platform,
        )?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let plan = plan_qc_post_with_qc_inputs(
            &tool_spec,
            &governed_qc.qc_inputs,
            &out_dir,
            aux_tools.clone(),
            paired_mode_for_bench_inputs(&bench_inputs),
            QcAggregationScope::GovernedQcArtifacts,
            Some(&bench_inputs.r1),
            bench_inputs.r2.as_deref(),
        )?;
        let params_hash = params_hash(&plan.params).unwrap_or_else(|_| Uuid::new_v4().to_string());
        let image_digest = tool_spec
            .image
            .digest
            .as_ref()
            .ok_or_else(|| anyhow!("image digest missing for tool {tool}"))?
            .clone();
        if let Ok(Some(record)) = fetch_fastq_qc_post_v1(
            &conn,
            tool,
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
            vec![bijux_dna_stage_contract::execution_step_from_stage_plan(
                &plan,
            )],
            bench_inputs.runner,
            jobs,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing execution result for {tool}"))?;
        let record = build_qc_post_record(
            platform,
            &bench_inputs,
            tool,
            &tool_spec,
            &plan.params,
            &out_dir,
            &execution,
            governed_qc.raw_fastqc_dir.as_deref(),
        )?;
        append_jsonl(&bench_path, &record).context("write bench.jsonl")?;
        insert_fastq_qc_post_v1(&conn, &record).context("insert bench sqlite")?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: STAGE_REPORT_QC.as_str().to_string(),
                tool: tool.clone(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
                category: ErrorCategory::ToolError,
            });
        }
        records.push(record);
    }

    Ok(BenchOutcome {
        records,
        failures,
        bench_dir: bench_inputs.bench_dir,
        explain: args.explain,
    })
}

#[derive(Debug, Clone)]
struct QcPostBenchInputs {
    runner: RuntimeKind,
    r1: PathBuf,
    r2: Option<PathBuf>,
    input_hash: String,
    input_stats: SeqkitMetrics,
    input_stats_r2: Option<SeqkitMetrics>,
    bench_dir: PathBuf,
    tools_root: PathBuf,
}

#[derive(Debug, Clone)]
struct GovernedQcInputs {
    qc_inputs: Vec<ArtifactRef>,
    raw_fastqc_dir: Option<PathBuf>,
}

fn prepare_qc_post_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqQcPostArgs,
) -> Result<QcPostBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(&STAGE_REPORT_QC)
        .ok_or_else(|| anyhow!("bench dir missing for {}", STAGE_REPORT_QC.as_str()))?;
    let bench_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = args.r1.canonicalize().context("resolve r1 path")?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();

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
        return Err(anyhow!(
            "seqkit qc observer failed: {}",
            stats_output.stderr
        ));
    }

    let (r2, input_stats_r2) = if let Some(r2) = args.r2.as_deref() {
        let r2 = r2.canonicalize().context("resolve r2 path")?;
        let r2_dir = r2
            .parent()
            .ok_or_else(|| anyhow!("r2 has no parent"))?
            .to_path_buf();
        let stats_spec = input_fastq_stats(&r2_dir, &r2)?;
        let stats_output = execute_observer_command(
            &seqkit_image.full_name,
            stats_spec.mount_dir.as_path(),
            &stats_spec.args,
            runner,
        )?;
        if stats_output.exit_code != 0 {
            return Err(anyhow!(
                "seqkit qc observer failed for r2: {}",
                stats_output.stderr
            ));
        }
        (Some(r2), Some(parse_seqkit_stats(&stats_output.stdout)?))
    } else {
        (None, None)
    };

    Ok(QcPostBenchInputs {
        runner,
        input_hash: if let Some(r2) = r2.as_ref() {
            format!(
                "{}+{}",
                hash_file_sha256(&r1).context("hash qc input r1")?,
                hash_file_sha256(r2).context("hash qc input r2")?
            )
        } else {
            hash_file_sha256(&r1).context("hash qc input")?
        },
        input_stats: parse_seqkit_stats(&stats_output.stdout)?,
        input_stats_r2,
        r1,
        r2,
        bench_dir,
        tools_root,
    })
}

fn build_qc_post_record(
    platform: &PlatformSpec,
    bench_inputs: &QcPostBenchInputs,
    tool: &str,
    tool_spec: &bijux_dna_core::prelude::ToolExecutionSpecV1,
    params: &serde_json::Value,
    out_dir: &Path,
    execution: &StageResultV1,
    raw_fastqc_dir: Option<&Path>,
) -> Result<BenchmarkRecord<FastqQcPostMetrics>> {
    let metrics = derive_qc_post_metrics(
        &bench_inputs.input_stats,
        bench_inputs.input_stats_r2.as_ref(),
        out_dir,
        raw_fastqc_dir,
    );
    let metric_set = metric_set(metrics.clone());
    bijux_dna_analyze::validate_metric_set(&metric_set)?;

    let report = serde_json::json!({
        "schema_version": "bijux.fastq.report_qc.report.v1",
        "stage_id": STAGE_REPORT_QC.as_str(),
        "tool_id": tool,
        "input_fastq_r1": bench_inputs.r1,
        "input_fastq_r2": bench_inputs.r2,
        "reads_in": metrics.reads_in,
        "reads_out": metrics.reads_out,
        "bases_in": metrics.bases_in,
        "bases_out": metrics.bases_out,
        "pairs_in": metrics.pairs_in,
        "pairs_out": metrics.pairs_out,
        "mean_q": metrics.mean_q,
        "contamination_rate": metrics.contamination_rate,
        "raw_fastqc_dir": metrics.raw_fastqc_dir,
        "trimmed_fastqc_dir": metrics.trimmed_fastqc_dir,
        "multiqc_report": metrics.multiqc_report,
        "multiqc_data": metrics.multiqc_data,
        "runtime_s": execution.runtime_s,
        "memory_mb": execution.memory_mb,
        "exit_code": execution.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("qc_report.json"), &report)
        .context("write qc report")?;
    let metrics_json = serde_json::to_value(&metric_set)?;
    bijux_dna_infra::atomic_write_json(&out_dir.join("metrics.json"), &metrics_json)
        .context("write qc metrics")?;

    let context = BenchmarkContext {
        tool: tool.to_string(),
        tool_version: tool_spec.tool_version.clone(),
        image_digest: tool_spec
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        runner: bench_inputs.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: bench_inputs.input_hash.clone(),
        parameters: params.clone().into(),
    };
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

fn derive_qc_post_metrics(
    input_stats: &SeqkitMetrics,
    input_stats_r2: Option<&SeqkitMetrics>,
    out_dir: &Path,
    raw_fastqc_dir: Option<&Path>,
) -> FastqQcPostMetrics {
    let multiqc_report = out_dir.join("multiqc_report.html");
    let multiqc_data = out_dir.join("multiqc_data");
    let trimmed_fastqc_dir = out_dir.join("fastqc_trimmed");
    let reads_in = input_stats.reads + input_stats_r2.map_or(0, |stats| stats.reads);
    let bases_in = input_stats.bases + input_stats_r2.map_or(0, |stats| stats.bases);
    let weighted_q_sum = input_stats.mean_q * input_stats.bases as f64
        + input_stats_r2.map_or(0.0, |stats| stats.mean_q * stats.bases as f64);
    let mean_q = if bases_in == 0 {
        0.0
    } else {
        weighted_q_sum / bases_in as f64
    };
    FastqQcPostMetrics {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        pairs_out: input_stats_r2.map(|stats| input_stats.reads.min(stats.reads)),
        mean_q,
        contamination_rate: 0.0,
        raw_fastqc_dir: raw_fastqc_dir.and_then(path_if_exists),
        trimmed_fastqc_dir: path_if_exists(&trimmed_fastqc_dir),
        multiqc_report: path_if_exists(&multiqc_report),
        multiqc_data: path_if_exists(&multiqc_data),
    }
}

fn path_if_exists(path: &Path) -> Option<String> {
    path.exists().then(|| path.display().to_string())
}

fn paired_mode_for_bench_inputs(bench_inputs: &QcPostBenchInputs) -> PairedMode {
    if bench_inputs.r2.is_some() {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}

fn prepare_governed_qc_inputs<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    registry: &bijux_dna_core::contract::ToolRegistry,
    runner: RuntimeKind,
    jobs: usize,
    bench_inputs: &QcPostBenchInputs,
    out_dir: &Path,
) -> Result<GovernedQcInputs> {
    let contributors = governed_qc_producer_stage_ids()
        .into_iter()
        .filter_map(|stage_id| {
            bijux_dna_planner_fastq::stage_api::default_tool_for_stage(&stage_id)
                .map(|tool_id| (stage_id, tool_id))
        })
        .collect::<Vec<_>>();
    let mut plans = Vec::with_capacity(contributors.len());
    for (stage_id, tool_id) in contributors {
        let stage_id_str = stage_id.as_str();
        let tool_id_str = tool_id.as_str();
        let tool_spec =
            build_tool_execution_spec(stage_id_str, tool_id_str, registry, catalog, platform)?;
        let tool_spec = scale_tool_spec_for_jobs(&tool_spec, jobs);
        let stage_out_dir = out_dir
            .join("governed_qc_inputs")
            .join(stage_id_str.trim_start_matches("fastq."))
            .join(tool_id_str);
        bijux_dna_infra::ensure_dir(&stage_out_dir)
            .with_context(|| {
                format!("create governed qc input dir for {stage_id_str}/{tool_id_str}")
            })?;
        let plan = match stage_id_str {
            "fastq.validate_reads" => {
                bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::plan(
                    &tool_spec,
                    &bench_inputs.r1,
                    bench_inputs.r2.as_deref(),
                    &stage_out_dir,
                )?
            }
            "fastq.detect_adapters" => {
                bijux_dna_planner_fastq::tool_adapters::fastq::detect_adapters::plan(
                    &tool_spec,
                    &bench_inputs.r1,
                    bench_inputs.r2.as_deref(),
                    &stage_out_dir,
                )?
            }
            "fastq.profile_reads" => {
                bijux_dna_planner_fastq::tool_adapters::fastq::profile_reads::plan_stats_neutral(
                    &tool_spec,
                    &bench_inputs.r1,
                    bench_inputs.r2.as_deref(),
                    &stage_out_dir,
                )?
            }
            _ => continue,
        };
        plans.push(plan);
    }
    let executions = execute_plans_with_jobs(
        plans
            .iter()
            .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
            .collect(),
        runner,
        jobs,
    )?;
    for (plan, execution) in plans.iter().zip(executions.iter()) {
        if execution.exit_code != 0 {
            return Err(anyhow!(
                "governed QC contributor {} / {} failed with status {}",
                plan.stage_id,
                plan.tool_id,
                execution.exit_code
            ));
        }
    }
    let mut qc_inputs = plans
        .iter()
        .flat_map(governed_qc_artifacts_for_plan)
        .collect::<Vec<_>>();
    qc_inputs.sort_by(|left, right| {
        left.name
            .as_str()
            .cmp(right.name.as_str())
            .then_with(|| left.path.cmp(&right.path))
    });
    qc_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    let raw_fastqc_dir = plans
        .iter()
        .find(|plan| plan.stage_id.as_str() == "fastq.detect_adapters")
        .and_then(|plan| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "adapter_evidence_dir")
                .map(|artifact| artifact.path.clone())
        });
    Ok(GovernedQcInputs {
        qc_inputs,
        raw_fastqc_dir,
    })
}

fn governed_qc_artifacts_for_plan(plan: &StagePlanV1) -> Vec<ArtifactRef> {
    let stage_node_id = plan
        .stage_instance_id
        .as_ref()
        .map(|step_id| step_id.as_str())
        .unwrap_or(plan.stage_id.as_str());
    plan.io
        .outputs
        .iter()
        .filter(|artifact| {
            governed_qc_output_ids_for_stage(plan.stage_id.as_str())
                .iter()
                .any(|artifact_id| artifact.name.as_str() == *artifact_id)
        })
        .map(|artifact| ArtifactRef {
            name: ArtifactId::new(format!("{stage_node_id}.{}", artifact.name.as_str())),
            path: artifact.path.clone(),
            role: artifact.role,
            optional: artifact.optional,
        })
        .collect()
}

fn governed_qc_output_ids_for_stage(stage_id: &str) -> &'static [&'static str] {
    bijux_dna_planner_fastq::stage_api::governed_qc_output_ids_for_stage(stage_id)
}

#[cfg(test)]
mod tests {
    use super::{derive_qc_post_metrics, governed_qc_artifacts_for_plan};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, StepId, ToolId};
    use bijux_dna_core::prelude::{ArtifactRef, CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_core::prelude::measure::SeqkitMetrics;
    use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};

    #[test]
    fn governed_qc_artifacts_keep_stage_instance_namespace() {
        let plan = StagePlanV1 {
            stage_id: StageId::from_static("fastq.detect_adapters"),
            stage_instance_id: Some(StepId::from_static("fastq.detect_adapters.tool.fastqc")),
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastqc"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["fastqc".to_string()],
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![
                    ArtifactRef::required(
                        ArtifactId::from_static("adapter_report"),
                        PathBuf::from("detect_adapters/adapter_report.json"),
                        ArtifactRole::ReportJson,
                    ),
                    ArtifactRef::optional(
                        ArtifactId::from_static("adapter_evidence_dir"),
                        PathBuf::from("detect_adapters/fastqc"),
                        ArtifactRole::StageReport,
                    ),
                ],
            },
            out_dir: PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        };

        let inputs = governed_qc_artifacts_for_plan(&plan);
        assert_eq!(inputs.len(), 2);
        assert!(inputs.iter().any(|artifact| {
            artifact.name.as_str() == "fastq.detect_adapters.tool.fastqc.adapter_report"
        }));
        assert!(inputs.iter().any(|artifact| {
            artifact.name.as_str() == "fastq.detect_adapters.tool.fastqc.adapter_evidence_dir"
        }));
    }

    #[test]
    fn qc_post_metrics_report_governed_fastqc_dir_when_present() {
        let temp = tempfile::tempdir().expect("tempdir");
        let multiqc_data = temp.path().join("multiqc_data");
        std::fs::create_dir_all(&multiqc_data).expect("multiqc data dir");
        std::fs::write(temp.path().join("multiqc_report.html"), b"report").expect("report");
        let raw_fastqc_dir = temp.path().join("governed_qc_inputs/detect_adapters/fastqc/fastqc");
        std::fs::create_dir_all(&raw_fastqc_dir).expect("fastqc dir");

        let metrics = derive_qc_post_metrics(
            &SeqkitMetrics {
                reads: 10,
                bases: 100,
                mean_q: 30.0,
                gc_percent: 50.0,
            },
            None,
            temp.path(),
            Some(raw_fastqc_dir.as_path()),
        );

        let expected_raw_fastqc_dir = raw_fastqc_dir.display().to_string();
        assert_eq!(
            metrics.raw_fastqc_dir.as_deref(),
            Some(expected_raw_fastqc_dir.as_str())
        );
        assert!(metrics.multiqc_report.is_some());
        assert!(metrics.multiqc_data.is_some());
    }
}
