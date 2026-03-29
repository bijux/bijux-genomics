use super::*;

pub(super) fn observed_processing_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.remove_duplicates" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_remove_duplicates_report(&raw_report) {
                    let provenance = artifacts
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "duplicate_provenance_json")
                        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
                        .and_then(|raw| parse_remove_duplicates_provenance(&raw).ok());
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "dedup_mode": report.dedup_mode,
                        "keep_order": report.keep_order,
                        "pair_count_match": report.pair_count_match,
                        "duplicates_removed": report.duplicates_removed,
                        "dedup_rate": report.dedup_rate,
                        "duplicate_class_count": report.duplicate_classes.len(),
                        "duplicate_classes": report.duplicate_classes,
                        "duplicate_provenance_json": report.duplicate_provenance_json,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "backend_log": provenance.as_ref().and_then(|value| value.backend_log.clone()),
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_chimeras" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_remove_chimeras_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "method": report.method,
                        "detection_scope": report.detection_scope,
                        "reads_in": report.reads_in,
                        "reads_out": report.reads_out,
                        "chimeras_removed": report.chimeras_removed,
                        "chimera_fraction": report.chimera_fraction,
                        "used_fallback": report.used_fallback,
                        "chimeras_fasta": report.chimeras_fasta,
                        "uchime_report_tsv": report.uchime_report_tsv,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "backend_metrics": report.backend_metrics,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.detect_adapters" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_detect_adapters_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "inspection_mode": report.inspection_mode,
                        "report_only": report.report_only,
                        "evidence_engine": report.evidence_engine,
                        "evidence_scope": report.evidence_scope,
                        "evidence_format": report.evidence_format,
                        "candidate_adapter_count": report.candidate_adapter_count,
                        "adapter_trimmed_fraction": report.adapter_trimmed_fraction,
                        "adapter_content_max": report.adapter_content_max,
                        "adapter_content_mean": report.adapter_content_mean,
                        "duplication_rate": report.duplication_rate,
                        "n_rate": report.n_rate,
                        "kmer_warning_count": report.kmer_warning_count,
                        "overrepresented_sequence_count": report.overrepresented_sequence_count,
                        "adapter_evidence_dir": report.adapter_evidence_dir,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_trim_reads_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "min_length".to_string(),
                            serde_json::json!(report.min_length),
                        ),
                        (
                            "quality_cutoff".to_string(),
                            serde_json::json!(report.quality_cutoff),
                        ),
                        (
                            "adapter_policy".to_string(),
                            serde_json::json!(report.adapter_policy),
                        ),
                        (
                            "adapter_overrides".to_string(),
                            serde_json::json!(report.adapter_overrides),
                        ),
                        (
                            "polyx_policy".to_string(),
                            serde_json::json!(report.polyx_policy),
                        ),
                        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
                        (
                            "contaminant_policy".to_string(),
                            serde_json::json!(report.contaminant_policy),
                        ),
                        (
                            "adapter_bank_id".to_string(),
                            serde_json::json!(report.adapter_bank_id),
                        ),
                        (
                            "polyx_bank_id".to_string(),
                            serde_json::json!(report.polyx_bank_id),
                        ),
                        (
                            "contaminant_bank_id".to_string(),
                            serde_json::json!(report.contaminant_bank_id),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_payload) = fs::read_to_string(raw_backend_report) {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_payload) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_payload)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return Some(serde_json::Value::Object(semantics));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.filter_low_complexity" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "filter_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_filter_low_complexity_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "entropy_threshold".to_string(),
                            serde_json::json!(report.entropy_threshold),
                        ),
                        (
                            "polyx_threshold".to_string(),
                            serde_json::json!(report.polyx_threshold),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        (
                            "reads_removed_low_complexity".to_string(),
                            serde_json::json!(report.reads_removed_low_complexity),
                        ),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                    }
                    return Some(serde_json::Value::Object(semantics));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.extract_umis" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_extract_umis_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "umi_pattern".to_string(),
                            serde_json::json!(report.umi_pattern),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "reads_with_umi".to_string(),
                            serde_json::json!(report.reads_with_umi),
                        ),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                    }
                    return Some(serde_json::Value::Object(semantics));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.filter_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_filter_reads_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        ("max_n".to_string(), serde_json::json!(report.max_n)),
                        (
                            "max_n_fraction".to_string(),
                            serde_json::json!(report.max_n_fraction),
                        ),
                        (
                            "max_n_count".to_string(),
                            serde_json::json!(report.max_n_count),
                        ),
                        (
                            "low_complexity_threshold".to_string(),
                            serde_json::json!(report.low_complexity_threshold),
                        ),
                        (
                            "entropy_threshold".to_string(),
                            serde_json::json!(report.entropy_threshold),
                        ),
                        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
                        (
                            "polyx_policy".to_string(),
                            serde_json::json!(report.polyx_policy),
                        ),
                        (
                            "contaminant_db".to_string(),
                            serde_json::json!(report.contaminant_db),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        (
                            "reads_dropped".to_string(),
                            serde_json::json!(report.reads_dropped),
                        ),
                        (
                            "reads_removed_by_n".to_string(),
                            serde_json::json!(report.reads_removed_by_n),
                        ),
                        (
                            "reads_removed_by_entropy".to_string(),
                            serde_json::json!(report.reads_removed_by_entropy),
                        ),
                        (
                            "reads_removed_low_complexity".to_string(),
                            serde_json::json!(report.reads_removed_low_complexity),
                        ),
                        (
                            "reads_removed_by_kmer".to_string(),
                            serde_json::json!(report.reads_removed_by_kmer),
                        ),
                        (
                            "reads_removed_contaminant_kmer".to_string(),
                            serde_json::json!(report.reads_removed_contaminant_kmer),
                        ),
                        (
                            "reads_removed_by_length".to_string(),
                            serde_json::json!(report.reads_removed_by_length),
                        ),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            if metric_name == "schema_version" {
                                continue;
                            }
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                        return Some(serde_json::Value::Object(semantics));
                    }
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_payload) = fs::read_to_string(raw_backend_report) {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_payload) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_payload)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return Some(serde_json::Value::Object(semantics));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_duplicates" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                let parsed: Result<(u64, u64), _> = parse_deduplicate_report(&raw_report);
                if let Ok((reads_in, reads_out)) = parsed {
                    let duplicates_removed = reads_in.saturating_sub(reads_out);
                    let dedup_rate = if reads_in > 0 {
                        duplicates_removed as f64 / reads_in as f64
                    } else {
                        0.0
                    };
                    return Some(serde_json::json!({
                        "reads_in": reads_in,
                        "reads_out": reads_out,
                        "duplicates_removed": duplicates_removed,
                        "dedup_rate": dedup_rate,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_terminal_damage" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_terminal_damage_report(&raw_report) {
                    return Some(serde_json::json!({
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
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_polyg_tails" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_trim_polyg_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "trim_polyg".to_string(),
                            serde_json::json!(report.trim_polyg),
                        ),
                        (
                            "min_polyg_run".to_string(),
                            serde_json::json!(report.min_polyg_run),
                        ),
                        (
                            "bases_trimmed_polyg".to_string(),
                            serde_json::json!(report.bases_trimmed_polyg),
                        ),
                        (
                            "polyx_bank_id".to_string(),
                            serde_json::json!(report.polyx_bank_id),
                        ),
                        (
                            "polyx_bank_hash".to_string(),
                            serde_json::json!(report.polyx_bank_hash),
                        ),
                        (
                            "polyx_preset".to_string(),
                            serde_json::json!(report.polyx_preset),
                        ),
                        (
                            "raw_backend_report".to_string(),
                            serde_json::json!(report.raw_backend_report),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            if metric_name == "schema_version" {
                                continue;
                            }
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                        return Some(serde_json::Value::Object(semantics));
                    }
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_report) = fs::read_to_string(raw_backend_report) {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_report) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_report)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return Some(serde_json::Value::Object(semantics));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.correct_errors" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_correct_errors_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "correction_engine": report.correction_engine,
                        "quality_encoding": report.quality_encoding,
                        "kmer_size": report.kmer_size,
                        "genome_size": report.genome_size,
                        "max_memory_gb": report.max_memory_gb,
                        "trusted_kmer_artifact": report.trusted_kmer_artifact,
                        "conservative_mode": report.conservative_mode,
                        "corrected_reads": report.corrected_reads,
                        "kmer_fix_rate": report.kmer_fix_rate,
                        "correction_effect": report.correction_effect,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "input_r1": report.input_r1,
                        "input_r2": report.input_r2,
                        "output_r1": report.output_r1,
                        "output_r2": report.output_r2,
                    }));
                }
            }
        }
    }
    None
}
