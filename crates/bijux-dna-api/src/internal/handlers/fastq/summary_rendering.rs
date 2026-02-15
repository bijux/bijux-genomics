pub(crate) fn render_run_summary(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[bijux_dna_planner_fastq::stage_api::RawFailure],
    merge_decision: Option<&MergeDecisionTrace>,
    correct_decision: Option<&CorrectDecisionTrace>,
    adapter_inference: Option<&serde_json::Value>,
    stage_skips: &[serde_json::Value],
) -> Result<ReportArtifacts> {
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(out_dir);
    bijux_dna_infra::ensure_dir(&root).context("create run summary artifacts dir")?;
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir =
                bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
            let metrics_path = artifacts_dir.join("metrics_envelope.json");
            let metrics =
                read_json_if_exists(&metrics_path).and_then(|value| value.get("metrics").cloned());
            let stage_report_path = artifacts_dir.join("stage_report.json");
            let retention_report_path = artifacts_dir
                .join("reports")
                .join(format!("{}.retention.json", entry.plan.step_id.0));
            serde_json::json!({
                "stage_id": entry.plan.step_id.0,
                "tool_id": entry.plan.image.image,
                "exit_code": entry.result.exit_code,
                "runtime_s": entry.result.runtime_s,
                "memory_mb": entry.result.memory_mb,
                "out_dir": relative_path_string(out_dir, &entry.plan.out_dir),
                "artifacts": {
                    "metrics_envelope": relative_path_string(out_dir, &metrics_path),
                    "stage_report": relative_path_string(out_dir, &stage_report_path),
                    "retention_report": relative_path_string(out_dir, &retention_report_path)
                },
                "metrics": metrics.unwrap_or(serde_json::Value::Null)
            })
        })
        .collect();
    let total_runtime_s: f64 = stage_runs.iter().map(|entry| entry.result.runtime_s).sum();
    let failures_json: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "category": format!("{:?}", failure.category),
            })
        })
        .collect();
    let run_provenance = run_provenance_from_stage_runs(out_dir, stage_runs);
    let summary = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "run_id": run_id,
        "total_runtime_s": total_runtime_s,
        "stages": stages,
        "failures": failures_json,
        "run_provenance": run_provenance,
        "fastq_scientific_summary": fastq_scientific_summary(stage_runs),
        "pipeline_decisions": {
            "merge": merge_decision,
            "correct": correct_decision,
            "adapter_inference": adapter_inference,
            "stage_skips": stage_skips
        }
    });
    let summary_path = root.join("run_summary.json");
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| "write run_summary.json")?;
    let html_path = root.join("run_summary.html");
    let html = render_run_summary_html(&summary);
    bijux_dna_infra::atomic_write_bytes(&html_path, html.as_bytes())
        .context("write run_summary.html")?;
    let summary_json_path = root.join("summary.json");
    bijux_dna_infra::atomic_write_json(&summary_json_path, &summary)
        .context("write summary.json")?;
    let summary_tsv_path = root.join("summary.tsv");
    let mut tsv = String::from("stage_id\ttool_id\truntime_s\texit_code\n");
    for entry in stage_runs {
        let _ = std::fmt::Write::write_fmt(
            &mut tsv,
            format_args!(
                "{}\t{}\t{:.3}\t{}\n",
                entry.plan.step_id.0,
                entry.plan.image.image,
                entry.result.runtime_s,
                entry.result.exit_code
            ),
        );
    }
    bijux_dna_infra::atomic_write_bytes(&summary_tsv_path, tsv.as_bytes())
        .context("write summary.tsv")?;
    let report_html_path = root.join("report.html");
    bijux_dna_infra::atomic_write_bytes(&report_html_path, html.as_bytes())
        .context("write report.html")?;
    Ok(ReportArtifacts {
        run_summary_path: summary_path,
        run_summary_html_path: html_path,
        summary_json_path,
        summary_tsv_path,
        report_html_path,
    })
}

fn fastq_scientific_summary(stage_runs: &[StageExecutionSummary]) -> serde_json::Value {
    let mut pre_qc_stages = 0_u64;
    let mut post_qc_stages = 0_u64;
    let mut classification_stages = 0_u64;
    let mut transforms = 0_u64;
    for entry in stage_runs {
        let id = entry.plan.stage_id.as_str();
        if id == "fastq.validate_pre" || id == "fastq.length_distribution_pre" {
            pre_qc_stages += 1;
        }
        if id == "fastq.qc_post" {
            post_qc_stages += 1;
        }
        if id == "fastq.screen" || id == "fastq.contaminant_screen" || id == "fastq.rrna" {
            classification_stages += 1;
        }
        if matches!(
            id,
            "fastq.trim"
                | "fastq.filter"
                | "fastq.correct"
                | "fastq.merge"
                | "fastq.deduplicate"
                | "fastq.umi"
                | "fastq.host_depletion"
                | "fastq.primer_normalization"
                | "fastq.chimera_detection"
                | "fastq.asv_inference"
                | "fastq.otu_clustering"
                | "fastq.abundance_normalization"
        ) {
            transforms += 1;
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq.scientific_summary.v1",
        "qc": {
            "pre_qc_stages": pre_qc_stages,
            "post_qc_stages": post_qc_stages
        },
        "read_loss_proxy": {
            "transform_stages": transforms,
            "note": "Use stage.metrics.standardized.json and retention reports for exact losses."
        },
        "classification": {
            "classification_stages": classification_stages
        }
    })
}

fn stage_contract_hash_for(stage_id: &str) -> Option<String> {
    if stage_id.starts_with(id_catalog::FASTQ_PREFIX)
        || stage_id.starts_with(id_catalog::CORE_PREFIX)
    {
        return bijux_dna_domain_fastq::stage_contract_hash(stage_id)
            .and_then(std::result::Result::ok);
    }
    if stage_id.starts_with(id_catalog::BAM_PREFIX) {
        return bijux_dna_domain_bam::stage_contract_hash(stage_id)
            .and_then(std::result::Result::ok);
    }
    None
}

pub(crate) fn report_stage_step(out_dir: &Path, steps: &[ExecutionStep]) -> ExecutionStep {
    let mut inputs = Vec::new();
    for entry in steps {
        let artifacts_dir = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.out_dir);
        let metrics_path = artifacts_dir.join("metrics_envelope.json");
        inputs.push(ArtifactRef::optional(
            ArtifactId::new(format!("metrics_envelope_{}", entry.step_id.0)),
            metrics_path,
            ArtifactRole::MetricsEnvelope,
        ));
    }
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(out_dir);
    let outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("summary"),
            root.join("summary.json"),
            ArtifactRole::SummaryJson,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("summary_tsv"),
            root.join("summary.tsv"),
            ArtifactRole::SummaryTsv,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("report_html"),
            root.join("report.html"),
            ArtifactRole::ReportHtml,
        ),
    ];
    build_report_stage_step(out_dir, inputs, outputs)
}
