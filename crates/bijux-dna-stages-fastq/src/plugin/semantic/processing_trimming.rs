use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::observer::{
    parse_bbduk_reads_removed, parse_fastp_metrics, parse_filter_low_complexity_report,
    parse_filter_reads_report, parse_terminal_damage_report, parse_trim_polyg_report,
    parse_trim_reads_report,
};

pub(super) fn observed_trimming_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    match plan.stage_id.as_str() {
        "fastq.trim_reads" => trim_reads_metrics(artifacts),
        "fastq.filter_low_complexity" => filter_low_complexity_metrics(artifacts),
        "fastq.filter_reads" => filter_reads_metrics(artifacts),
        "fastq.trim_terminal_damage" => terminal_damage_metrics(artifacts),
        "fastq.trim_polyg_tails" => trim_polyg_metrics(artifacts),
        _ => None,
    }
}

fn report_raw(artifacts: &[ArtifactRef], name: &str) -> Option<String> {
    artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
}

fn trim_reads_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_trim_reads_report(&report_raw(artifacts, "report_json")?).ok()?;
    let mut semantics = serde_json::Map::from_iter([
        ("paired_mode".to_string(), serde_json::json!(report.paired_mode)),
        ("threads".to_string(), serde_json::json!(report.threads)),
        ("min_length".to_string(), serde_json::json!(report.min_length)),
        ("quality_cutoff".to_string(), serde_json::json!(report.quality_cutoff)),
        ("adapter_policy".to_string(), serde_json::json!(report.adapter_policy)),
        ("adapter_overrides".to_string(), serde_json::json!(report.adapter_overrides)),
        ("polyx_policy".to_string(), serde_json::json!(report.polyx_policy)),
        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
        ("contaminant_policy".to_string(), serde_json::json!(report.contaminant_policy)),
        ("adapter_bank_id".to_string(), serde_json::json!(report.adapter_bank_id)),
        ("polyx_bank_id".to_string(), serde_json::json!(report.polyx_bank_id)),
        ("contaminant_bank_id".to_string(), serde_json::json!(report.contaminant_bank_id)),
        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
        ("mean_q_before".to_string(), serde_json::json!(report.mean_q_before)),
        ("mean_q_after".to_string(), serde_json::json!(report.mean_q_after)),
        (
            "raw_backend_report_format".to_string(),
            serde_json::json!(report.raw_backend_report_format),
        ),
    ]);
    enrich_from_backend_report(
        &mut semantics,
        report.raw_backend_report.as_deref(),
        report.raw_backend_report_format.as_deref(),
    );
    Some(serde_json::Value::Object(semantics))
}

fn filter_low_complexity_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report =
        parse_filter_low_complexity_report(&report_raw(artifacts, "filter_report_json")?).ok()?;
    let mut semantics = serde_json::Map::from_iter([
        ("paired_mode".to_string(), serde_json::json!(report.paired_mode)),
        ("threads".to_string(), serde_json::json!(report.threads)),
        ("entropy_threshold".to_string(), serde_json::json!(report.entropy_threshold)),
        ("polyx_threshold".to_string(), serde_json::json!(report.polyx_threshold)),
        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
        (
            "reads_removed_low_complexity".to_string(),
            serde_json::json!(report.reads_removed_low_complexity),
        ),
        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
        ("mean_q_before".to_string(), serde_json::json!(report.mean_q_before)),
        ("mean_q_after".to_string(), serde_json::json!(report.mean_q_after)),
        (
            "raw_backend_report_format".to_string(),
            serde_json::json!(report.raw_backend_report_format),
        ),
    ]);
    insert_backend_metrics_object(&mut semantics, report.backend_metrics.as_ref(), false);
    Some(serde_json::Value::Object(semantics))
}

fn filter_reads_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_filter_reads_report(&report_raw(artifacts, "report_json")?).ok()?;
    let mut semantics = filter_reads_base_semantics(&report);
    if insert_backend_metrics_object(&mut semantics, report.backend_metrics.as_ref(), true) {
        return Some(serde_json::Value::Object(semantics));
    }
    enrich_from_backend_report(
        &mut semantics,
        report.raw_backend_report.as_deref(),
        report.raw_backend_report_format.as_deref(),
    );
    Some(serde_json::Value::Object(semantics))
}

fn filter_reads_base_semantics(
    report: &bijux_dna_domain_fastq::FilterReadsReportV1,
) -> serde_json::Map<String, serde_json::Value> {
    serde_json::Map::from_iter([
        ("paired_mode".to_string(), serde_json::json!(report.paired_mode)),
        ("threads".to_string(), serde_json::json!(report.threads)),
        ("max_n".to_string(), serde_json::json!(report.max_n)),
        ("max_n_fraction".to_string(), serde_json::json!(report.max_n_fraction)),
        ("max_n_count".to_string(), serde_json::json!(report.max_n_count)),
        (
            "low_complexity_threshold".to_string(),
            serde_json::json!(report.low_complexity_threshold),
        ),
        ("entropy_threshold".to_string(), serde_json::json!(report.entropy_threshold)),
        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
        ("polyx_policy".to_string(), serde_json::json!(report.polyx_policy)),
        ("contaminant_db".to_string(), serde_json::json!(report.contaminant_db)),
        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
        ("reads_dropped".to_string(), serde_json::json!(report.reads_dropped)),
        ("reads_removed_by_n".to_string(), serde_json::json!(report.reads_removed_by_n)),
        (
            "reads_removed_by_entropy".to_string(),
            serde_json::json!(report.reads_removed_by_entropy),
        ),
        (
            "reads_removed_low_complexity".to_string(),
            serde_json::json!(report.reads_removed_low_complexity),
        ),
        ("reads_removed_by_kmer".to_string(), serde_json::json!(report.reads_removed_by_kmer)),
        (
            "reads_removed_contaminant_kmer".to_string(),
            serde_json::json!(report.reads_removed_contaminant_kmer),
        ),
        ("reads_removed_by_length".to_string(), serde_json::json!(report.reads_removed_by_length)),
        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
        ("mean_q_before".to_string(), serde_json::json!(report.mean_q_before)),
        ("mean_q_after".to_string(), serde_json::json!(report.mean_q_after)),
        (
            "raw_backend_report_format".to_string(),
            serde_json::json!(report.raw_backend_report_format),
        ),
    ])
}

