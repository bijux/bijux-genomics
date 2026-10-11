//! Owner: scratch layout composition over explicit commands, jobs and current sources.
use crate::repo::hpc_scratch::{destination, input_link, relative};
use crate::{
    collect_hpc_job_source_inputs, materialize_stage_id, resolve_hpc_job_resources,
    AllDomainRenderedCommandRow, BenchmarkLocalAllDomainSlurmSubmitJob as SubmitJob,
    BenchmarkLocalHpcScratchLayout as Layout, BenchmarkLocalHpcScratchLayoutJob as Job,
    HpcJobResourceHints, HpcStageInputHint,
};
use anyhow::{anyhow, ensure, Result};
use bijux_dna_bench_model::{
    assemble_hpc_scratch_layout, project_hpc_scratch_job, HpcScratchJobRequest,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
/// In-memory pipeline command projection; persisted command schemas stay with their producer.
pub struct HpcScratchPipelineCommands {
    pub pipeline_id: String,
    pub node_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub domain: String,
    pub render_status: String,
    pub command_argv: Vec<Vec<String>>,
}
/// Current commands supplied by the public command adapters.
pub struct HpcScratchCommands {
    pub benchmark_rows: Vec<AllDomainRenderedCommandRow>,
    pub pipeline_rows: Vec<HpcScratchPipelineCommands>,
}
impl HpcScratchCommands {
    /// Stage identities whose input hints are needed, including materialization commands.
    #[must_use]
    pub fn stage_ids(&self, jobs: &[SubmitJob]) -> BTreeSet<String> {
        let mut ids = jobs.iter().map(|job| job.stage_id.clone()).collect::<BTreeSet<_>>();
        for row in &self.benchmark_rows {
            ids.extend(
                row.command_steps.iter().filter_map(|step| materialize_stage_id(&step.argv)),
            );
        }
        for row in &self.pipeline_rows {
            ids.extend(row.command_argv.iter().filter_map(|argv| materialize_stage_id(argv)));
        }
        ids
    }
}
/// Explicit request separating source observation from CLI defaults and planner acquisition.
pub struct HpcScratchRequest<'a> {
    pub repository_root: &'a Path,
    pub output_path: &'a Path,
    pub staging_root: &'a Path,
    pub scratch_root: &'a Path,
    pub jobs: &'a [SubmitJob],
    pub commands: &'a HpcScratchCommands,
    pub stage_input_hints: &'a BTreeMap<String, Vec<HpcStageInputHint>>,
    pub resource_hints: &'a HpcJobResourceHints,
    pub generated_output_root: &'a str,
}
/// Observe sources and assemble an admitted layout without publishing it.
/// # Errors
/// Rejects unsafe destinations, ambiguous or mismatched command identities and invalid sources/layouts.
pub fn collect_hpc_scratch_layout(request: HpcScratchRequest<'_>) -> Result<Layout> {
    let root = request.repository_root;
    let output = destination(root, request.output_path)?;
    let staging = destination(root, request.staging_root)?;
    let scratch = destination(root, request.scratch_root)?;
    let benchmark_rows = unique_benchmark_rows(&request.commands.benchmark_rows)?;
    let pipeline_rows = unique_pipeline_rows(&request.commands.pipeline_rows)?;
    let jobs = request
        .jobs
        .iter()
        .map(|job| {
            let resources = resolve_hpc_job_resources(
                request.resource_hints,
                &job.domain,
                &job.stage_id,
                &job.tool_id,
            );
            let (scratch_job_root, consumer, stage, argv) =
                match (&job.result_id, &job.pipeline_id, &job.node_id) {
                    (Some(result), None, None) => {
                        let row = benchmark_rows.get(result.as_str()).ok_or_else(|| {
                            anyhow!(
                                "benchmark scratch layout is missing rendered command row for `{}`",
                                job.job_id_local
                            )
                        })?;
                        ensure!(
                            row.domain == job.domain
                                && row.stage_id == job.stage_id
                                && row.tool_id == job.tool_id
                                && row.corpus_id == job.corpus_id
                                && row.asset_profile_id == job.asset_profile_id,
                            "benchmark scratch layout command identity differs from job `{}`",
                            job.job_id_local
                        );
                        (
                            scratch
                                .join("benchmark-results")
                                .join(&row.domain)
                                .join(&row.corpus_id)
                                .join(&row.stage_id)
                                .join(&row.asset_profile_id)
                                .join(&row.tool_id)
                                .join(&row.result_id),
                            row.result_id.clone(),
                            row.stage_id.as_str(),
                            row.command_steps
                                .iter()
                                .map(|step| step.argv.as_slice())
                                .collect::<Vec<_>>(),
                        )
                    }
                    (None, Some(pipeline), Some(node)) => {
                        let row = pipeline_rows
                            .get(&(pipeline.as_str(), node.as_str()))
                            .ok_or_else(|| {
                                anyhow!(
                                "pipeline scratch layout is missing rendered command row for `{}`",
                                job.job_id_local
                            )
                            })?;
                        ensure!(
                            row.domain == job.domain
                                && row.stage_id == job.stage_id
                                && row.tool_id == job.tool_id,
                            "pipeline scratch layout command identity differs from job `{}`",
                            job.job_id_local
                        );
                        (
                            scratch
                                .join("essential-pipelines")
                                .join(&row.pipeline_id)
                                .join(&row.node_id),
                            format!("{}::{}", row.pipeline_id, row.node_id),
                            row.stage_id.as_str(),
                            row.command_argv.iter().map(Vec::as_slice).collect::<Vec<_>>(),
                        )
                    }
                    _ => {
                        return Err(anyhow!(
                            "HPC scratch layout could not classify submit-manifest job `{}`",
                            job.job_id_local
                        ))
                    }
                };
            let input_links_root = scratch_job_root.join("inputs");
            let inputs = collect_hpc_job_source_inputs(
                root,
                &consumer,
                stage,
                argv,
                request.stage_input_hints,
                request.generated_output_root,
            )?
            .into_iter()
            .map(|input| input_link(root, &staging, &input_links_root, input))
            .collect::<Result<Vec<_>>>()?;
            project_hpc_scratch_job(HpcScratchJobRequest {
                repository_root: root,
                job,
                scratch_root: scratch_job_root,
                input_links_root,
                input_links: inputs,
                resources: &resources,
            })
        })
        .collect::<Result<Vec<Job>>>()?;
    assemble_hpc_scratch_layout(
        relative(root, &output),
        relative(root, &staging),
        relative(root, &scratch),
        jobs,
    )
}
fn unique_benchmark_rows(
    rows: &[AllDomainRenderedCommandRow],
) -> Result<BTreeMap<&str, &AllDomainRenderedCommandRow>> {
    let mut result = BTreeMap::new();
    for row in rows {
        ensure!(
            result.insert(row.result_id.as_str(), row).is_none(),
            "duplicate scratch benchmark command identity `{}`",
            row.result_id
        );
    }
    Ok(result)
}
fn unique_pipeline_rows(
    rows: &[HpcScratchPipelineCommands],
) -> Result<BTreeMap<(&str, &str), &HpcScratchPipelineCommands>> {
    let mut result = BTreeMap::new();
    for row in rows {
        ensure!(
            row.render_status == "rendered",
            "HPC scratch layout requires rendered essential pipeline command argv rows"
        );
        ensure!(
            result.insert((row.pipeline_id.as_str(), row.node_id.as_str()), row).is_none(),
            "duplicate scratch pipeline command identity `{}::{}`",
            row.pipeline_id,
            row.node_id
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn scratch_collection_binds_commands_and_rechecks_current_sources() -> Result<()> {
        let temp = bijux_dna_testkit::temp::tempdir_for("scratch-collection");
        let root = temp.path();
        std::fs::create_dir_all(root.join("benchmarks"))?;
        std::fs::write(root.join("benchmarks/input.txt"), b"first")?;
        let submit: SubmitJob = serde_json::from_value(json!({
            "job_id_local":"job-benchmark", "domain":"bam", "stage_id":"bam.align", "pipeline_id":null, "node_id":null,
            "tool_id":"bwa-mem2", "corpus_id":"corpus-01-mini", "asset_profile_id":"sample-set", "result_id":"result-01",
            "script_path":"artifacts/runs/bench/jobs/job.sh", "stdout":"artifacts/runs/bench/logs/job/stdout.log", "stderr":"artifacts/runs/bench/logs/job/stderr.log",
            "outputs":["artifacts/runs/bench/results/job/output.bam"], "dependencies":[],
            "resources":{"cpus_per_task":1,"memory_mb":2048,"time_limit":"00:20:00"}
        }))?;
        let row: AllDomainRenderedCommandRow = serde_json::from_value(json!({
            "result_id":"result-01", "domain":"bam", "stage_id":"bam.align", "tool_id":"bwa-mem2", "corpus_id":"corpus-01-mini", "asset_profile_id":"sample-set",
            "readiness_kind":"ready", "benchmark_status":"ready", "command_source":"governed",
            "command_steps":[{"step_id":"align", "step_kind":"tool", "consumes_previous_stdout":false, "argv":["tool","benchmarks/input.txt"], "command":"tool benchmarks/input.txt"}], "script_commands":[]
        }))?;
        let mut commands = HpcScratchCommands { benchmark_rows: vec![row], pipeline_rows: vec![] };
        let mut jobs = vec![submit];
        let hints = BTreeMap::new();
        let resources = BTreeMap::new();
        let output = Path::new("artifacts/runs/bench/request/scratch-layout.json");
        let collect = |commands: &HpcScratchCommands, jobs: &[SubmitJob]| {
            collect_hpc_scratch_layout(HpcScratchRequest {
                repository_root: root,
                output_path: output,
                staging_root: Path::new("artifacts/runs/bench/request/staged"),
                scratch_root: Path::new("artifacts/runs/bench/request/scratch"),
                jobs,
                commands,
                stage_input_hints: &hints,
                resource_hints: &resources,
                generated_output_root: "artifacts/generated",
            })
        };
        let first = collect(&commands, &jobs)?;
        assert_eq!(first.selected_job_count, 1);
        assert_eq!(first.input_link_count, 1);
        assert!(first.jobs[0].cleanup_policy.retain_scratch_on_failure);
        assert_eq!(first.jobs[0].output_root, "artifacts/runs/bench/results/job");
        assert!(!root.join("artifacts").exists(), "collection must not publish directories");
        commands.benchmark_rows.push(commands.benchmark_rows[0].clone());
        assert!(collect(&commands, &jobs)
            .err()
            .is_some_and(|error| error.to_string().contains("duplicate scratch benchmark")));
        commands.benchmark_rows.pop();
        commands.benchmark_rows[0].tool_id = "different-tool".into();
        assert!(collect(&commands, &jobs)
            .err()
            .is_some_and(|error| error.to_string().contains("command identity differs")));
        commands.benchmark_rows[0].tool_id = jobs[0].tool_id.clone();
        jobs[0].stderr = "artifacts/runs/bench/other/stderr.log".into();
        assert!(collect(&commands, &jobs)
            .err()
            .is_some_and(|error| error.to_string().contains("roots diverged")));
        jobs[0].stderr = "artifacts/runs/bench/logs/job/stderr.log".into();
        std::fs::write(root.join("benchmarks/input.txt"), b"changed")?;
        let current = collect(&commands, &jobs)?;
        assert_ne!(
            current.jobs[0].input_links[0].checksum_sha256,
            first.jobs[0].input_links[0].checksum_sha256
        );
        crate::publish_hpc_scratch_layout(root, output, &current)?;
        assert_eq!(crate::load_hpc_scratch_layout(root, output)?, current);
        assert!(crate::publish_hpc_scratch_layout(
            root,
            Path::new("artifacts/runs/bench/../escaped.json"),
            &current
        )
        .is_err());
        assert!(crate::publish_hpc_scratch_layout(
            root,
            Path::new("artifacts/runs/bench/other.json"),
            &current
        )
        .is_err());
        let renamed = root.join("artifacts/runs/bench/request/renamed.json");
        std::fs::copy(root.join(output), &renamed)?;
        assert!(crate::load_hpc_scratch_layout(root, &renamed).is_err());
        jobs[0].result_id = None;
        jobs[0].pipeline_id = Some("pipeline-01".into());
        jobs[0].node_id = Some("align".into());
        commands.benchmark_rows.clear();
        commands.pipeline_rows.push(HpcScratchPipelineCommands {
            pipeline_id: "pipeline-01".into(),
            node_id: "align".into(),
            domain: "bam".into(),
            stage_id: "bam.align".into(),
            tool_id: "bwa-mem2".into(),
            render_status: "rendered".into(),
            command_argv: vec![vec!["tool".into(), "benchmarks/input.txt".into()]],
        });
        let pipeline = collect(&commands, &jobs)?;
        assert_eq!(pipeline.essential_pipeline_job_count, 1);
        assert_eq!(pipeline.benchmark_job_count, 0);
        commands.pipeline_rows[0].render_status = "structured_skip".into();
        assert!(collect(&commands, &jobs)
            .err()
            .is_some_and(|error| error.to_string().contains("requires rendered")));
        commands.pipeline_rows[0].render_status = "rendered".into();
        std::fs::remove_file(root.join("benchmarks/input.txt"))?;
        assert!(collect(&commands, &jobs).is_err());
        Ok(())
    }
}
