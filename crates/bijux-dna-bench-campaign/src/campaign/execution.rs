use super::model::{CampaignScope, PlannedSamples};
use crate::cache::{
    prepare_sortmerna_sample_workdir, sortmerna_shared_index_dir, sortmerna_shared_index_seeded,
    warm_sortmerna_shared_index_cache,
};
use crate::execution::{execute_pending_samples, execute_sample, PostSuccessAction};
use anyhow::Result;
use std::path::Path;
pub(super) fn execute_samples(
    scope: &CampaignScope<'_>,
    planned: PlannedSamples,
) -> Result<PlannedSamples> {
    let args = scope.args;
    let workspace_config = scope.workspace_config;
    let executor = scope.executor;
    let program = args.program.clone();
    let repo_root = args.repo_root.clone();
    let out_root = args.out_root.clone();
    let platform = args.platform.clone();
    let tools = scope.tools.clone();
    let deplete_rrna_options = &scope.deplete_rrna_options;
    let PlannedSamples { mut runs, mut pending, mut extra_manifest_fields } = planned;
    if !pending.is_empty() {
        if let Some(options) = deplete_rrna_options.as_ref() {
            if tools == ["sortmerna"] && !args.dry_run {
                let shared_idx_dir = sortmerna_shared_index_dir(&out_root, &options.rrna_bundle_id);
                warm_sortmerna_shared_index_cache(
                    workspace_config,
                    &platform,
                    &options.rrna_db,
                    Path::new(&pending[0].sample.r1),
                    &shared_idx_dir,
                    args.threads,
                    executor,
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
                    runs.push(execute_sample(&program, &repo_root, first, executor)?);
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
        runs.extend(execute_pending_samples(
            &program,
            &repo_root,
            std::mem::take(&mut pending),
            executor,
            args.sample_jobs,
        )?);
    }

    Ok(PlannedSamples { runs, pending, extra_manifest_fields })
}
