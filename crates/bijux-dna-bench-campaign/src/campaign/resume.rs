use anyhow::{Context, Result};
use std::{fs, path::Path};
pub(crate) fn sample_report_is_resume_ready(sample_report: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(sample_report) else {
        return false;
    };
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    if payload
        .get("failures")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|row| !row.is_empty())
    {
        return false;
    }
    if payload.get("gate").and_then(|row| row.get("passes")).and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return false;
    }
    payload.get("records").and_then(serde_json::Value::as_array).is_some_and(|row| !row.is_empty())
}

pub(crate) fn reset_sample_payload(sample_root: &Path) -> Result<()> {
    if sample_root.is_dir() {
        fs::remove_dir_all(sample_root)
            .with_context(|| format!("remove stale sample payload {}", sample_root.display()))?;
    }
    Ok(())
}

use super::model::CampaignScope;
use crate::cache::{prune_sortmerna_sample_payload, sortmerna_shared_index_dir};
use bijux_dna_bench_corpus::normalized::NormalizedFastqSample;
use bijux_dna_bench_corpus::run::BenchmarkCorpusSampleRun;
use std::collections::BTreeMap;
pub(super) fn try_resume_sample(
    scope: &CampaignScope<'_>,
    sample: &NormalizedFastqSample,
    sample_root: &Path,
    report_json: &Path,
    run_extra_fields: &BTreeMap<String, serde_json::Value>,
    extra_manifest_fields: &mut BTreeMap<String, serde_json::Value>,
) -> Result<Option<BenchmarkCorpusSampleRun>> {
    let args = scope.args;
    let stage_spec = scope.stage_spec;
    let tools = scope.tools.as_slice();
    let deplete_rrna_options = &scope.deplete_rrna_options;
    let out_root = &args.out_root;
    if args.resume && !args.dry_run {
        if stage_spec.strict_resume_report {
            if sample_root.is_dir() && !report_json.is_file() {
                reset_sample_payload(sample_root)?;
            }
            if report_json.is_file() {
                if sample_report_is_resume_ready(report_json) {
                    if let Some(options) = deplete_rrna_options.as_ref() {
                        if tools == ["sortmerna"] && !args.dry_run {
                            prune_sortmerna_sample_payload(out_root, &sample.sample_id)?;
                            extra_manifest_fields.insert(
                                "rrna_index_dir".to_string(),
                                serde_json::Value::String(
                                    sortmerna_shared_index_dir(out_root, &options.rrna_bundle_id)
                                        .display()
                                        .to_string(),
                                ),
                            );
                        }
                    }
                    return Ok(Some(BenchmarkCorpusSampleRun {
                        sample_id: sample.sample_id.clone(),
                        r1: sample.r1.display().to_string(),
                        r2: sample.r2.as_ref().map(|path| path.display().to_string()),
                        layout: sample.layout.clone(),
                        status: "skipped_existing_report".to_string(),
                        exit_code: 0,
                        command: Vec::new(),
                        report_json: report_json.display().to_string(),
                        extra_fields: run_extra_fields.clone(),
                    }));
                }
                reset_sample_payload(sample_root)?;
            }
        } else if report_json.is_file() {
            return Ok(Some(BenchmarkCorpusSampleRun {
                sample_id: sample.sample_id.clone(),
                r1: sample.r1.display().to_string(),
                r2: sample.r2.as_ref().map(|path| path.display().to_string()),
                layout: sample.layout.clone(),
                status: "skipped_existing_report".to_string(),
                exit_code: 0,
                command: Vec::new(),
                report_json: report_json.display().to_string(),
                extra_fields: run_extra_fields.clone(),
            }));
        }
    }

    Ok(None)
}
