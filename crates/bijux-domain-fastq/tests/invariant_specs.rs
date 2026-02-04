#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::unnecessary_wraps,
    clippy::unwrap_used
)]

use anyhow::Result;
use bijux_core::{
    FastqDeltaMetricsV1, FastqFilterMetricsV1, FastqMergeMetricsV1, FastqTrimMetricsV1,
    FastqValidateMetricsV1, RetentionReportMetricV1,
};
use bijux_domain_fastq::invariants::{evaluate_invariants, thresholds_from_env};
use bijux_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_domain_fastq::params::trim::TrimEffectiveParams;
use bijux_domain_fastq::params::validate::ValidateEffectiveParams;
use bijux_domain_fastq::{fastq_invariant_specs, PairedMode};

fn retention_metric() -> RetentionReportMetricV1 {
    RetentionReportMetricV1 {
        value: 0.9,
        numerator_reads: 90,
        denominator_reads: 100,
        numerator_bases: 900,
        denominator_bases: 1000,
        definition: "reads_out/reads_in".to_string(),
        stage_boundary: "stage".to_string(),
        conditions: serde_json::json!({}),
    }
}

fn trim_metrics(read_retention: f64, mean_q_delta: f64) -> FastqTrimMetricsV1 {
    FastqTrimMetricsV1 {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 30.0 + mean_q_delta,
        delta_metrics: FastqDeltaMetricsV1 {
            read_retention,
            base_retention: 0.9,
            mean_q_delta,
            gc_delta: 0.0,
        },
        retention: retention_metric(),
    }
}

fn filter_metrics(n_rate: f64) -> FastqFilterMetricsV1 {
    let reads_in = 100_u64;
    let reads_removed_by_n = (n_rate * reads_in as f64).round() as u64;
    FastqFilterMetricsV1 {
        reads_in,
        reads_out: reads_in.saturating_sub(reads_removed_by_n),
        reads_dropped: reads_removed_by_n,
        reads_removed_by_n,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity: 0,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetricsV1 {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.0,
        },
        retention: retention_metric(),
    }
}

fn merge_metrics(merge_rate: f64) -> FastqMergeMetricsV1 {
    FastqMergeMetricsV1 {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: 50,
        pairs_out: 50,
        reads_r1: 50,
        reads_r2: 50,
        reads_merged: 60,
        reads_unmerged: 40,
        merge_rate,
    }
}

fn validate_metrics(reads_invalid: u64) -> FastqValidateMetricsV1 {
    FastqValidateMetricsV1 {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: None,
        pairs_out: None,
        reads_total: 100,
        reads_valid: 100 - reads_invalid,
        reads_invalid,
        mean_q: 30.0,
    }
}

fn effective_params_trim() -> serde_json::Value {
    serde_json::to_value(TrimEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: 2,
        min_len: 30,
        q_cutoff: Some(20),
        adapter_policy: "auto".to_string(),
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
    })
    .unwrap()
}

fn effective_params_filter() -> serde_json::Value {
    serde_json::to_value(FilterEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: 2,
        max_n: Some(1),
        max_n_fraction: None,
        max_n_count: None,
        low_complexity_threshold: None,
        entropy_threshold: None,
        contaminant_db: None,
        n_policy: None,
        polyx_policy: None,
    })
    .unwrap()
}

fn effective_params_merge() -> serde_json::Value {
    serde_json::to_value(MergeEffectiveParams {
        paired_mode: PairedMode::PairedEnd,
        threads: 2,
        merge_overlap: Some(10),
        min_len: Some(30),
    })
    .unwrap()
}

fn effective_params_validate() -> serde_json::Value {
    serde_json::to_value(ValidateEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: 2,
        q_cutoff: None,
    })
    .unwrap()
}

fn fixture_for_invariant(id: &str) -> (String, serde_json::Value, serde_json::Value) {
    match id {
        "effective_params_present" => (
            "fastq.trim".to_string(),
            serde_json::to_value(trim_metrics(0.9, 1.0)).unwrap(),
            serde_json::json!({}),
        ),
        "metrics_parse" => (
            "fastq.trim".to_string(),
            serde_json::json!({}),
            effective_params_trim(),
        ),
        "retention_sanity" => (
            "fastq.trim".to_string(),
            serde_json::to_value(trim_metrics(0.1, 1.0)).unwrap(),
            effective_params_trim(),
        ),
        "quality_direction" => (
            "fastq.trim".to_string(),
            serde_json::to_value(trim_metrics(0.9, -10.0)).unwrap(),
            effective_params_trim(),
        ),
        "merge_rate_range" => (
            "fastq.merge".to_string(),
            serde_json::to_value(merge_metrics(1.5)).unwrap(),
            effective_params_merge(),
        ),
        "n_rate_sanity" => (
            "fastq.filter".to_string(),
            serde_json::to_value(filter_metrics(0.1)).unwrap(),
            effective_params_filter(),
        ),
        "validate_malformed_reads" => (
            "fastq.validate_pre".to_string(),
            serde_json::to_value(validate_metrics(1)).unwrap(),
            effective_params_validate(),
        ),
        _ => panic!("missing fixture for invariant {id}"),
    }
}

#[test]
fn fastq_invariants_have_specs_and_fixtures() -> Result<()> {
    let specs = fastq_invariant_specs();
    let mut ids = std::collections::BTreeSet::new();
    for spec in &specs {
        assert!(
            ids.insert(spec.id.clone()),
            "duplicate invariant {}",
            spec.id
        );
        assert!(!spec.definition.trim().is_empty());
        assert!(!spec.threshold_provenance.trim().is_empty());
        assert!(!spec.next_steps.trim().is_empty());
        let (stage_id, metrics, params) = fixture_for_invariant(&spec.id);
        let eval = evaluate_invariants(&stage_id, &metrics, &params, &thresholds_from_env());
        let status = eval
            .results
            .iter()
            .find(|entry| entry.id == spec.id)
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| panic!("missing invariant {} for {stage_id}", spec.id));
        assert!(
            status != bijux_core::InvariantStatusV1::Pass,
            "fixture did not trigger {}",
            spec.id
        );
    }
    Ok(())
}
