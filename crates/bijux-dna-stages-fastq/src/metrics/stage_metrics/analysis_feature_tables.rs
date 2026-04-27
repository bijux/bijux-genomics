use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::path_from_params;

pub(super) fn normalize_abundance_metrics(plan: &StagePlanV1) -> serde_json::Value {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("normalize_abundance_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_normalize_abundance_report(&raw).ok());
    if let Some(report) = governed_report {
        serde_json::json!({
            "table_rows": report.table_rows,
            "sample_count": report.sample_count,
            "feature_count": report.feature_count,
            "zero_fraction": report.zero_fraction,
            "method": report.method,
            "input_value_column": report.input_value_column,
            "normalized_value_column": report.normalized_value_column,
            "compositional_rule": report.compositional_rule,
            "scale_factor": report.scale_factor,
            "per_sample_sums": report.per_sample_sums,
        })
    } else {
        serde_json::json!({})
    }
}

pub(super) fn infer_asvs_metrics(plan: &StagePlanV1) -> serde_json::Value {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("infer_asvs_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_infer_asvs_report(&raw).ok());
    if let Some(report) = governed_report {
        serde_json::json!({
            "asv_count": report.asv_count,
            "sample_count": report.sample_count,
            "representative_sequence_count": report.representative_sequence_count,
            "paired_mode": report.paired_mode,
            "denoising_method": report.denoising_method,
            "pooling_mode": report.pooling_mode,
            "chimera_policy": report.chimera_policy,
            "output_table_kind": report.output_table_kind,
            "used_fallback": report.used_fallback,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
    } else {
        serde_json::json!({})
    }
}

pub(super) fn cluster_otus_metrics(plan: &StagePlanV1) -> serde_json::Value {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("cluster_otus_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_cluster_otus_report(&raw).ok());
    if let Some(report) = governed_report {
        serde_json::json!({
            "otu_identity": report.otu_identity,
            "threads": report.threads,
            "otu_count": report.otu_count,
            "sample_count": report.sample_count,
            "representative_sequence_count": report.representative_sequence_count,
            "output_table_kind": report.output_table_kind,
            "used_fallback": report.used_fallback,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
    } else {
        serde_json::json!({})
    }
}

pub(super) fn index_reference_metrics(plan: &StagePlanV1) -> serde_json::Value {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("index_reference_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_index_reference_report(&raw).ok());
    if let Some(report) = governed_report {
        serde_json::json!({
            "threads": report.threads,
            "index_format": report.index_format,
            "reference_bytes": report.reference_bytes,
            "index_bytes": report.index_bytes,
            "index_file_count": report.index_file_count,
            "emitted_file_count": report.emitted_files.len(),
        })
    } else {
        serde_json::json!({})
    }
}
