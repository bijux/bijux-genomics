use super::model::{CampaignScope, PlannedSamples};
use super::paths::benchmark_sample_root;
use crate::execution::PendingSampleRun;
use crate::planning::build_stage_command_args;
use crate::qc::{prepare_report_qc_sample, StageSamplePreparation};
use anyhow::Result;
use bijux_dna_bench_corpus::normalized::NormalizedFastqSample as CorpusNormalizedSample;
use bijux_dna_bench_corpus::run::BenchmarkCorpusSampleRun;
pub(super) fn plan_samples(
    scope: &CampaignScope<'_>,
    selected_samples: Vec<CorpusNormalizedSample>,
    extra_manifest_fields: std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<PlannedSamples> {
    let args = scope.args;
    let workspace_config = scope.workspace_config;
    let executor = scope.executor;
    let contract = &args.contract;
    let corpus_spec = &args.source;
    let program = &args.program;
    let repo_root = &args.repo_root;
    let out_root = &args.out_root;
    let platform = &args.platform;
    let stage_spec = scope.stage_spec;
    let tools = &scope.tools;
    let resolved_stage_args = &scope.resolved_stage_args;
    let runtime_env = &scope.runtime_env;
    let mut runs = Vec::new();
    let mut pending = Vec::new();
    let mut extra_manifest_fields = extra_manifest_fields;
    for sample in selected_samples {
        let sample_root = benchmark_sample_root(out_root, stage_spec.report_dir, &sample.sample_id);
        crate::admission::owned_sample_root(out_root, &sample_root)?;
        let report_json = out_root
            .join("bench")
            .join(stage_spec.report_dir)
            .join(&sample.sample_id)
            .join("report.json");
        let prepared = if contract.stage_id == "fastq.report_qc" {
            prepare_report_qc_sample(
                &crate::qc::PreparationContext {
                    program,
                    repo_root,
                    workspace_config,
                    corpus_id: &corpus_spec.corpus_id,
                    platform,
                    dry_run: args.dry_run,
                    executor,
                },
                out_root,
                &sample,
            )?
        } else {
            StageSamplePreparation::default()
        };

        if let Some(run) = super::resume::try_resume_sample(
            scope,
            &sample,
            &sample_root,
            &report_json,
            &prepared.run_extra_fields,
            &mut extra_manifest_fields,
        )? {
            runs.push(run);
            continue;
        }

        let command_args = build_stage_command_args(crate::planning::StageCommandRequest {
            stage_spec,
            platform,
            sample: &sample,
            out_root,
            tools,
            threads: args.threads,
            jobs: args.jobs,
            stage_args: resolved_stage_args,
            extra_stage_args: &prepared.extra_stage_args,
        });

        let command = std::iter::once(program.display().to_string())
            .chain(command_args.iter().cloned())
            .collect::<Vec<_>>();
        if args.dry_run {
            runs.push(BenchmarkCorpusSampleRun {
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

    Ok(PlannedSamples { runs, pending, extra_manifest_fields })
}