fn terminal_damage_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_terminal_damage_report(&report_raw(artifacts, "report_json")?).ok()?;
    Some(serde_json::json!({
        "paired_mode": report.paired_mode,
        "threads": report.threads,
        "damage_mode": report.damage_mode,
        "execution_policy": report.execution_policy,
        "trim_5p_bases": report.trim_5p_bases,
        "trim_3p_bases": report.trim_3p_bases,
        "requested_trim_5p_bases": report.requested_trim_5p_bases,
        "requested_trim_3p_bases": report.requested_trim_3p_bases,
        "udg_classification": report.udg_classification,
        "ct_ga_asymmetry_pre": report.ct_ga_asymmetry_pre,
        "ct_ga_asymmetry_post": report.ct_ga_asymmetry_post,
        "used_fallback": report.used_fallback,
        "backend_metrics": report.backend_metrics,
        "raw_backend_report_format": report.raw_backend_report_format,
    }))
}

fn trim_polyg_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_trim_polyg_report(&report_raw(artifacts, "report_json")?).ok()?;
    let mut semantics = serde_json::Map::from_iter([
        ("paired_mode".to_string(), serde_json::json!(report.paired_mode)),
        ("threads".to_string(), serde_json::json!(report.threads)),
        ("trim_polyg".to_string(), serde_json::json!(report.trim_polyg)),
        ("min_polyg_run".to_string(), serde_json::json!(report.min_polyg_run)),
        ("bases_trimmed_polyg".to_string(), serde_json::json!(report.bases_trimmed_polyg)),
        ("polyx_bank_id".to_string(), serde_json::json!(report.polyx_bank_id)),
        ("polyx_bank_hash".to_string(), serde_json::json!(report.polyx_bank_hash)),
        ("polyx_preset".to_string(), serde_json::json!(report.polyx_preset)),
        ("raw_backend_report".to_string(), serde_json::json!(report.raw_backend_report)),
        (
            "raw_backend_report_format".to_string(),
            serde_json::json!(report.raw_backend_report_format),
        ),
    ]);
    if insert_backend_metrics_object(&mut semantics, report.backend_metrics.as_ref(), true) {
        return Some(serde_json::Value::Object(semantics));
    }
    enrich_from_backend_report(
        &mut semantics,
        report.raw_backend_report.as_deref(),
        report.raw_backend_report_format.as_deref(),
    );
    Some(serde_json::Value::Object(semantics))
}

fn insert_backend_metrics_object(
    semantics: &mut serde_json::Map<String, serde_json::Value>,
    backend_metrics: Option<&serde_json::Value>,
    skip_schema_version: bool,
) -> bool {
    let Some(metrics) = backend_metrics.and_then(serde_json::Value::as_object) else {
        return false;
    };
    for (metric_name, metric_value) in metrics {
        if skip_schema_version && metric_name == "schema_version" {
            continue;
        }
        semantics.insert(metric_name.clone(), metric_value.clone());
    }
    true
}

fn enrich_from_backend_report(
    semantics: &mut serde_json::Map<String, serde_json::Value>,
    raw_backend_report: Option<&str>,
    raw_backend_report_format: Option<&str>,
) {
    let (Some(raw_backend_report), Some(raw_backend_report_format)) =
        (raw_backend_report, raw_backend_report_format)
    else {
        return;
    };
    let Ok(raw_backend_payload) = fs::read_to_string(raw_backend_report) else {
        return;
    };
    match raw_backend_report_format {
        "fastp_json" => insert_fastp_metrics(semantics, &raw_backend_payload),
        "bbduk_stats" => insert_bbduk_metrics(semantics, &raw_backend_payload),
        _ => {}
    }
}

fn insert_fastp_metrics(semantics: &mut serde_json::Map<String, serde_json::Value>, raw: &str) {
    let Ok(metrics) = parse_fastp_metrics(raw) else {
        return;
    };
    semantics.insert("passed_filter_reads".to_string(), serde_json::json!(metrics.passed_filter_reads));
    semantics.insert("low_quality_reads".to_string(), serde_json::json!(metrics.low_quality_reads));
    semantics.insert("too_many_n_reads".to_string(), serde_json::json!(metrics.too_many_n_reads));
    semantics.insert("too_short_reads".to_string(), serde_json::json!(metrics.too_short_reads));
}

fn insert_bbduk_metrics(semantics: &mut serde_json::Map<String, serde_json::Value>, raw: &str) {
    if let Ok(reads_removed) = parse_bbduk_reads_removed(raw) {
        semantics.insert("reads_removed".to_string(), serde_json::json!(reads_removed));
    }
}
